//! Offline core for durable, typed TV setup operations.
//!
//! This module intentionally exposes no Tauri command, ADB adapter, shell arguments,
//! or artifact path. A future trusted backend must implement `SetupBackend` and keep
//! its transport separate from this journal and state machine.

use crate::journal::{atomic_json, private_directory, DeviceLock, Journal};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, io::Read, path::PathBuf};

const SCHEMA_VERSION: u32 = 2;
const MAX_SETUP_PLAN_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TrustedArtifact {
    /// Opaque release identifier, never a URL or a local filesystem path.
    pub reference: String,
    pub sha256: String,
    pub package: String,
    pub version: u64,
    pub signer_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InstalledLauncher {
    pub package: String,
    pub version: u64,
    pub signer_sha256: String,
    pub artifact_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HomeSelection {
    pub package: String,
    pub activity: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SetupOperation {
    InstallLauncher {
        artifact: TrustedArtifact,
        before: Option<InstalledLauncher>,
        after: InstalledLauncher,
    },
    SelectHome {
        before: HomeSelection,
        after: HomeSelection,
    },
    SetPackageEnabled {
        package: String,
        before: bool,
        after: bool,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum OperationState {
    Installed(Option<InstalledLauncher>),
    Home(Option<HomeSelection>),
    PackageEnabled(bool),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SetupPhase {
    Pending,
    Intent,
    Verified,
    Conflict,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SetupPlan {
    pub schema_version: u32,
    pub id: String,
    /// Exact identity snapshot. It remains private in this local journal.
    pub identity: Value,
    pub expected_revision: String,
    pub operation: SetupOperation,
    pub phase: SetupPhase,
    pub created_at: u64,
    pub expires_at: u64,
    pub restored_from: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub identity: Value,
    pub revision: String,
    pub state: OperationState,
}

/// Fake-only contract until a trusted, supported launcher installation profile exists.
pub trait SetupBackend {
    fn observe(
        &mut self,
        identity: &Value,
        operation: &SetupOperation,
    ) -> Result<Observation, String>;
    fn apply(&mut self, identity: &Value, operation: &SetupOperation) -> Result<(), String>;
}

pub struct SetupStore {
    root: PathBuf,
    locks: Journal,
}

impl SetupStore {
    /// `root` is the existing operations root, so setup plans share its per-device lock.
    /// The v2 records live in a separate directory and legacy v1 enumeration is unchanged.
    pub fn open(root: PathBuf) -> Result<Self, String> {
        let locks = Journal::open(root.clone())?;
        private_directory(&root.join("setup_plans"))
            .map_err(|_| "Could not create the local setup operation journal.")?;
        Ok(Self { root, locks })
    }

    fn path(&self, id: &str) -> Result<PathBuf, String> {
        if !crate::launcher_protocol::hex_id(id, 32) {
            return Err("Invalid setup operation identifier.".into());
        }
        Ok(self.root.join("setup_plans").join(format!("{id}.json")))
    }

    pub fn save(&self, plan: &SetupPlan) -> Result<(), String> {
        validate_plan(plan)?;
        let bytes = serde_json::to_vec(plan).map_err(|_| "Could not encode setup operation.")?;
        if bytes.len() as u64 > MAX_SETUP_PLAN_BYTES {
            return Err("Setup operation exceeds the local journal limit.".into());
        }
        atomic_json(&self.path(&plan.id)?, &bytes)
            .map_err(|_| "Could not durably save setup operation.".to_owned())
    }

    pub fn load(&self, id: &str) -> Result<SetupPlan, String> {
        let path = self.path(id)?;
        let metadata = fs::symlink_metadata(&path).map_err(|_| "Setup operation was not found.")?;
        if !metadata.file_type().is_file() || metadata.len() > MAX_SETUP_PLAN_BYTES {
            return Err("Setup operation is not a supported regular file.".into());
        }
        let mut bytes = Vec::new();
        fs::File::open(path)
            .map_err(|_| "Could not read setup operation.")?
            .take(MAX_SETUP_PLAN_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Could not read setup operation.")?;
        if bytes.len() as u64 > MAX_SETUP_PLAN_BYTES {
            return Err("Setup operation exceeds the local journal limit.".into());
        }
        let plan: SetupPlan =
            serde_json::from_slice(&bytes).map_err(|_| "Setup operation is invalid.")?;
        validate_plan(&plan)?;
        if plan.id != id {
            return Err("Setup operation identifier does not match its record.".into());
        }
        Ok(plan)
    }

    pub fn lock(&self, identity: &Value) -> Result<DeviceLock, String> {
        self.locks.lock(identity)
    }
}

pub fn prepare(
    store: &SetupStore,
    identity: Value,
    expected_revision: String,
    operation: SetupOperation,
    created_at: u64,
) -> Result<SetupPlan, String> {
    validate_identity(&identity)?;
    validate_revision(&expected_revision)?;
    validate_operation(&operation)?;
    let id =
        crate::launcher_transport::new_id().map_err(|_| "Could not create setup operation.")?;
    let expires_at = created_at
        .checked_add(15 * 60)
        .ok_or("Invalid review expiry.")?;
    let plan = SetupPlan {
        schema_version: SCHEMA_VERSION,
        id,
        identity,
        expected_revision,
        operation,
        phase: SetupPhase::Pending,
        created_at,
        expires_at,
        restored_from: None,
    };
    store.save(&plan)?;
    Ok(plan)
}

/// Apply a reviewed plan once, persisting intent before calling the typed backend.
pub fn apply_reviewed<B: SetupBackend>(
    store: &SetupStore,
    backend: &mut B,
    id: &str,
    now: u64,
) -> Result<SetupPlan, String> {
    let mut plan = store.load(id)?;
    let _lock = store.lock(&plan.identity)?;
    if plan.phase != SetupPhase::Pending {
        return Err("Reconcile this setup operation before applying it again.".into());
    }
    apply_pending(store, backend, &mut plan, now)?;
    Ok(plan)
}

/// Read state only. An intent observed at its prior state returns to Pending; this function
/// never retries a write. Retrying requires a separate call to `resume_reviewed`.
pub fn reconcile<B: SetupBackend>(
    store: &SetupStore,
    backend: &mut B,
    id: &str,
) -> Result<SetupPlan, String> {
    let mut plan = store.load(id)?;
    let _lock = store.lock(&plan.identity)?;
    if plan.phase == SetupPhase::Intent {
        reconcile_locked(store, backend, &mut plan)?;
    }
    Ok(plan)
}

/// Explicit retry path. It first reconciles any interrupted intent, then writes only if
/// identity, revision and operation-owned prior state still match the reviewed plan.
pub fn resume_reviewed<B: SetupBackend>(
    store: &SetupStore,
    backend: &mut B,
    id: &str,
    now: u64,
) -> Result<SetupPlan, String> {
    let mut plan = store.load(id)?;
    let _lock = store.lock(&plan.identity)?;
    if plan.phase == SetupPhase::Intent {
        reconcile_locked(store, backend, &mut plan)?;
    }
    match plan.phase {
        SetupPhase::Pending => apply_pending(store, backend, &mut plan, now)?,
        SetupPhase::Verified | SetupPhase::Conflict => {}
        SetupPhase::Intent => {
            return Err("Setup outcome remains uncertain. Reconcile before continuing.".into())
        }
    }
    Ok(plan)
}

fn apply_pending<B: SetupBackend>(
    store: &SetupStore,
    backend: &mut B,
    plan: &mut SetupPlan,
    now: u64,
) -> Result<(), String> {
    if now > plan.expires_at {
        return mark_conflict(store, plan);
    }
    let observed = backend.observe(&plan.identity, &plan.operation)?;
    if observed.identity != plan.identity {
        mark_conflict(store, plan)?;
        return Ok(());
    }
    if observed.revision != plan.expected_revision || observed.state != plan.operation.before() {
        mark_conflict(store, plan)?;
        return Ok(());
    }
    if plan.operation.before() == plan.operation.after() {
        plan.phase = SetupPhase::Verified;
        store.save(plan)?;
        return Ok(());
    }
    plan.phase = SetupPhase::Intent;
    store.save(plan)?;
    // Any error leaves durable Intent. Reconcile reads actual state before another attempt.
    backend.apply(&plan.identity, &plan.operation)?;
    reconcile_locked(store, backend, plan)
}

fn reconcile_locked<B: SetupBackend>(
    store: &SetupStore,
    backend: &mut B,
    plan: &mut SetupPlan,
) -> Result<(), String> {
    let observed = backend.observe(&plan.identity, &plan.operation)?;
    if observed.identity != plan.identity {
        return mark_conflict(store, plan);
    }
    if observed.state == plan.operation.after() {
        plan.phase = SetupPhase::Verified;
    } else if observed.state == plan.operation.before()
        && observed.revision == plan.expected_revision
    {
        plan.phase = SetupPhase::Pending;
    } else {
        plan.phase = SetupPhase::Conflict;
    }
    store.save(plan)
}

fn mark_conflict(store: &SetupStore, plan: &mut SetupPlan) -> Result<(), String> {
    plan.phase = SetupPhase::Conflict;
    store.save(plan)
}

/// Prepare a reviewed inverse for a verified HOME/package-state operation only.
/// The inverse's `before` is the operation's applied value, never an installation baseline.
pub fn prepare_inverse(
    store: &SetupStore,
    original_id: &str,
    current_identity: Value,
    current_revision: String,
    current_state: OperationState,
    created_at: u64,
) -> Result<SetupPlan, String> {
    let original = store.load(original_id)?;
    if original.phase != SetupPhase::Verified {
        return Err("Only a verified setup operation can be restored.".into());
    }
    if original.identity != current_identity {
        return Err("This restore belongs to a different TV identity.".into());
    }
    validate_revision(&current_revision)?;
    if matches!(&original.operation, SetupOperation::InstallLauncher { .. }) {
        return Err("Launcher installation cannot be rolled back as an APK downgrade.".into());
    }
    let inverse = match original.operation {
        SetupOperation::SelectHome { before, after } => {
            if current_state != OperationState::Home(Some(after.clone())) {
                return Err(
                    "HOME changed after setup. Keep the current selection and review again.".into(),
                );
            }
            SetupOperation::SelectHome {
                before: after,
                after: before,
            }
        }
        SetupOperation::SetPackageEnabled {
            package,
            before,
            after,
        } => {
            if current_state != OperationState::PackageEnabled(after) {
                return Err(
                    "Package state changed after setup. Keep the current state and review again."
                        .into(),
                );
            }
            SetupOperation::SetPackageEnabled {
                package,
                before: after,
                after: before,
            }
        }
        SetupOperation::InstallLauncher { .. } => unreachable!(),
    };
    let mut plan = prepare(
        store,
        current_identity,
        current_revision,
        inverse,
        created_at,
    )?;
    plan.restored_from = Some(original.id);
    store.save(&plan)?;
    Ok(plan)
}

impl SetupOperation {
    pub fn before(&self) -> OperationState {
        match self {
            Self::InstallLauncher { before, .. } => OperationState::Installed(before.clone()),
            Self::SelectHome { before, .. } => OperationState::Home(Some(before.clone())),
            Self::SetPackageEnabled { before, .. } => OperationState::PackageEnabled(*before),
        }
    }

    pub fn after(&self) -> OperationState {
        match self {
            Self::InstallLauncher { after, .. } => OperationState::Installed(Some(after.clone())),
            Self::SelectHome { after, .. } => OperationState::Home(Some(after.clone())),
            Self::SetPackageEnabled { after, .. } => OperationState::PackageEnabled(*after),
        }
    }
}

fn validate_plan(plan: &SetupPlan) -> Result<(), String> {
    if plan.schema_version != SCHEMA_VERSION || !crate::launcher_protocol::hex_id(&plan.id, 32) {
        return Err("Setup operation version or identifier is unsupported.".into());
    }
    validate_identity(&plan.identity)?;
    validate_revision(&plan.expected_revision)?;
    validate_operation(&plan.operation)?;
    if plan.expires_at <= plan.created_at || plan.expires_at - plan.created_at != 15 * 60 {
        return Err("Setup operation review expiry is invalid.".into());
    }
    if let Some(id) = &plan.restored_from {
        if !crate::launcher_protocol::hex_id(id, 32) {
            return Err("Invalid restore operation reference.".into());
        }
    }
    Ok(())
}

fn validate_identity(identity: &Value) -> Result<(), String> {
    let parsed: crate::TvIdentity = serde_json::from_value(identity.clone())
        .map_err(|_| "Setup operation requires a complete verified identity.")?;
    let normalized = serde_json::to_value(&parsed)
        .map_err(|_| "Setup operation requires a complete verified identity.")?;
    if normalized != *identity
        || !crate::identity_value(&parsed.serial)
        || !crate::identity_value(&parsed.manufacturer)
        || !crate::identity_value(&parsed.model)
        || !crate::identity_value(&parsed.fingerprint)
        || parsed.user.is_empty()
        || !parsed.user.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err("Setup operation requires a complete verified identity.".into());
    }
    Ok(())
}

fn validate_revision(revision: &str) -> Result<(), String> {
    if crate::launcher_protocol::hex_id(revision, 64) {
        Ok(())
    } else {
        Err("Invalid setup state revision.".into())
    }
}

fn validate_operation(operation: &SetupOperation) -> Result<(), String> {
    match operation {
        SetupOperation::InstallLauncher {
            artifact,
            before,
            after,
        } => {
            validate_artifact(artifact)?;
            if let Some(before) = before {
                validate_installed(before)?;
                if before.package != artifact.package
                    || before.signer_sha256 != artifact.signer_sha256
                {
                    return Err("Existing launcher package and signer must match the trusted update target.".into());
                }
                if after.version < before.version {
                    return Err("Launcher downgrade is not supported by setup operations.".into());
                }
            }
            validate_installed(after)?;
            if after.package != artifact.package
                || after.version != artifact.version
                || after.signer_sha256 != artifact.signer_sha256
                || after.artifact_sha256 != artifact.sha256
            {
                return Err("Launcher result does not match the trusted artifact.".into());
            }
        }
        SetupOperation::SelectHome { before, after } => {
            validate_home(before)?;
            validate_home(after)?;
        }
        SetupOperation::SetPackageEnabled { package, .. } => validate_package(package)?,
    }
    Ok(())
}

fn validate_artifact(artifact: &TrustedArtifact) -> Result<(), String> {
    if artifact.reference.len() > 160
        || !artifact.reference.starts_with("release:")
        || !artifact.reference[8..]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        || artifact.reference.len() == 8
    {
        return Err("Artifact reference must be an opaque trusted release identifier.".into());
    }
    validate_hash(&artifact.sha256)?;
    validate_hash(&artifact.signer_sha256)?;
    validate_package(&artifact.package)?;
    if artifact.version == 0 {
        return Err("Launcher version must be a positive integer.".into());
    }
    Ok(())
}

fn validate_installed(installed: &InstalledLauncher) -> Result<(), String> {
    validate_package(&installed.package)?;
    validate_hash(&installed.signer_sha256)?;
    validate_hash(&installed.artifact_sha256)?;
    if installed.version == 0 {
        return Err("Launcher version must be a positive integer.".into());
    }
    Ok(())
}

fn validate_home(home: &HomeSelection) -> Result<(), String> {
    validate_package(&home.package)?;
    let activity = home
        .activity
        .strip_prefix('.')
        .map(|name| format!("{}.{name}", home.package))
        .unwrap_or_else(|| home.activity.clone());
    if activity.len() > 255
        || !activity.starts_with(&format!("{}.", home.package))
        || !valid_dotted_identifier(&activity)
    {
        return Err("HOME activity must be a class within its validated package.".into());
    }
    Ok(())
}

fn validate_package(package: &str) -> Result<(), String> {
    if package.len() > 255 || !valid_dotted_identifier(package) || !package.contains('.') {
        return Err("Invalid Android package identifier.".into());
    }
    Ok(())
}

fn valid_dotted_identifier(value: &str) -> bool {
    value.split('.').all(|part| {
        let mut bytes = part.bytes();
        matches!(bytes.next(), Some(first) if first.is_ascii_alphabetic() || first == b'_')
            && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
    })
}

fn validate_hash(hash: &str) -> Result<(), String> {
    if crate::launcher_protocol::hex_id(hash, 64) {
        Ok(())
    } else {
        Err("Expected a lowercase SHA-256 digest.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        journal::{Journal, Phase as JournalPhase},
        launcher_protocol::SettingsPatch,
    };
    use serde_json::json;

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "cytv-setup-{}",
                crate::launcher_transport::new_id().unwrap()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn rev_a() -> String {
        "a".repeat(64)
    }
    fn rev_b() -> String {
        "b".repeat(64)
    }
    fn identity() -> Value {
        json!({"serial":"fixture-tv","manufacturer":"Fixture","model":"TV","user":"0","fingerprint":"fixture-build"})
    }
    fn home(name: &str) -> HomeSelection {
        HomeSelection {
            package: "com.example.home".into(),
            activity: format!(".{name}"),
        }
    }
    fn package_plan(before: bool, after: bool) -> SetupOperation {
        SetupOperation::SetPackageEnabled {
            package: "com.example.wallpaper".into(),
            before,
            after,
        }
    }
    fn plan(store: &SetupStore, operation: SetupOperation) -> SetupPlan {
        prepare(store, identity(), rev_a(), operation, 10).unwrap()
    }

    #[derive(Clone, Copy)]
    enum Failure {
        None,
        BeforeEffect,
        AfterEffect,
    }
    struct FakeBackend {
        identity: Value,
        revision: String,
        state: OperationState,
        applies: usize,
        failure: Failure,
        durable_intent_seen: bool,
        store_root: Option<PathBuf>,
        plan_id: Option<String>,
    }
    impl FakeBackend {
        fn new(state: OperationState) -> Self {
            Self {
                identity: identity(),
                revision: rev_a(),
                state,
                applies: 0,
                failure: Failure::None,
                durable_intent_seen: false,
                store_root: None,
                plan_id: None,
            }
        }
    }
    impl SetupBackend for FakeBackend {
        fn observe(
            &mut self,
            _identity: &Value,
            _operation: &SetupOperation,
        ) -> Result<Observation, String> {
            Ok(Observation {
                identity: self.identity.clone(),
                revision: self.revision.clone(),
                state: self.state.clone(),
            })
        }
        fn apply(&mut self, identity: &Value, operation: &SetupOperation) -> Result<(), String> {
            assert_eq!(identity, &self.identity);
            self.applies += 1;
            if let (Some(root), Some(id)) = (&self.store_root, &self.plan_id) {
                let store = SetupStore::open(root.clone()).unwrap();
                self.durable_intent_seen = store.load(id).unwrap().phase == SetupPhase::Intent;
            }
            if matches!(self.failure, Failure::BeforeEffect) {
                return Err("simulated disconnect before effect".into());
            }
            self.state = operation.after();
            self.revision = rev_b();
            if matches!(self.failure, Failure::AfterEffect) {
                return Err("simulated lost response after effect".into());
            }
            Ok(())
        }
    }

    #[test]
    fn no_op_is_verified_without_backend_write() {
        let temp = Temp::new();
        let store = SetupStore::open(temp.0.clone()).unwrap();
        let operation = package_plan(true, true);
        let mut backend = FakeBackend::new(operation.before());
        let prepared = plan(&store, operation);
        let result = apply_reviewed(&store, &mut backend, &prepared.id, 10).unwrap();
        assert_eq!(result.phase, SetupPhase::Verified);
        assert_eq!(backend.applies, 0);
    }

    #[test]
    fn expired_pending_review_is_conflicted_without_a_write() {
        let temp = Temp::new();
        let store = SetupStore::open(temp.0.clone()).unwrap();
        let operation = package_plan(false, true);
        let prepared = plan(&store, operation.clone());
        let mut backend = FakeBackend::new(operation.before());
        let result =
            apply_reviewed(&store, &mut backend, &prepared.id, prepared.expires_at + 1).unwrap();
        assert_eq!(result.phase, SetupPhase::Conflict);
        assert_eq!(backend.applies, 0);
    }

    #[test]
    fn interruption_before_effect_requires_reconcile_then_explicit_resume() {
        let temp = Temp::new();
        let store = SetupStore::open(temp.0.clone()).unwrap();
        let operation = package_plan(false, true);
        let prepared = plan(&store, operation.clone());
        let mut backend = FakeBackend::new(operation.before());
        backend.failure = Failure::BeforeEffect;
        assert!(apply_reviewed(&store, &mut backend, &prepared.id, 10).is_err());
        assert_eq!(store.load(&prepared.id).unwrap().phase, SetupPhase::Intent);
        assert_eq!(
            reconcile(&store, &mut backend, &prepared.id).unwrap().phase,
            SetupPhase::Pending
        );
        assert_eq!(backend.applies, 1);
        backend.failure = Failure::None;
        let result = resume_reviewed(&store, &mut backend, &prepared.id, 11).unwrap();
        assert_eq!(result.phase, SetupPhase::Verified);
        assert_eq!(backend.applies, 2);
    }

    #[test]
    fn interruption_after_effect_reconciles_without_repeating_write_and_intent_was_durable() {
        let temp = Temp::new();
        let store = SetupStore::open(temp.0.clone()).unwrap();
        let operation = package_plan(false, true);
        let prepared = plan(&store, operation.clone());
        let mut backend = FakeBackend::new(operation.before());
        backend.failure = Failure::AfterEffect;
        backend.store_root = Some(temp.0.clone());
        backend.plan_id = Some(prepared.id.clone());
        assert!(apply_reviewed(&store, &mut backend, &prepared.id, 10).is_err());
        assert!(backend.durable_intent_seen);
        assert_eq!(store.load(&prepared.id).unwrap().phase, SetupPhase::Intent);
        assert_eq!(
            reconcile(&store, &mut backend, &prepared.id).unwrap().phase,
            SetupPhase::Verified
        );
        assert_eq!(
            resume_reviewed(&store, &mut backend, &prepared.id, 11)
                .unwrap()
                .phase,
            SetupPhase::Verified
        );
        assert_eq!(backend.applies, 1);
    }

    #[test]
    fn stale_revision_identity_and_owned_state_drift_conflict_before_write() {
        for case in 0..3 {
            let temp = Temp::new();
            let store = SetupStore::open(temp.0.clone()).unwrap();
            let operation = package_plan(false, true);
            let prepared = plan(&store, operation.clone());
            let mut backend = FakeBackend::new(operation.before());
            match case {
                0 => backend.revision = rev_b(),
                1 => backend.identity = json!({"serial":"different-tv","user":"0"}),
                _ => backend.state = OperationState::PackageEnabled(true),
            }
            let result = apply_reviewed(&store, &mut backend, &prepared.id, 10).unwrap();
            assert_eq!(result.phase, SetupPhase::Conflict);
            assert_eq!(backend.applies, 0);
        }
    }

    #[test]
    fn post_intent_unrelated_revision_change_conflicts_instead_of_retrying() {
        let temp = Temp::new();
        let store = SetupStore::open(temp.0.clone()).unwrap();
        let operation = package_plan(false, true);
        let prepared = plan(&store, operation.clone());
        let mut backend = FakeBackend::new(operation.before());
        backend.failure = Failure::BeforeEffect;
        assert!(apply_reviewed(&store, &mut backend, &prepared.id, 10).is_err());
        backend.failure = Failure::None;
        backend.revision = rev_b();
        assert_eq!(
            reconcile(&store, &mut backend, &prepared.id).unwrap().phase,
            SetupPhase::Conflict
        );
        assert_eq!(backend.applies, 1);
    }

    #[test]
    fn inverse_uses_companion_applied_state_and_preserves_pre_disabled_package() {
        let temp = Temp::new();
        let store = SetupStore::open(temp.0.clone()).unwrap();
        let operation = package_plan(false, true);
        let original = plan(&store, operation.clone());
        let mut backend = FakeBackend::new(operation.before());
        assert_eq!(
            apply_reviewed(&store, &mut backend, &original.id, 10)
                .unwrap()
                .phase,
            SetupPhase::Verified
        );
        let inverse = prepare_inverse(
            &store,
            &original.id,
            identity(),
            rev_b(),
            backend.state.clone(),
            11,
        )
        .unwrap();
        assert_eq!(inverse.operation, package_plan(true, false));
        assert_eq!(inverse.restored_from.as_deref(), Some(original.id.as_str()));
        assert_eq!(backend.state, OperationState::PackageEnabled(true));
    }

    #[test]
    fn inverse_restores_home_only_when_unchanged_and_install_is_nonrestorable() {
        let temp = Temp::new();
        let store = SetupStore::open(temp.0.clone()).unwrap();
        let prior = home("PreviousHome");
        let chosen = home("CompanionHome");
        let original = plan(
            &store,
            SetupOperation::SelectHome {
                before: prior.clone(),
                after: chosen.clone(),
            },
        );
        let mut backend = FakeBackend::new(OperationState::Home(Some(prior.clone())));
        assert_eq!(
            apply_reviewed(&store, &mut backend, &original.id, 10)
                .unwrap()
                .phase,
            SetupPhase::Verified
        );
        let inverse = prepare_inverse(
            &store,
            &original.id,
            identity(),
            rev_b(),
            backend.state.clone(),
            12,
        )
        .unwrap();
        assert_eq!(
            inverse.operation,
            SetupOperation::SelectHome {
                before: chosen.clone(),
                after: prior
            }
        );
        assert!(prepare_inverse(
            &store,
            &original.id,
            identity(),
            rev_b(),
            OperationState::Home(None),
            12
        )
        .is_err());

        let artifact = TrustedArtifact {
            reference: "release:stable-1".into(),
            sha256: "c".repeat(64),
            package: "com.example.launcher".into(),
            version: 2,
            signer_sha256: "d".repeat(64),
        };
        let installed = InstalledLauncher {
            package: artifact.package.clone(),
            version: artifact.version,
            signer_sha256: artifact.signer_sha256.clone(),
            artifact_sha256: artifact.sha256.clone(),
        };
        let install = plan(
            &store,
            SetupOperation::InstallLauncher {
                artifact,
                before: None,
                after: installed,
            },
        );
        let install_op = store.load(&install.id).unwrap().operation;
        let mut install_backend = FakeBackend::new(install_op.before());
        assert_eq!(
            apply_reviewed(&store, &mut install_backend, &install.id, 10)
                .unwrap()
                .phase,
            SetupPhase::Verified
        );
        assert!(prepare_inverse(
            &store,
            &install.id,
            identity(),
            rev_a(),
            OperationState::Installed(None),
            13
        )
        .unwrap_err()
        .contains("cannot be rolled back"));
    }

    #[test]
    fn validation_rejects_shell_strings_bad_hashes_and_noninteger_versions() {
        let temp = Temp::new();
        let store = SetupStore::open(temp.0.clone()).unwrap();
        assert!(prepare(
            &store,
            json!({"serial":"fixture-tv","user":"0"}),
            rev_a(),
            package_plan(false, true),
            1
        )
        .is_err());
        let mut with_unknown_identity_field = identity();
        with_unknown_identity_field["extra"] = json!("unrecognized");
        assert!(prepare(
            &store,
            with_unknown_identity_field,
            rev_a(),
            SetupOperation::SetPackageEnabled {
                package: "com.good;reboot".into(),
                before: false,
                after: true
            },
            1
        )
        .is_err());
        assert!(prepare(
            &store,
            identity(),
            rev_a(),
            SetupOperation::SelectHome {
                before: home("Previous"),
                after: HomeSelection {
                    package: "com.good.home".into(),
                    activity: "com.bad/.Main".into()
                }
            },
            1
        )
        .is_err());
        let valid_before = InstalledLauncher {
            package: "com.example.launcher".into(),
            version: 2,
            signer_sha256: "d".repeat(64),
            artifact_sha256: "e".repeat(64),
        };
        let downgrade_artifact = TrustedArtifact {
            reference: "release:old".into(),
            sha256: "f".repeat(64),
            package: valid_before.package.clone(),
            version: 1,
            signer_sha256: valid_before.signer_sha256.clone(),
        };
        let downgrade = InstalledLauncher {
            package: downgrade_artifact.package.clone(),
            version: downgrade_artifact.version,
            signer_sha256: downgrade_artifact.signer_sha256.clone(),
            artifact_sha256: downgrade_artifact.sha256.clone(),
        };
        assert!(prepare(
            &store,
            identity(),
            rev_a(),
            SetupOperation::InstallLauncher {
                artifact: downgrade_artifact,
                before: Some(valid_before),
                after: downgrade
            },
            1
        )
        .unwrap_err()
        .contains("downgrade"));
        let artifact = TrustedArtifact {
            reference: "release:ok".into(),
            sha256: "x".repeat(64),
            package: "com.example.launcher".into(),
            version: 1,
            signer_sha256: "d".repeat(64),
        };
        let installed = InstalledLauncher {
            package: artifact.package.clone(),
            version: 1,
            signer_sha256: artifact.signer_sha256.clone(),
            artifact_sha256: artifact.sha256.clone(),
        };
        assert!(prepare(
            &store,
            identity(),
            rev_a(),
            SetupOperation::InstallLauncher {
                artifact,
                before: None,
                after: installed
            },
            1
        )
        .is_err());
    }

    #[test]
    fn setup_v2_storage_does_not_change_legacy_v1_journal_readability_or_history() {
        let temp = Temp::new();
        let journal = Journal::open(temp.0.clone()).unwrap();
        let identity = identity();
        let patch: SettingsPatch =
            serde_json::from_value(json!({"set":{"wallpaper":false}})).unwrap();
        let old = journal.prepare(identity.clone(), &json!({"revision":rev_a(),"settings":{"wallpaper":true},"overrides":{},"collections":[]}), patch, vec![]).unwrap();
        assert_eq!(
            journal.load(old.id.as_ref().unwrap()).unwrap().phase,
            JournalPhase::Prepared
        );
        let store = SetupStore::open(temp.0.clone()).unwrap();
        let operation = package_plan(false, true);
        let new = plan(&store, operation);
        assert_eq!(store.load(&new.id).unwrap(), new);
        assert_eq!(
            journal.load(old.id.as_ref().unwrap()).unwrap().phase,
            JournalPhase::Prepared
        );
        assert_eq!(journal.history(&identity).unwrap().operations.len(), 1);
    }
}
