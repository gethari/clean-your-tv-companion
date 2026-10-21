//! Native adapter for the reviewed transaction coordinator. No arbitrary paths or shell IPC.
use crate::{journal::{Journal, OperationHistory, OperationSummary, Record, Review}, launcher_protocol, launcher_transport::{self, Method}, transaction::{self, Backend, DeviceSnapshot, Validation, ActivationResult, ResultStatus}};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{io::Read, process::Stdio, time::Duration};
use tauri::{Emitter, Manager};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, process::Command, time::timeout};

pub struct NativeBackend {
    pub app: tauri::AppHandle,
    pub serial: String,
    pub identity: Value,
    pub operation_id: String,
}
impl NativeBackend {
    async fn check(&self) -> Result<(), String> {
        let identity = serde_json::to_value(crate::tv_identity(&self.app, &self.serial).await?).map_err(|_| "Could not verify this TV.")?;
        if identity != self.identity { return Err("The TV or active user changed. Refresh and review again.".into()); }
        Ok(())
    }
    async fn call(&self, method: Method, request: Value) -> Result<Value, String> {
        self.check().await?;
        let user = self.identity["user"].as_str().ok_or("Missing reviewed TV user.")?;
        launcher_transport::call_for_user(&self.app, &self.serial, user, method, request).await
    }
}

/// Read and verify the immutable cache object before any bytes leave the desktop.
fn cached_asset(root: &std::path::Path, hash: &str, expected: u64) -> Result<Vec<u8>, String> {
    if !launcher_protocol::hex_id(hash, 64) || expected == 0 || expected > 20 * 1024 * 1024 { return Err("Invalid image manifest.".into()); }
    let path = root.join(hash);
    let metadata = std::fs::symlink_metadata(&path).map_err(|_| "The imported image is missing. Import it again.")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() != expected { return Err("The imported image changed. Import it again.".into()); }
    let file = std::fs::File::open(path).map_err(|_| "Could not read the imported image.")?;
    let mut bytes = Vec::new();
    file.take(expected + 1).read_to_end(&mut bytes).map_err(|_| "Could not read the imported image.")?;
    let actual: String = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect();
    if bytes.len() as u64 != expected || actual != hash { return Err("The imported image failed verification. Import it again.".into()); }
    Ok(bytes)
}

impl Backend for NativeBackend {
    async fn snapshot(&mut self) -> Result<DeviceSnapshot, String> {
        let caps = self.call(Method::Capabilities, json!({})).await?;
        launcher_protocol::capabilities(caps).map_err(|_| "This launcher does not support recoverable personalization.")?;
        let state = self.call(Method::State, json!({})).await?;
        self.check().await?;
        Ok(DeviceSnapshot { identity: self.identity.clone(), state })
    }
    async fn stage(&mut self, session_id: &str, expected_revision: &str, patch: &Value, collections: &[Value]) -> Result<(), String> {
        let caps = launcher_protocol::capabilities(self.call(Method::Capabilities, json!({})).await?).map_err(|_| "Unsupported launcher.")?;
        let typed = serde_json::from_value(patch.clone()).map_err(|_| "Invalid saved settings.")?;
        launcher_protocol::validate_patch(&typed, &caps).map_err(|_| "Unsupported saved settings.")?;
        launcher_protocol::validate_collections(collections, &caps).map_err(|_| "Unsupported saved collection.")?;
        let result = self.call(Method::Stage, json!({"request_id":session_id,"expected_revision":expected_revision,"patch":patch,"collections":collections})).await?;
        if result["session_id"] != session_id { return Err("The staging response did not match this plan.".into()); }
        Ok(())
    }
    async fn validate(&mut self, session_id: &str) -> Result<Validation, String> {
        let value = self.call(Method::Validate, json!({"session_id":session_id})).await?;
        let hashes = value["missing_assets"].as_array().ok_or("Invalid image verification response.")?;
        if hashes.len() > 200 { return Err("Image verification response exceeds its limit.".into()); }
        let missing_assets = hashes.iter().map(|hash| hash.as_str().filter(|s| launcher_protocol::hex_id(s,64)).map(str::to_owned).ok_or_else(|| "Invalid image verification response.".to_owned())).collect::<Result<Vec<_>,_>>()?;
        Ok(Validation { ready: value["ready"].as_bool().ok_or("Missing verification status.")?, missing_assets, revision: value["revision"].as_str().ok_or("Missing verification revision.")?.into() })
    }
    async fn upload(&mut self, session_id: &str, hash: &str, expected: u64) -> Result<(), String> {
        self.check().await?;
        if !launcher_protocol::hex_id(session_id,32) { return Err("Invalid transfer session.".into()); }
        let root = self.app.path().app_data_dir().map_err(|_| "Could not locate imported media.")?.join("media");
        let bytes = cached_asset(&root, hash, expected)?;
        let user = self.identity["user"].as_str().filter(|s| !s.is_empty() && s.len()<=5 && s.bytes().all(|b| b.is_ascii_digit())).ok_or("Invalid TV user.")?;
        let path = crate::adb_path(&self.app)?;
        crate::ensure_server(&path).await?;
        let uri = format!("{}/sessions/{session_id}/assets/{hash}", launcher_transport::URI);
        let mut command = Command::new(path);
        command.args(["-s", &self.serial, "shell", "-T", "content", "write", "--user", user, "--uri", &uri]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
        #[cfg(windows)] command.creation_flags(0x08000000);
        let mut child = command.spawn().map_err(|_| "Could not start image transfer.")?;
        let mut stdin = child.stdin.take().ok_or("Image transfer input unavailable.")?;
        let stdout = child.stdout.take().ok_or("Image transfer output unavailable.")?;
        let stderr = child.stderr.take().ok_or("Image transfer output unavailable.")?;
        timeout(Duration::from_secs(65), async {
            let write = async {
                let mut sent = 0u64;
                for chunk in bytes.chunks(65536) {
                    stdin.write_all(chunk).await.map_err(|_| "Image transfer interrupted. Reconnect to resume.")?;
                    sent += chunk.len() as u64;
                    let _ = self.app.emit("personalization-progress", json!({"operation_id":self.operation_id,"phase":"transferring","asset":hash,"bytes_sent":sent,"total_bytes":expected}));
                }
                drop(stdin);
                Ok::<_, String>(())
            };
            let read = async {
                let mut out=Vec::new(); let mut err=Vec::new();
                let mut stdout=stdout.take(65537); let mut stderr=stderr.take(65537);
                let (a,b)=tokio::join!(stdout.read_to_end(&mut out),stderr.read_to_end(&mut err));
                a.map_err(|_| "Image transfer interrupted.")?; b.map_err(|_| "Image transfer interrupted.")?;
                if out.len()>65536 || err.len()>65536 { return Err("Image transfer response exceeded its limit.".to_owned()); }
                Ok(())
            };
            let (a,b)=tokio::join!(write,read); a?; b?;
            if !child.wait().await.map_err(|_| "Image transfer interrupted.")?.success() { return Err("The TV rejected the image transfer. Reconnect to resume.".into()); }
            // Confirm asynchronous receiver verification before allowing activation.
            for _ in 0..20 {
                let value=self.call(Method::Validate,json!({"session_id":session_id})).await?;
                let missing=value["missing_assets"].as_array().ok_or("Invalid verification response.")?;
                if !missing.iter().any(|item| item.as_str()==Some(hash)) { return Ok(()); }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Err("The TV has not verified this image. Reconnect to resume.".into())
        }).await.map_err(|_| "Image transfer timed out. Reconnect to resume.")?
    }
    async fn activate(&mut self, operation_id: &str, session_id: &str, expected_revision: &str) -> Result<String, String> {
        let value=self.call(Method::Activate,json!({"request_id":operation_id,"session_id":session_id,"expected_revision":expected_revision})).await?;
        if value["operation_id"] != operation_id { return Err("Activation acknowledgment did not match this operation.".into()); }
        value["revision"].as_str().map(str::to_owned).ok_or_else(|| "Activation revision missing.".into())
    }
    async fn result(&mut self, operation_id: &str) -> Result<ActivationResult, String> {
        let value=self.call(Method::Result,json!({"operation_id":operation_id})).await?;
        let status=match value["status"].as_str() { Some("activated")=>ResultStatus::Activated,Some("unknown")=>ResultStatus::Unknown,_=>return Err("Unknown operation status. Refresh before continuing.".into()) };
        Ok(ActivationResult {status,session_id:value["session_id"].as_str().map(str::to_owned),revision:value["revision"].as_str().map(str::to_owned)})
    }
    async fn discard(&mut self, session_id: &str) -> Result<(), String> { self.call(Method::Discard,json!({"session_id":session_id})).await?; Ok(()) }
}

fn journal(app: &tauri::AppHandle) -> Result<Journal,String> {
    Journal::open(app.path().app_data_dir().map_err(|_| "Could not locate change journal.")?.join("operations"))
}

#[tauri::command]
pub async fn list_personalization_history(
    app: tauri::AppHandle,
    serial: String,
) -> Result<OperationHistory, String> {
    let identity = serde_json::to_value(crate::tv_identity(&app, &serial).await?)
        .map_err(|_| "Could not verify this TV.")?;
    let history = journal(&app)?.history(&identity)?;
    let verified = serde_json::to_value(crate::tv_identity(&app, &serial).await?)
        .map_err(|_| "Could not verify this TV.")?;
    if verified != identity {
        return Err("The TV or active user changed. Refresh before opening operation history.".into());
    }
    Ok(history)
}

/// Read actual operation result and TV state only. This command never reactivates an operation.
#[tauri::command]
pub async fn reconcile_personalization(
    app: tauri::AppHandle,
    serial: String,
    id: String,
) -> Result<OperationSummary, String> {
    let journal = journal(&app)?;
    let record = journal.load(&id)?;
    let identity = serde_json::to_value(crate::tv_identity(&app, &serial).await?)
        .map_err(|_| "Could not verify this TV.")?;
    if identity != record.identity {
        return Err("This operation belongs to a different TV or user. Select its verified TV to continue.".into());
    }
    let mut backend = NativeBackend { app: app.clone(), serial: serial.clone(), identity: record.identity.clone(), operation_id: id.clone() };
    let reconciled = transaction::reconcile(&journal, &mut backend, &id).await?;
    let verified = serde_json::to_value(crate::tv_identity(&app, &serial).await?)
        .map_err(|_| "Could not verify this TV.")?;
    if verified != record.identity {
        return Err("The TV or active user changed while reconciling. Refresh before continuing.".into());
    }
    Ok(reconciled.safe_summary())
}

#[tauri::command]
pub async fn personalization_state(app: tauri::AppHandle, serial: String) -> Result<Value,String> {
    let identity=crate::tv_identity(&app,&serial).await?;
    let user=identity.user.clone();
    let caps=launcher_transport::call_for_user(&app,&serial,&user,Method::Capabilities,json!({})).await;
    let supported=match caps {
        Ok(value)=>launcher_protocol::capabilities(value).is_ok(),
        Err(_)=>false,
    };
    if !supported { return Ok(json!({"editable":false,"reason":"This launcher supports inspection only. Install a compatible companion-enabled launcher before editing backgrounds."})); }
    let state=launcher_transport::call_for_user(&app,&serial,&user,Method::State,json!({})).await?;
    if crate::tv_identity(&app,&serial).await? != identity { return Err("The TV or user changed. Refresh before editing.".into()); }
    Ok(json!({"editable":true,"state":state}))
}
#[tauri::command]
pub async fn apply_personalization(app: tauri::AppHandle, serial: String, id: String) -> Result<Record,String> {
    let journal=journal(&app)?; let record=journal.load(&id)?;
    let mut backend=NativeBackend { app,serial,identity:record.identity,operation_id:id.clone() };
    let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    transaction::apply(&journal,&mut backend,&id,now).await
}
#[tauri::command]
pub async fn cancel_personalization(app: tauri::AppHandle, serial: String, id: String) -> Result<Record,String> {
    let journal=journal(&app)?; let record=journal.load(&id)?;
    let mut backend=NativeBackend { app,serial,identity:record.identity,operation_id:id.clone() };
    transaction::cancel(&journal,&mut backend,&id).await
}
#[tauri::command]
pub async fn prepare_restore(app: tauri::AppHandle, serial: String, id: String) -> Result<Review,String> {
    let journal=journal(&app)?; let record=journal.load(&id)?; let _lock=journal.lock(&record.identity)?;
    let mut backend=NativeBackend { app,serial,identity:record.identity.clone(),operation_id:id };
    let snapshot=backend.snapshot().await?;
    journal.prepare_restore(&record,snapshot.identity,&snapshot.state)
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn cache_rejects_wrong_hash_size_and_path_before_transfer() {
        let path=std::env::temp_dir().join(format!("cytv-cache-{}",launcher_transport::new_id().unwrap()));
        std::fs::create_dir(&path).unwrap();
        let bytes=b"fixture"; let hash:String=Sha256::digest(bytes).iter().map(|b|format!("{b:02x}")).collect();
        std::fs::write(path.join(&hash),bytes).unwrap();
        assert_eq!(cached_asset(&path,&hash,7).unwrap(),bytes);
        assert!(cached_asset(&path,&hash,8).is_err());
        assert!(cached_asset(&path,"../outside",7).is_err());
        std::fs::write(path.join(&hash),b"changed").unwrap();
        assert!(cached_asset(&path,&hash,7).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }
}
