use crate::launcher_protocol::SettingsPatch;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Prepared,
    Staged,
    Activating,
    Verified,
    NeedsReview,
    Cancelled,
    Restored,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Record {
    pub schema_version: u32,
    pub id: String,
    pub session_id: String,
    pub identity: Value,
    pub expected_revision: String,
    pub previous_overrides: Map<String, Value>,
    pub owned_keys: Vec<String>,
    pub previous_collections: Vec<Value>,
    pub patch: SettingsPatch,
    pub collections: Vec<Value>,
    pub phase: Phase,
    pub resulting_revision: Option<String>,
    pub restore_of: Option<String>,
    pub expires_at: u64,
}

impl Record {
    pub fn check_before_write(&self, identity: &Value, revision: &str, now: u64) -> Result<(), String> {
        if &self.identity != identity {
            return Err("This plan belongs to a different TV, firmware or user. Refresh and review again.".into());
        }
        if !matches!(self.phase, Phase::Prepared | Phase::Staged) {
            return Err("Reconcile this operation before sending any more changes.".into());
        }
        if now > self.expires_at {
            return Err("This review has expired. Refresh the TV and review the changes again.".into());
        }
        if revision != self.expected_revision {
            return Err("The TV’s setup changed. Refresh and review your changes again.".into());
        }
        Ok(())
    }

    pub fn inverse_patch(&self, current: &Map<String, Value>) -> Result<SettingsPatch, String> {
        if self.phase != Phase::Verified {
            return Err("Only a verified operation can be restored.".into());
        }
        let mut inverse = SettingsPatch::default();
        for key in &self.owned_keys {
            let expected = self.patch.set.get(key);
            if current.get(key) != expected {
                return Err("This setting changed after the companion applied it. Keep the current value or review a new plan.".into());
            }
            match self.previous_overrides.get(key) {
                Some(value) => { inverse.set.insert(key.clone(), value.clone()); }
                None => inverse.remove.push(key.clone()),
            }
        }
        Ok(inverse)
    }
}

#[derive(Debug, Serialize)]
pub struct Change {
    pub key: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct Review {
    pub id: Option<String>,
    pub changes: Vec<Change>,
    pub collections: usize,
    pub baseline_location: Option<String>,
}

pub struct Journal {
    root: PathBuf,
}

pub struct DeviceLock {
    _file: File,
}

fn private_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

fn private_directory(path: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WritePoint {
    FileSynced,
    Renamed,
}

fn atomic_json(
    path: &Path,
    data: &[u8],
    checkpoint: impl Fn(WritePoint) -> io::Result<()>,
) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| io::Error::other("Missing journal directory"))?;
    let temporary = parent.join(format!(".{}.tmp", crate::launcher_transport::new_id().map_err(io::Error::other)?));
    let result = (|| {
        let mut file = private_options().create_new(true).open(&temporary)?;
        file.write_all(data)?;
        file.sync_all()?;
        checkpoint(WritePoint::FileSynced)?;
        fs::rename(&temporary, path)?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        checkpoint(WritePoint::Renamed)
    })();
    if temporary.exists() {
        let _ = fs::remove_file(temporary);
    }
    result
}

pub fn device_key(identity: &Value) -> Result<String, String> {
    let bytes = serde_json::to_vec(identity).map_err(|_| "Could not identify this TV.")?;
    Ok(Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Differences concern effective values; removing an override restores inheritance.
pub fn differences(
    effective: &Map<String, Value>,
    overrides: &Map<String, Value>,
    desired: &SettingsPatch,
) -> (SettingsPatch, Vec<Change>) {
    let mut patch = SettingsPatch::default();
    let mut changes = vec![];
    for (key, value) in &desired.set {
        if effective.get(key) != Some(value) {
            patch.set.insert(key.clone(), value.clone());
            changes.push(Change { key: key.clone(), before: effective.get(key).cloned(), after: Some(value.clone()) });
        }
    }
    for key in &desired.remove {
        if overrides.contains_key(key) {
            patch.remove.push(key.clone());
            changes.push(Change { key: key.clone(), before: effective.get(key).cloned(), after: None });
        }
    }
    (patch, changes)
}

impl Journal {
    pub fn open(root: PathBuf) -> Result<Self, String> {
        private_directory(&root.join("plans")).map_err(|_| "Could not create the local change journal.")?;
        private_directory(&root.join("locks")).map_err(|_| "Could not create local device locks.")?;
        Ok(Self { root })
    }

    pub fn lock(&self, identity: &Value) -> Result<DeviceLock, String> {
        let path = self.root.join("locks").join(format!("{}.lock", device_key(identity)?));
        let file = private_options().read(true).open(path).map_err(|_| "Could not open this TV’s operation lock.")?;
        file.try_lock().map_err(|_| "Another companion operation is using this TV. Wait for it to finish.")?;
        Ok(DeviceLock { _file: file })
    }

    pub fn path(&self, id: &str) -> Result<PathBuf, String> {
        if !crate::launcher_protocol::hex_id(id, 32) {
            return Err("Invalid change plan identifier.".into());
        }
        Ok(self.root.join("plans").join(format!("{id}.json")))
    }

    pub fn save(&self, record: &Record) -> Result<(), String> {
        let path = self.path(&record.id)?;
        let bytes = serde_json::to_vec(record).map_err(|_| "Could not encode the change journal.")?;
        if bytes.len() > 1024 * 1024 {
            return Err("This plan exceeds the local journal limit.".into());
        }
        atomic_json(&path, &bytes, |_| Ok(())).map_err(|_| "Could not durably save the change journal. The TV operation was not confirmed.".to_owned())
    }

    pub fn load(&self, id: &str) -> Result<Record, String> {
        let path = self.path(id)?;
        if fs::metadata(&path).map_err(|_| "This change plan could not be found.")?.len() > 1024 * 1024 {
            return Err("The change journal exceeds its size limit.".into());
        }
        let bytes = fs::read(path).map_err(|_| "Could not read the change journal.")?;
        let record: Record = serde_json::from_slice(&bytes).map_err(|_| "The change journal is invalid. No TV changes were made.")?;
        if record.schema_version != 1 || record.id != id || !crate::launcher_protocol::hex_id(&record.session_id, 32) {
            return Err("This change journal version is not supported.".into());
        }
        Ok(record)
    }

    pub fn prepare(
        &self,
        identity: Value,
        state: &Value,
        desired: SettingsPatch,
        collections: Vec<Value>,
    ) -> Result<Review, String> {
        self.prepare_mode(identity, state, desired, collections, None)
    }

    /// Restore override presence even when its value equals an inherited value.
    pub fn prepare_restore(&self, original: &Record, identity: Value, state: &Value) -> Result<Review, String> {
        if identity != original.identity { return Err("This restore belongs to a different TV or user.".into()); }
        let overrides = state["overrides"].as_object().ok_or("Missing current setting ownership.")?;
        let patch = original.inverse_patch(overrides)?;
        let current = state["collections"].as_array().ok_or("Missing current collections.")?;
        for applied in &original.collections {
            if current.iter().find(|item| item["id"] == applied["id"]) != Some(applied) {
                return Err("A collection changed after this operation. Refresh and review it manually.".into());
            }
        }
        // Old manifests remain on the TV. Restore only manifests this operation replaced.
        let collections = original.previous_collections.iter().filter(|old| original.collections.iter().any(|applied| applied["id"] == old["id"])).cloned().collect();
        self.prepare_mode(identity, state, patch, collections, Some(original.id.clone()))
    }

    fn prepare_mode(&self, identity: Value, state: &Value, desired: SettingsPatch, collections: Vec<Value>, restore_of: Option<String>) -> Result<Review, String> {
        let effective = state["settings"].as_object().ok_or("The TV did not report its current settings.")?;
        let overrides = state["overrides"].as_object().ok_or("The TV did not report its setting ownership.")?;
        let revision = state["revision"].as_str().filter(|value| crate::launcher_protocol::hex_id(value, 64))
            .ok_or("The TV did not report a valid configuration revision.")?;
        let previous_collections = state["collections"].as_array().ok_or("The TV did not report its current collections.")?;
        let (patch, changes) = differences(if restore_of.is_some() { overrides } else { effective }, overrides, &desired);
        let collections: Vec<_> = collections.into_iter().filter(|collection| !previous_collections.contains(collection)).collect();
        if changes.is_empty() && collections.is_empty() {
            return Ok(Review { id: None, changes, collections: 0, baseline_location: None });
        }
        let mut owned_keys: Vec<_> = patch.set.keys().chain(patch.remove.iter()).cloned().collect();
        owned_keys.sort();
        owned_keys.dedup();
        let previous_overrides = owned_keys.iter().filter_map(|key| overrides.get(key).map(|value| (key.clone(), value.clone()))).collect();
        let id = crate::launcher_transport::new_id()?;
        let count = collections.len();
        let record = Record {
            schema_version: 1,
            id: id.clone(),
            session_id: crate::launcher_transport::new_id()?,
            identity,
            expected_revision: revision.to_owned(),
            previous_overrides,
            previous_collections: previous_collections.clone(),
            owned_keys,
            patch,
            collections,
            phase: Phase::Prepared,
            resulting_revision: None,
            restore_of,
            expires_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() + 900,
        };
        self.save(&record)?;
        Ok(Review { id: Some(id.clone()), changes, collections: count, baseline_location: Some(self.path(&id)?.to_string_lossy().into_owned()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Keep the subprocess fork from transiently inheriting another lock test’s open descriptor.
    static LOCK_TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());
    struct Temporary(PathBuf);
    impl Temporary {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("cytv-journal-{}", crate::launcher_transport::new_id().unwrap()));
            private_directory(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temporary {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    fn state() -> Value {
        json!({"revision":"a".repeat(64),"settings":{"wallpaper":true,"wallpaper_dim":30},"overrides":{"wallpaper_dim":30},"collections":[]})
    }

    #[test]
    fn no_op_does_not_create_a_change_record() {
        let temp = Temporary::new();
        let journal = Journal::open(temp.0.clone()).unwrap();
        let patch = serde_json::from_value(json!({"set":{"wallpaper":true}})).unwrap();
        assert!(journal.prepare(json!({"serial":"fixture"}), &state(), patch, vec![]).unwrap().id.is_none());
        assert_eq!(fs::read_dir(temp.0.join("plans")).unwrap().count(), 0);
    }

    #[test]
    fn baseline_preserves_value_and_absence_and_round_trips() {
        let temp = Temporary::new();
        let journal = Journal::open(temp.0.clone()).unwrap();
        let patch = serde_json::from_value(json!({"set":{"wallpaper":false},"remove":["wallpaper_dim"]})).unwrap();
        let review = journal.prepare(json!({"serial":"fixture"}), &state(), patch, vec![]).unwrap();
        let record = journal.load(review.id.as_ref().unwrap()).unwrap();
        assert_eq!(record.previous_overrides["wallpaper_dim"], 30);
        assert!(!record.previous_overrides.contains_key("wallpaper"));
        assert_eq!(record.owned_keys, vec!["wallpaper", "wallpaper_dim"]);
        assert_eq!(record.phase, Phase::Prepared);
        assert!(record.check_before_write(&record.identity, &"a".repeat(64), record.expires_at).is_ok());
        assert!(record.check_before_write(&json!({"serial":"different"}), &"a".repeat(64), record.expires_at).is_err());
        assert!(record.check_before_write(&record.identity, &"b".repeat(64), record.expires_at).is_err());
        assert!(record.check_before_write(&record.identity, &"a".repeat(64), record.expires_at + 1).is_err());
        assert!(journal.load("../../outside").is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(journal.path(&record.id).unwrap()).unwrap().permissions().mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn restoration_requires_unchanged_owned_values_and_preserves_absence() {
        let temp = Temporary::new();
        let journal = Journal::open(temp.0.clone()).unwrap();
        let patch = serde_json::from_value(json!({"set":{"wallpaper":false},"remove":["wallpaper_dim"]})).unwrap();
        let review = journal.prepare(json!({"serial":"fixture"}), &state(), patch, vec![]).unwrap();
        let mut record = journal.load(review.id.as_ref().unwrap()).unwrap();
        record.phase = Phase::Verified;
        let current = json!({"wallpaper":false,"unrelated":42});
        let inverse = record.inverse_patch(current.as_object().unwrap()).unwrap();
        assert_eq!(inverse.set["wallpaper_dim"], 30);
        assert_eq!(inverse.remove, vec!["wallpaper"]);
        assert!(!inverse.set.contains_key("unrelated"));
        assert!(record.inverse_patch(json!({"wallpaper":true}).as_object().unwrap()).is_err());
    }

    #[test]
    fn restore_records_prior_override_even_when_inherited_value_is_identical() {
        let temp = Temporary::new();
        let journal = Journal::open(temp.0.clone()).unwrap();
        let patch = serde_json::from_value(json!({"remove":["wallpaper_dim"]})).unwrap();
        let identity = json!({"serial":"fixture"});
        let review = journal.prepare(identity.clone(), &state(), patch, vec![]).unwrap();
        let mut original = journal.load(review.id.as_ref().unwrap()).unwrap();
        original.phase = Phase::Verified;
        let mut current = state();
        current["overrides"] = json!({});
        let restore = journal.prepare_restore(&original, identity, &current).unwrap();
        let inverse = journal.load(restore.id.as_ref().unwrap()).unwrap();
        assert_eq!(inverse.patch.set["wallpaper_dim"], 30);
        assert_eq!(inverse.restore_of.as_deref(), Some(original.id.as_str()));
        current["overrides"] = json!({"wallpaper_dim":50});
        assert!(journal.prepare_restore(&original, original.identity.clone(), &current).is_err());
    }

    #[test]
    fn device_lock_is_exclusive_across_file_handles_and_released_on_drop() {
        let _test_guard = LOCK_TEST.lock().unwrap();
        let temp = Temporary::new();
        let journal = Journal::open(temp.0.clone()).unwrap();
        let identity = json!({"serial":"fixture"});
        let guard = journal.lock(&identity).unwrap();
        assert!(journal.lock(&identity).is_err());
        assert!(journal.lock(&json!({"serial":"different"})).is_ok());
        drop(guard);
        assert!(journal.lock(&identity).is_ok());
    }

    #[test]
    fn device_lock_excludes_another_process() {
        let _test_guard = LOCK_TEST.lock().unwrap();
        let temp = Temporary::new();
        let journal = Journal::open(temp.0.clone()).unwrap();
        let mut guard = Some(journal.lock(&json!({"serial":"child-fixture"})).unwrap());
        for blocked in [true, false] {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--ignored", "--exact", "journal::tests::lock_child"])
                .env("CYTV_TEST_LOCK_DIRECTORY", &temp.0)
                .env("CYTV_TEST_LOCK_BLOCKED", if blocked { "1" } else { "0" })
                .output().unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stdout));
            if blocked { drop(guard.take()); }
        }
        drop(guard);

    }

    #[test]
    #[ignore = "subprocess helper invoked by device_lock_excludes_another_process"]
    fn lock_child() {
        let path = std::env::var_os("CYTV_TEST_LOCK_DIRECTORY").expect("child lock directory");
        let blocked = std::env::var("CYTV_TEST_LOCK_BLOCKED").unwrap() == "1";
        let journal = Journal::open(PathBuf::from(path)).unwrap();
        assert_eq!(journal.lock(&json!({"serial":"child-fixture"})).is_err(), blocked);
    }

    #[test]
    fn crash_boundaries_leave_either_the_old_or_the_new_complete_record() {
        for fail_at in [WritePoint::FileSynced, WritePoint::Renamed] {
            let temp = Temporary::new();
            let path = temp.0.join("record.json");
            atomic_json(&path, br#"{"phase":"prepared"}"#, |_| Ok(())).unwrap();
            assert!(atomic_json(&path, br#"{"phase":"activating"}"#, |point| {
                if point == fail_at { Err(io::Error::other("injected interruption")) } else { Ok(()) }
            }).is_err());
            let observed: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            assert_eq!(observed["phase"], if fail_at == WritePoint::FileSynced { "prepared" } else { "activating" });
        }
    }
}
