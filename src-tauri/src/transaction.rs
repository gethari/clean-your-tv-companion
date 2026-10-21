//! Durable coordinator for reviewed launcher changes.
//!
//! The backend is deliberately narrow so recovery rules can be tested without ADB or a TV.
use crate::journal::{Journal, Phase, Record};
use serde_json::Value;
use std::collections::BTreeSet;
use std::future::Future;

#[derive(Clone, Debug)]
pub struct DeviceSnapshot {
    /// Stable serial/model/firmware/user identity, assembled by the native adapter.
    pub identity: Value,
    /// Safe launcher `state` response.
    pub state: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Validation {
    pub ready: bool,
    pub missing_assets: Vec<String>,
    pub revision: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivationResult {
    pub status: ResultStatus,
    pub session_id: Option<String>,
    pub revision: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResultStatus { Activated, Staged, Validated, Discarded, Unknown }

/// Native launcher adapter. Asset bytes are looked up by hash in the companion's bounded cache.
pub trait Backend {
    fn snapshot(&mut self) -> impl Future<Output = Result<DeviceSnapshot, String>> + Send;
    fn stage(&mut self, session_id: &str, expected_revision: &str, patch: &Value, collections: &[Value]) -> impl Future<Output = Result<(), String>> + Send;
    fn validate(&mut self, session_id: &str) -> impl Future<Output = Result<Validation, String>> + Send;
    fn upload(&mut self, session_id: &str, sha256: &str, bytes: u64) -> impl Future<Output = Result<(), String>> + Send;
    fn activate(&mut self, operation_id: &str, session_id: &str, expected_revision: &str) -> impl Future<Output = Result<String, String>> + Send;
    fn result(&mut self, operation_id: &str) -> impl Future<Output = Result<ActivationResult, String>> + Send;
    fn discard(&mut self, session_id: &str) -> impl Future<Output = Result<(), String>> + Send;
}

const MAX_ASSETS: usize = 200;
const MAX_COLLECTION_BYTES: u64 = 256 * 1024 * 1024;
const MAX_ASSET_BYTES: u64 = 20 * 1024 * 1024;

fn manifest_assets(record: &Record) -> Result<Vec<(String, u64)>, String> {
    let mut assets = Vec::new();
    let mut hashes = BTreeSet::new();
    let mut total = 0u64;
    for collection in &record.collections {
        let list = collection["assets"].as_array().ok_or("The saved collection manifest is invalid.")?;
        for asset in list {
            let hash = asset["sha256"].as_str().filter(|s| crate::launcher_protocol::hex_id(s, 64))
                .ok_or("The saved image manifest is invalid.")?.to_owned();
            let bytes = asset["bytes"].as_u64().filter(|n| *n > 0 && *n <= MAX_ASSET_BYTES)
                .ok_or("The saved image size is invalid.")?;
            if hashes.insert(hash.clone()) {
                total = total.checked_add(bytes).ok_or("The image collection exceeds the transfer limit.")?;
                if hashes.len() > MAX_ASSETS || total > MAX_COLLECTION_BYTES {
                    return Err("The image collection exceeds the transfer limit.".into());
                }
                assets.push((hash, bytes));
            }
        }
    }
    Ok(assets)
}

fn revision(state: &Value) -> Result<&str, String> {
    state["revision"].as_str().filter(|s| crate::launcher_protocol::hex_id(s, 64))
        .ok_or_else(|| "The TV did not report a valid configuration revision.".into())
}

fn check_identity(record: &Record, snapshot: &DeviceSnapshot) -> Result<(), String> {
    if record.identity != snapshot.identity {
        Err("This plan belongs to a different TV, firmware or user. Refresh and review again.".into())
    } else { Ok(()) }
}

fn verify_applied(record: &Record, state: &Value, expected_revision: &str) -> Result<(), String> {
    let actual_revision = revision(state)?;
    if actual_revision != expected_revision { return Err("The TV’s applied revision could not be verified.".into()); }
    let operation = &state["operation"];
    let operation = if operation.is_null() { &state["last_operation"] } else { operation };
    if operation["request_id"].as_str() != Some(&record.id)
        || operation["session_id"].as_str() != Some(&record.session_id)
        || operation["revision"].as_str() != Some(expected_revision) {
        return Err("The TV’s applied operation could not be verified.".into());
    }
    let overrides = state["overrides"].as_object().ok_or("The TV did not report setting ownership.")?;
    for key in &record.owned_keys {
        match record.patch.set.get(key) {
            Some(value) if overrides.get(key) == Some(value) => (),
            Some(_) => return Err("The TV’s applied settings did not match this plan.".into()),
            None if !overrides.contains_key(key) => (),
            None => return Err("The TV’s applied settings did not match this plan.".into()),
        }
    }
    let collections = state["collections"].as_array().ok_or("The TV did not report its current collections.")?;
    if record.collections.iter().any(|item| !collections.contains(item)) {
        return Err("The TV’s applied collections did not match this plan.".into());
    }
    Ok(())
}

fn mark_verified(journal: &Journal, record: &mut Record, resulting_revision: String) -> Result<(), String> {
    record.phase = Phase::Verified;
    record.resulting_revision = Some(resulting_revision);
    journal.save(record)?;
    if let Some(original_id) = record.restore_of.as_deref() {
        let mut original = journal.load(original_id)?;
        if !matches!(original.phase, Phase::Verified | Phase::Restored) {
            return Err("The original operation is not available for restoration.".into());
        }
        if original.phase == Phase::Verified {
            original.phase = Phase::Restored;
            journal.save(&original)?;
        }
    }
    Ok(())
}

fn mark_review(journal: &Journal, record: &mut Record) -> Result<(), String> {
    record.phase = Phase::NeedsReview;
    journal.save(record)
}

/// Reconcile a durable Activating record. It never repeats activation.
pub async fn reconcile<B: Backend>(journal: &Journal, backend: &mut B, id: &str) -> Result<Record, String> {
    let record = journal.load(id)?;
    let _lock = journal.lock(&record.identity)?;
    reconcile_locked(journal, backend, record).await
}

async fn reconcile_locked<B: Backend>(journal: &Journal, backend: &mut B, mut record: Record) -> Result<Record, String> {
    if record.phase != Phase::Activating { return Ok(record); }
    // A failed result request is inconclusive; preserve Activating for a later retry.
    let result = match backend.result(&record.id).await { Ok(value) => value, Err(error) => return Err(error) };
    let snapshot = match backend.snapshot().await { Ok(value) => value, Err(error) => return Err(error) };
    check_identity(&record, &snapshot)?;
    let result_revision = match &result {
        ActivationResult { status: ResultStatus::Activated, session_id: Some(session), revision: Some(revision) }
            if session == &record.session_id => Some(revision.clone()),
        _ => None,
    };
    let state_operation = [&snapshot.state["operation"], &snapshot.state["last_operation"]]
        .into_iter().find(|operation| operation["request_id"].as_str() == Some(&record.id)
            && operation["session_id"].as_str() == Some(&record.session_id));
    let observed_revision = state_operation.and_then(|op| op["revision"].as_str()).map(str::to_owned);
    let confirmed = result_revision.as_ref().filter(|rev| Some(*rev) == observed_revision.as_ref());
    if let Some(rev) = confirmed {
        if verify_applied(&record, &snapshot.state, rev).is_ok() {
            mark_verified(journal, &mut record, rev.clone())?;
            return Ok(record);
        }
    }
    // If both the durable result and readback show the reviewed state is still current, preserve
    // intent so a later apply can retry using the same idempotent operation ID.
    if result.status == ResultStatus::Unknown && revision(&snapshot.state)? == record.expected_revision
        && state_operation.is_none() {
        return Ok(record);
    }
    mark_review(journal, &mut record)?;
    Ok(record)
}

/// Apply a prepared plan. Journal intent is persisted before stage and Activating before activation.
pub async fn apply<B: Backend>(journal: &Journal, backend: &mut B, id: &str, now: u64) -> Result<Record, String> {
    let mut record = journal.load(id)?;
    let _lock = journal.lock(&record.identity)?;
    if record.phase == Phase::Activating {
        record = reconcile_locked(journal, backend, record).await?;
        if record.phase != Phase::Activating { return Ok(record); }
        if now > record.expires_at {
            mark_review(journal, &mut record)?;
            return Ok(record);
        }
        // Reconciliation proved no operation committed; retry with the same idempotent ID.
        let _ = backend.activate(&record.id, &record.session_id, &record.expected_revision).await;
        return reconcile_locked(journal, backend, record).await;
    }
    if record.phase == Phase::Verified && record.restore_of.is_some() {
        let revision = record.resulting_revision.clone().ok_or("The restored revision is missing.")?;
        mark_verified(journal, &mut record, revision)?;
        return Ok(record);
    }
    if !matches!(record.phase, Phase::Prepared | Phase::Staged) { return Err("Reconcile this operation before sending any more changes.".into()); }
    let snapshot = backend.snapshot().await?;
    check_identity(&record, &snapshot)?;
    let observed_revision = revision(&snapshot.state)?.to_owned();
    record.check_before_write(&snapshot.identity, &observed_revision, now)?;
    let assets = manifest_assets(&record)?;

    if record.phase == Phase::Prepared {
        backend.stage(&record.session_id, &record.expected_revision, &serde_json::to_value(&record.patch).map_err(|_| "Could not encode this plan.")?, &record.collections).await?;
        record.phase = Phase::Staged;
        journal.save(&record)?;
    }
    // Validate first and upload only hashes listed in the persisted manifest.
    let mut validation = backend.validate(&record.session_id).await?;
    if validation.revision != record.expected_revision { return Err("The TV’s setup changed. Refresh and review your changes again.".into()); }
    let declared: BTreeSet<_> = assets.iter().map(|(hash, _)| hash.clone()).collect();
    if validation.missing_assets.len() > assets.len()
        || validation.missing_assets.iter().any(|hash| !declared.contains(hash)) {
        return Err("The TV requested an image outside this plan’s manifest.".into());
    }
    let missing: BTreeSet<_> = validation.missing_assets.into_iter().collect();
    for hash in missing {
        let bytes = assets.iter().find(|(declared_hash, _)| declared_hash == &hash).unwrap().1;
        backend.upload(&record.session_id, &hash, bytes).await?;
    }
    validation = backend.validate(&record.session_id).await?;
    if validation.revision != record.expected_revision || !validation.ready || !validation.missing_assets.is_empty() {
        return Err("The TV has not verified every image. Reconnect to resume.".into());
    }
    // Last revision/identity check before persisting the irreversible phase.
    let snapshot = backend.snapshot().await?;
    check_identity(&record, &snapshot)?;
    record.check_before_write(&snapshot.identity, revision(&snapshot.state)?, now)?;
    record.phase = Phase::Activating;
    journal.save(&record)?;

    // Once phase=Activating is durable, all outcomes go through result+state reconciliation.
    let _ = backend.activate(&record.id, &record.session_id, &record.expected_revision).await;
    reconcile_locked(journal, backend, record).await
}

/// Cancel only a known pre-activation session. Activating operations must be reconciled first.
pub async fn cancel<B: Backend>(journal: &Journal, backend: &mut B, id: &str) -> Result<Record, String> {
    let mut record = journal.load(id)?;
    let _lock = journal.lock(&record.identity)?;
    if matches!(record.phase, Phase::Activating | Phase::Verified | Phase::NeedsReview | Phase::Restored) {
        return Err("This operation may already be active. Reconcile it before cancelling.".into());
    }
    let result = backend.result(&record.id).await?;
    let snapshot = backend.snapshot().await?;
    check_identity(&record, &snapshot)?;
    let observed_operation = [&snapshot.state["operation"], &snapshot.state["last_operation"]]
        .into_iter().any(|op| op["request_id"].as_str() == Some(&record.id)
            && op["session_id"].as_str() == Some(&record.session_id));
    if result.status == ResultStatus::Activated || observed_operation
        || revision(&snapshot.state)? != record.expected_revision {
        return Err("This operation is already active and cannot be cancelled.".into());
    }
    // discard is idempotent for a missing session and rejects any session already activated.
    backend.discard(&record.session_id).await?;
    record.phase = Phase::Cancelled;
    journal.save(&record)?;
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::{Journal, Phase};
    use crate::launcher_protocol::SettingsPatch;
    use serde_json::json;

    struct Fake {
        identity: Value,
        state: Value,
        session_id: Option<String>,
        staged: bool,
        activated: bool,
        disconnect: Option<&'static str>,
        activation_calls: usize,
        discarded: usize,
    }
    impl Fake {
        fn maybe_disconnect(&mut self, point: &'static str) -> Result<(), String> {
            if self.disconnect == Some(point) { self.disconnect = None; return Err("injected disconnect".into()); } Ok(())
        }
    }
    impl Backend for Fake {
        fn snapshot(&mut self) -> impl Future<Output=Result<DeviceSnapshot,String>> + Send {
            async move { self.maybe_disconnect("snapshot")?; Ok(DeviceSnapshot { identity: self.identity.clone(), state: self.state.clone() }) }
        }
        fn stage(&mut self, session_id: &str, _: &str, _: &Value, _: &[Value]) -> impl Future<Output=Result<(),String>> + Send {
            let session_id = session_id.to_owned();
            async move { self.maybe_disconnect("stage_before")?; self.session_id=Some(session_id); self.staged=true; self.maybe_disconnect("stage_after") }
        }
        fn validate(&mut self, _: &str) -> impl Future<Output=Result<Validation,String>> + Send {
            async move { Ok(Validation { ready:true, missing_assets:vec![], revision:self.state["revision"].as_str().unwrap().into() }) }
        }
        fn upload(&mut self, _: &str, _: &str, _: u64) -> impl Future<Output=Result<(),String>> + Send { async { Ok(()) } }
        fn activate(&mut self, operation_id: &str, session_id: &str, _: &str) -> impl Future<Output=Result<String,String>> + Send {
            let operation_id=operation_id.to_owned(); let session_id=session_id.to_owned();
            async move {
                self.activation_calls+=1;
                if self.disconnect==Some("activate_before") { self.disconnect=None; return Err("injected disconnect".into()); }
                if !self.activated {
                    self.activated=true; let resulting="b".repeat(64);
                    self.state["revision"]=json!(resulting);
                    self.state["settings"]["wallpaper"]=json!(true);
                    self.state["overrides"]["wallpaper"]=json!(true);
                    self.state["operation"]=json!({"request_id":operation_id,"session_id":session_id,"revision":resulting});
                }
                self.maybe_disconnect("activate_after")?; Ok("b".repeat(64))
            }
        }
        fn result(&mut self, _: &str) -> impl Future<Output=Result<ActivationResult,String>> + Send {
            async move {
                self.maybe_disconnect("result")?;
                Ok(if self.activated { ActivationResult { status:ResultStatus::Activated,session_id:self.session_id.clone(),revision:Some("b".repeat(64)) } }
                    else { ActivationResult { status:ResultStatus::Unknown,session_id:None,revision:None } })
            }
        }
        fn discard(&mut self, _: &str) -> impl Future<Output=Result<(),String>> + Send { async move { self.discarded+=1; Ok(()) } }
    }

    fn fixture() -> (Journal, String, Fake) {
        let root = std::env::temp_dir().join(format!("cytv-transaction-{}", crate::launcher_transport::new_id().unwrap()));
        let journal = Journal::open(root.clone()).unwrap();
        let identity = json!({"serial":"fixture"});
        let state = json!({"revision":"a".repeat(64),"settings":{"wallpaper":false},"overrides":{},"collections":[],"operation":null});
        let mut patch = SettingsPatch::default(); patch.set.insert("wallpaper".into(), json!(true));
        let review = journal.prepare(identity.clone(), &state, patch, vec![]).unwrap();
        let fake = Fake { identity, state, session_id:None, staged:false, activated:false, disconnect:None, activation_calls:0, discarded:0 };
        (journal, review.id.unwrap(), fake)
    }

    #[tokio::test]
    async fn disconnect_before_stage_can_retry_with_same_session_and_only_one_activation() {
        let (journal,id,mut fake)=fixture(); fake.disconnect=Some("stage_before");
        assert!(apply(&journal,&mut fake,&id,0).await.is_err());
        assert_eq!(journal.load(&id).unwrap().phase,Phase::Prepared);
        assert_eq!(apply(&journal,&mut fake,&id,0).await.unwrap().phase,Phase::Verified);
        assert_eq!(fake.activation_calls,1);
    }

    #[tokio::test]
    async fn disconnect_after_activation_ack_is_reconciled_without_repeating_activation() {
        let (journal,id,mut fake)=fixture(); fake.disconnect=Some("activate_after");
        let record=apply(&journal,&mut fake,&id,0).await.unwrap();
        assert_eq!(record.phase,Phase::Verified); assert_eq!(fake.activation_calls,1);
    }

    #[tokio::test]
    async fn disconnect_before_activation_retries_only_after_result_and_state_confirm_no_effect() {
        let (journal,id,mut fake)=fixture(); fake.disconnect=Some("activate_before");
        let first=apply(&journal,&mut fake,&id,0).await.unwrap();
        assert_eq!(first.phase,Phase::Activating); assert_eq!(fake.activation_calls,1);
        let second=apply(&journal,&mut fake,&id,0).await.unwrap();
        assert_eq!(second.phase,Phase::Verified); assert_eq!(fake.activation_calls,2);
    }

    #[tokio::test]
    async fn identity_or_revision_drift_stops_before_staging() {
        let (journal,id,mut fake)=fixture(); fake.state["revision"]=json!("c".repeat(64));
        assert!(apply(&journal,&mut fake,&id,0).await.is_err()); assert!(!fake.staged);
        let (journal,id,mut fake)=fixture(); fake.identity=json!({"serial":"other"});
        assert!(apply(&journal,&mut fake,&id,0).await.is_err()); assert!(!fake.staged);
    }

    #[tokio::test]
    async fn cancellation_cannot_discard_a_session_after_activation_started() {
        let (journal,id,mut fake)=fixture();
        let mut record=journal.load(&id).unwrap(); record.phase=Phase::Staged; journal.save(&record).unwrap();
        fake.session_id=Some(record.session_id.clone()); fake.activated=true;
        fake.state["revision"]=json!("b".repeat(64));
        fake.state["operation"]=json!({"request_id":record.id,"session_id":record.session_id,"revision":"b".repeat(64)});
        assert!(cancel(&journal,&mut fake,&id).await.is_err()); assert_eq!(fake.discarded,0);
    }
    #[tokio::test]
    async fn unavailable_result_keeps_uncertain_intent_and_later_reconciles_once() {
        let (journal,id,mut fake)=fixture(); fake.disconnect=Some("result");
        assert!(apply(&journal,&mut fake,&id,0).await.is_err());
        assert_eq!(journal.load(&id).unwrap().phase,Phase::Activating);
        assert!(fake.activated);
        assert_eq!(apply(&journal,&mut fake,&id,0).await.unwrap().phase,Phase::Verified);
        assert_eq!(fake.activation_calls,1);
    }

    #[tokio::test]
    async fn manual_change_after_commit_requires_review_without_reactivation() {
        let (journal,id,mut fake)=fixture(); fake.disconnect=Some("result");
        assert!(apply(&journal,&mut fake,&id,0).await.is_err());
        fake.state["revision"]=json!("c".repeat(64));
        fake.state["overrides"]["wallpaper"]=json!(false);
        assert_eq!(apply(&journal,&mut fake,&id,0).await.unwrap().phase,Phase::NeedsReview);
        assert_eq!(fake.activation_calls,1);
        assert!(journal.load(&id).unwrap().inverse_patch(fake.state["overrides"].as_object().unwrap()).is_err());
    }

    #[test]
    fn remove_verification_checks_ownership_while_preserving_inherited_value() {
        let (journal,id,_)=fixture(); let mut record=journal.load(&id).unwrap();
        record.patch.set.clear(); record.patch.remove=vec!["wallpaper".into()];
        let state=json!({"revision":"b".repeat(64),"settings":{"wallpaper":true},"overrides":{},"collections":[],
            "operation":{"request_id":record.id,"session_id":record.session_id,"revision":"b".repeat(64)}});
        assert!(verify_applied(&record,&state,&"b".repeat(64)).is_ok());
    }

    struct TransferFake {
        base: Fake,
        incoming: Vec<Value>,
        missing: BTreeSet<String>,
        uploaded: Vec<String>,
        fail_hash_once: Option<String>,
        full: bool,
        unsolicited: bool,
    }
    impl Backend for TransferFake {
        async fn snapshot(&mut self) -> Result<DeviceSnapshot,String> { self.base.snapshot().await }
        async fn stage(&mut self, session: &str, revision: &str, patch: &Value, collections: &[Value]) -> Result<(),String> {
            if self.full { return Err("storage_full".into()); }
            self.base.stage(session,revision,patch,collections).await?;
            self.incoming=collections.to_vec(); Ok(())
        }
        async fn validate(&mut self,_: &str) -> Result<Validation,String> {
            let mut missing:Vec<_>=self.missing.iter().cloned().collect();
            if self.unsolicited { missing.push("f".repeat(64)); }
            Ok(Validation {ready:missing.is_empty(),missing_assets:missing,revision:self.base.state["revision"].as_str().unwrap().into()})
        }
        async fn upload(&mut self,_: &str,hash: &str,_: u64) -> Result<(),String> {
            if self.fail_hash_once.as_deref()==Some(hash) { self.fail_hash_once=None; return Err("injected transfer disconnect".into()); }
            self.missing.remove(hash); self.uploaded.push(hash.to_owned()); Ok(())
        }
        async fn activate(&mut self,op: &str,session: &str,revision: &str) -> Result<String,String> {
            let result=self.base.activate(op,session,revision).await;
            if self.base.activated { self.base.state["collections"]=json!(self.incoming); }
            result
        }
        async fn result(&mut self,op: &str) -> Result<ActivationResult,String> { self.base.result(op).await }
        async fn discard(&mut self,session: &str) -> Result<(),String> { self.base.discard(session).await }
    }
    fn transfer_fixture() -> (Journal,String,TransferFake) {
        let (journal,id,base)=fixture();
        let mut record=journal.load(&id).unwrap();
        record.collections=vec![json!({"id":"c".repeat(32),"order":"sequential","interval_minutes":15,"assets":[
            {"sha256":"a".repeat(64),"bytes":100,"mime":"image/png"},
            {"sha256":"b".repeat(64),"bytes":100,"mime":"image/png"}]})];
        journal.save(&record).unwrap();
        let fake=TransferFake{base,incoming:vec![],missing:BTreeSet::from(["a".repeat(64),"b".repeat(64)]),uploaded:vec![],fail_hash_once:None,full:false,unsolicited:false};
        (journal,id,fake)
    }
    #[tokio::test]
    async fn interrupted_transfer_resumes_only_missing_verified_assets() {
        let (journal,id,mut fake)=transfer_fixture(); fake.fail_hash_once=Some("b".repeat(64));
        let previous=fake.base.state.clone();
        assert!(apply(&journal,&mut fake,&id,0).await.is_err());
        assert_eq!(fake.base.state,previous); assert_eq!(fake.base.activation_calls,0);
        assert_eq!(journal.load(&id).unwrap().phase,Phase::Staged);
        assert_eq!(apply(&journal,&mut fake,&id,0).await.unwrap().phase,Phase::Verified);
        assert_eq!(fake.uploaded,vec!["a".repeat(64),"b".repeat(64)]);
        assert_eq!(fake.base.activation_calls,1);
    }
    #[tokio::test]
    async fn insufficient_storage_and_cancelled_transfer_preserve_active_collection() {
        let (journal,id,mut fake)=transfer_fixture(); let previous=fake.base.state.clone(); fake.full=true;
        assert!(apply(&journal,&mut fake,&id,0).await.is_err());
        assert_eq!(fake.base.state,previous); assert!(fake.uploaded.is_empty());
        assert_eq!(journal.load(&id).unwrap().phase,Phase::Prepared);
        fake.full=false;fake.fail_hash_once=Some("b".repeat(64));
        assert!(apply(&journal,&mut fake,&id,0).await.is_err());
        assert_eq!(cancel(&journal,&mut fake,&id).await.unwrap().phase,Phase::Cancelled);
        assert_eq!(fake.base.state,previous); assert_eq!(fake.base.discarded,1);
        assert!(apply(&journal,&mut fake,&id,0).await.is_err());
    }
    #[tokio::test]
    async fn unsolicited_missing_hash_and_unverified_hash_never_activate() {
        let (journal,id,mut fake)=transfer_fixture(); fake.unsolicited=true;
        assert!(apply(&journal,&mut fake,&id,0).await.is_err());
        assert!(fake.uploaded.is_empty());assert_eq!(fake.base.activation_calls,0);
        // A hash mismatch is reported as an upload failure and remains missing, never ready.
        fake.unsolicited=false;fake.fail_hash_once=Some("a".repeat(64));
        assert!(apply(&journal,&mut fake,&id,0).await.is_err());
        assert_eq!(fake.base.activation_calls,0);assert!(fake.missing.contains(&"a".repeat(64)));
    }

}
