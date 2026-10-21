use serde::{Deserialize, Serialize};
pub mod launcher_protocol;
pub mod launcher_transport;
pub mod journal;
pub mod transaction;
pub mod cleanup_policy;
pub mod setup_operations;
mod personalization;
mod media;
use serde_json::{json, Value};
use std::{net::Ipv4Addr, path::PathBuf, process::Stdio, time::{Duration, SystemTime, UNIX_EPOCH}};
use tauri::Manager;
use tokio::{io::{AsyncWriteExt, AsyncReadExt}, process::Command, time::timeout};

fn endpoint_valid(value: &str) -> bool {
    let Some((host, port)) = value.rsplit_once(':') else { return false };
    let Ok(ip) = host.parse::<Ipv4Addr>() else { return false };
    let Ok(port) = port.parse::<u16>() else { return false };
    port > 0 && (ip.is_private() || ip.is_link_local()) && !host.starts_with('0')
}
fn serial_valid(value: &str) -> bool {
    endpoint_valid(value) || (!value.is_empty() && value.len() < 160 && !value.starts_with('-')
        && value.chars().all(|c| c.is_ascii_alphanumeric() || "-._".contains(c)))
}
fn adb_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if cfg!(debug_assertions) {
        if let Some(path) = std::env::var_os("CYTV_ADB_PATH") { return Ok(PathBuf::from(path)); }
    }
    let filename = if cfg!(windows) { "adb.exe" } else { "adb" };
    let path = app.path().resource_dir().map_err(|_| "Could not locate app resources")?
        .join("resources").join("platform-tools").join(filename);
    if path.is_file() { Ok(path) } else {
        Err("Bundled Android tools are missing. Use a complete desktop installer; developers can set CYTV_ADB_PATH.".into())
    }
}
// Start the daemon without captured pipes: on Windows a newly spawned server can
// inherit client output handles and keep a pairing command's pipes open.
async fn ensure_server(path: &std::path::Path) -> Result<(), String> {
    let mut command = Command::new(path);
    command.arg("start-server").stdin(Stdio::null()).stdout(Stdio::null())
        .stderr(Stdio::null()).kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let mut child = command.spawn().map_err(|_| "Could not start Android tools.")?;
    let status = timeout(Duration::from_secs(12), child.wait()).await
        .map_err(|_| "Android tools took too long to start. Close other Android tools and try again.")?
        .map_err(|_| "Could not start Android tools.")?;
    if status.success() { Ok(()) } else { Err("Android tools could not start. Check OS permissions and try again.".into()) }
}

async fn adb(app: &tauri::AppHandle, args: &[&str], input: Option<&str>) -> Result<String, String> {
    let path = adb_path(app)?;
    ensure_server(&path).await?;
    let mut command = Command::new(path);
    command.args(args).stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let mut child = command.spawn().map_err(|_| "Could not start Android tools. Check the installation and OS permissions.")?;
    if let Some(text) = input {
        let mut stdin = child.stdin.take().ok_or("Pairing input is unavailable")?;
        stdin.write_all(text.as_bytes()).await.map_err(|_| "Could not send pairing code")?;
        drop(stdin);
    }
    let stdout = child.stdout.take().ok_or("TV output unavailable")?;
    let stderr = child.stderr.take().ok_or("TV output unavailable")?;
    let result = timeout(Duration::from_secs(18), async {
        let mut out = vec![];
        let mut err = vec![];
        let mut stdout = stdout.take(1024 * 1024 + 1);
        let mut stderr = stderr.take(64 * 1024 + 1);
        let (a, b) = tokio::join!(stdout.read_to_end(&mut out), stderr.read_to_end(&mut err));
        a.map_err(|_| "TV response interrupted")?;
        b.map_err(|_| "TV response interrupted")?;
        if out.len() > 1024 * 1024 || err.len() > 64 * 1024 {
            return Err("TV response exceeded the inspection limit.");
        }
        let status = child.wait().await.map_err(|_| "The connection was interrupted.")?;
        if !status.success() { return Err("The TV could not complete this check. Accept authorization on the TV, keep it awake, and verify its address."); }
        Ok(String::from_utf8_lossy(&out).trim().to_owned())
    }).await.map_err(|_| "The TV didn’t reply in time. Keep it awake and check its current debugging port.")?;
    result.map_err(str::to_owned)
}

#[derive(Serialize)]
struct Device { endpoint: String, status: String, kind: String }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TvIdentity { serial: String, manufacturer: String, model: String, fingerprint: String, user: String }

fn identity_value(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
        && !matches!(value.to_ascii_lowercase().as_str(), "unknown" | "null" | "none" | "0123456789abcdef")
}
async fn tv_identity(app: &tauri::AppHandle, serial: &str) -> Result<TvIdentity, String> {
    if !serial_valid(serial) { return Err("Choose a valid discovered device.".into()); }
    // One bounded, read-only probe; do not store an endpoint as device identity.
    let output = adb(app, &["-s", serial, "shell", "getprop ro.serialno; getprop ro.product.manufacturer; getprop ro.product.model; getprop ro.build.fingerprint; am get-current-user; pm list features"], None).await?;
    parse_identity(&output).ok_or_else(|| "This TV did not provide a reliable identity. Choose it again next time.".into())
}
fn parse_identity(output: &str) -> Option<TvIdentity> {
    let lines: Vec<_> = output.lines().map(str::trim).collect();
    if lines.len() < 6 || !lines[5..].contains(&"feature:android.software.leanback") { return None; }
    if !lines[..4].iter().all(|v| identity_value(v)) || lines[4].is_empty()
        || !lines[4].bytes().all(|c| c.is_ascii_digit()) { return None; }
    Some(TvIdentity { serial: lines[0].into(), manufacturer: lines[1].into(), model: lines[2].into(), fingerprint: lines[3].into(), user: lines[4].into() })
}
fn remembered_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(|_| "Could not locate local app preferences.")?.join("remembered-tv.json"))
}
#[tauri::command]
async fn remember_device(app: tauri::AppHandle, serial: String) -> Result<(), String> {
    let path = remembered_path(&app)?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|_| "Could not save the selected TV locally.")?;
    // A new explicit selection must not leave an older TV remembered if this probe fails.
    match std::fs::remove_file(&path) {
        Ok(()) => {},
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
        Err(_) => return Err("Could not replace the selected TV preference.".into()),
    }
    let identity = tv_identity(&app, &serial).await?;
    // Remove stale preference before replacement; an interrupted write fails closed on next load.
    let bytes = serde_json::to_vec(&identity).map_err(|_| "Could not save the selected TV.")?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
    use std::io::Write;
    let mut file = options.open(path).map_err(|_| "Could not save the selected TV locally.")?;
    file.write_all(&bytes).map_err(|_| "Could not save the selected TV locally.".to_owned())
}
fn unique_remembered(matches: Vec<String>, uncertain: bool) -> Option<String> {
    if !uncertain && matches.len() == 1 { matches.into_iter().next() } else { None }
}
#[tauri::command]
async fn remembered_device(app: tauri::AppHandle, endpoints: Vec<String>) -> Result<Option<String>, String> {
    let path = remembered_path(&app)?;
    let Ok(bytes) = std::fs::read(path) else { return Ok(None) };
    if bytes.len() > 4096 { return Ok(None); }
    let Ok(remembered) = serde_json::from_slice::<TvIdentity>(&bytes) else { return Ok(None) };
    let mut unique = endpoints;
    unique.sort(); unique.dedup();
    if unique.len() > 8 || unique.iter().any(|e| !serial_valid(e)) { return Ok(None); }
    let mut matches = vec![];
    let mut uncertain = false;
    for endpoint in unique {
        let probe = timeout(Duration::from_secs(5), async {
            if endpoint_valid(&endpoint) { connect_device(app.clone(), endpoint.clone()).await?; }
            tv_identity(&app, &endpoint).await
        }).await;
        match probe {
            Ok(Ok(identity)) if identity == remembered => matches.push(endpoint),
            Ok(Ok(_)) => {},
            _ => uncertain = true,
        }
    }
    // An unreadable candidate could be a duplicate identity. Ask rather than guessing.
    Ok(unique_remembered(matches, uncertain))
}
#[tauri::command]
async fn discover(app: tauri::AppHandle) -> Result<Vec<Device>, String> {
    let devices = adb(&app, &["devices"], None).await?;
    let services = adb(&app, &["mdns", "services"], None).await.unwrap_or_default();
    Ok(parse_discovery(&devices, &services))
}
fn parse_discovery(devices: &str, services: &str) -> Vec<Device> {
    let mut result = vec![];
    let advertised: Vec<Vec<&str>> = services.lines().skip(1).map(|line| line.split_whitespace().collect())
        .filter(|fields: &Vec<&str>| fields.len() >= 3 && endpoint_valid(fields[fields.len() - 1])).collect();
    for line in devices.lines().skip(1) {
        let mut fields = line.split_whitespace();
        if let (Some(serial), Some(status)) = (fields.next(), fields.next()) {
            let canonical = advertised.iter().find_map(|service| {
                let alias = format!("{}.{}", service[0], service[1]);
                if serial == alias || serial == format!("{alias}.") { service.last().copied() } else { None }
            }).unwrap_or(serial);
            if serial_valid(canonical) && !result.iter().any(|d: &Device| d.endpoint == canonical) {
                result.push(Device { endpoint: canonical.into(), status: status.into(), kind: "Connected device".into() });
            }
        }
    }
    // ADB's mDNS service list may be unavailable; manual connection remains usable.
        for fields in advertised {
            if let Some(address) = fields.last() {
                if endpoint_valid(address) && !result.iter().any(|d| d.endpoint == *address) {
                    let pairing = fields[1].contains("_adb-tls-pairing");
                    result.push(Device { endpoint: (*address).into(), status: "Advertised · not yet inspected".into(),
                        kind: if pairing { "Pairing service".into() } else if fields[1].contains("_adb-tls-connect") { "Connection service".into() } else { "Legacy connection service".into() } });
                }
            }
        }
    result
}
#[tauri::command]
async fn connect_device(app: tauri::AppHandle, endpoint: String) -> Result<(), String> {
    if !endpoint_valid(&endpoint) { return Err("Use a private IPv4 address and a valid connection port.".into()); }
    let response = adb(&app, &["connect", &endpoint], None).await?;
    if response.contains("connected to") { Ok(()) } else {
        Err("Could not connect. Confirm the connection port, approve this computer on the TV, or pair first.".into())
    }
}
#[tauri::command]
async fn pair_device(app: tauri::AppHandle, endpoint: String, code: String) -> Result<(), String> {
    if !endpoint_valid(&endpoint) || code.len() != 6 || !code.bytes().all(|c| c.is_ascii_digit()) {
        return Err("Enter the pairing address and six-digit code shown on the TV.".into());
    }
    let response = adb(&app, &["pair", &endpoint], Some(&(code + "\n"))).await?;
    if response.contains("Successfully paired") { Ok(()) } else { Err("Pairing did not complete. Open a fresh pairing code on the TV and try again.".into()) }
}
async fn query(app: &tauri::AppHandle, serial: &str, command: &[&str], label: &str, errors: &mut Vec<String>) -> Option<String> {
    let mut args = vec!["-s", serial, "shell"];
    args.extend_from_slice(command);
    match adb(app, &args, None).await {
        Ok(value) if !value.starts_with("Error") && !value.starts_with("Exception")
                && !value.starts_with("Permission Denial") && !value.starts_with("java.") => Some(value),
        Ok(_) => { errors.push(format!("The TV denied access to {label}. This state is unknown.")); None },
        Err(_) => { errors.push(format!("Couldn’t check {label}. Reconnect the TV and refresh.")); None }
    }
}
fn packages(value: &str) -> Option<Vec<String>> {
    let mut result = vec![];
    for line in value.lines().filter(|line| !line.trim().is_empty()) {
        let name = line.trim().strip_prefix("package:")?;
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || "._".contains(c)) { return None; }
        result.push(name.to_owned());
    }
    Some(result)
}
fn safe_theme(value: &str) -> Option<Value> {
    let source: Value = serde_json::from_str(value).ok()?;
    let mut clean = serde_json::Map::new();
    for name in ["wallpaper", "wallpaper_rotation_minutes", "wallpaper_dim", "columns", "reduced_motion", "dock_apps"] {
        if let Some(value) = source.get(name) { clean.insert(name.into(), value.clone()); }
    }
    Some(Value::Object(clean))
}
#[tauri::command]
async fn prepare_personalization(
    app: tauri::AppHandle,
    serial: String,
    patch: launcher_protocol::SettingsPatch,
    collections: Vec<Value>,
) -> Result<journal::Review, String> {
    let identity = tv_identity(&app, &serial).await?;
    let identity_value = serde_json::to_value(&identity).map_err(|_| "Could not identify this TV.")?;
    let root = app.path().app_data_dir().map_err(|_| "Could not locate the local journal.")?.join("operations");
    let journal = journal::Journal::open(root)?;
    let _guard = journal.lock(&identity_value)?;
    let caps = launcher_transport::call_for_user(&app, &serial, &identity.user, launcher_transport::Method::Capabilities, json!({})).await?;
    let caps = launcher_protocol::capabilities(caps).map_err(|_| "This launcher does not support recoverable personalization.")?;
    launcher_protocol::validate_patch(&patch, &caps).map_err(|_| "These settings are not supported by this launcher.")?;
    launcher_protocol::validate_collections(&collections, &caps).map_err(|_| "This collection exceeds the supported format or storage limits.")?;
    let state = launcher_transport::call_for_user(&app, &serial, &identity.user, launcher_transport::Method::State, json!({})).await?;
    if tv_identity(&app, &serial).await? != identity {
        return Err("The TV or active user changed during review. Refresh and try again.".into());
    }
    journal.prepare(identity_value, &state, patch, collections)
}
#[tauri::command]
async fn inspect_device(app: tauri::AppHandle, serial: String) -> Result<Value, String> {
    if !serial_valid(&serial) { return Err("Choose a discovered device or a valid local address.".into()); }
    let mut errors = vec![];
    let model = query(&app, &serial, &["getprop", "ro.product.model"], "TV model", &mut errors).await;
    if model.is_none() { return Err("Your TV is offline or not authorized. Keep it awake, approve this computer, and check the current debugging port.".into()); }
    let manufacturer = query(&app, &serial, &["getprop", "ro.product.manufacturer"], "manufacturer", &mut errors).await;
    let android = query(&app, &serial, &["getprop", "ro.build.version.release"], "Android version", &mut errors).await;
    let features = query(&app, &serial, &["pm", "list", "features"], "TV compatibility", &mut errors).await;
    let user = query(&app, &serial, &["am", "get-current-user"], "active TV user", &mut errors).await
        .filter(|s| !s.is_empty() && s.len() < 6 && s.bytes().all(|c| c.is_ascii_digit()));
    let (all, disabled) = if let Some(user) = &user {
        (query(&app, &serial, &["pm", "list", "packages", "--user", user], "installed apps", &mut errors).await.and_then(|s| packages(&s)),
         query(&app, &serial, &["pm", "list", "packages", "-d", "--user", user], "disabled apps", &mut errors).await.and_then(|s| packages(&s)))
    } else { (None, None) };
    let launcher_version = if all.as_ref().is_some_and(|v| v.iter().any(|p| p == "com.gathin.tvhome")) {
        query(&app, &serial, &["dumpsys", "package", "com.gathin.tvhome"], "launcher version", &mut errors).await
            .and_then(|s| s.lines().find_map(|l| l.trim().strip_prefix("versionName=").map(str::to_owned)))
    } else { None };
    let home = query(&app, &serial, &["cmd", "package", "resolve-activity", "--brief", "-a", "android.intent.action.MAIN", "-c", "android.intent.category.HOME"], "default home screen", &mut errors).await;
    let home = home.and_then(|s| s.lines().rev().find(|l| l.contains('/') && !l.contains(' ')).map(str::to_owned));
    let theme = query(&app, &serial, &["cat", "/sdcard/Android/data/com.gathin.tvhome/files/config.json"], "theme preferences", &mut errors).await.and_then(|s| safe_theme(&s));
    let collected = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    Ok(json!({"model":model,"manufacturer":manufacturer,"androidVersion":android,"home":home,
        "isTv":features.map(|s| s.contains("android.software.leanback")),
        "packages":all,"disabled":disabled,"launcherVersion":launcher_version,"user":user,
        "theme":theme,"errors":errors,"collectedAt":format!("Checked at Unix {collected}")}))
}
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                let paths = paths.clone();
                let window = window.clone();
                if let Ok(root) = window.app_handle().path().app_data_dir() {
                    tauri::async_runtime::spawn_blocking(move || {
                        use tauri::Emitter;
                        let batch = media::ingest_native_paths(&root, &paths);
                        let _ = window.emit("media-imported", batch);
                    });
                }
            }
        })
        .invoke_handler(tauri::generate_handler![discover, connect_device, pair_device, inspect_device, remember_device, remembered_device, prepare_personalization, personalization::list_personalization_history, personalization::reconcile_personalization, personalization::personalization_state, personalization::apply_personalization, personalization::cancel_personalization, personalization::prepare_restore, media::pick_photo_files, media::pick_photo_folder, media::transform_image])
        .run(tauri::generate_context!()).expect("Could not start Clean Your TV");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn discovery_collapses_mdns_alias_and_ip_for_the_same_endpoint() {
        let devices = "List of devices attached\n192.168.1.20:43210 device\nadb-example._adb-tls-connect._tcp device\n";
        let services = "List of discovered mdns services\nadb-example _adb-tls-connect._tcp 192.168.1.20:43210\nadb-pair _adb-tls-pairing._tcp 192.168.1.20:45678\n";
        let found = parse_discovery(devices, services);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].endpoint, "192.168.1.20:43210");
        assert_eq!(found[1].kind, "Pairing service");
        assert_eq!(parse_discovery(devices, "").len(), 2);
    }
    #[test] fn identity_requires_tv_features_and_complete_values() {
        let valid = "SERIAL123\nExample\nTV\nexample/build/version\n0\nfeature:android.software.leanback\n";
        assert!(parse_identity(valid).is_some());
        assert!(parse_identity(&valid.replace("SERIAL123", "unknown")).is_none());
        assert!(parse_identity(&valid.replace("SERIAL123", "")).is_none());
        assert!(parse_identity(&valid.replace("feature:android.software.leanback", "feature:android.hardware.telephony")).is_none());
        assert_ne!(parse_identity(valid), parse_identity(&valid.replace("SERIAL123", "SERIAL456")));
    }
    #[test] fn reconnect_requires_one_certain_identity_match() {
        assert_eq!(unique_remembered(vec!["192.168.1.2:43210".into()], false), Some("192.168.1.2:43210".into()));
        assert_eq!(unique_remembered(vec!["one".into(), "two".into()], false), None);
        assert_eq!(unique_remembered(vec!["one".into()], true), None);
        assert_eq!(unique_remembered(vec![], true), None);
    }
    #[test] fn endpoint_rejects_shell_and_public_hosts() {
        for invalid in ["8.8.8.8:5555", "192.168.1.2:0", "192.168.1.2:65536", "192.168.1.2:5555;id", "--help", "localhost:5555", "[::1]:5555"] { assert!(!endpoint_valid(invalid)); }
        for valid in ["192.168.1.2:46481", "10.0.0.2:5555", "172.16.0.2:1234"] { assert!(endpoint_valid(valid)); }
    }
    #[test] fn sanitizes_theme_credentials() {
        assert_eq!(safe_theme(r#"{"wallpaper":true,"access_key":"private","unknown":"x"}"#), Some(json!({"wallpaper":true})));
        assert_eq!(safe_theme("invalid"), None);
    }
    #[test] fn parses_package_output() {
        assert_eq!(packages("package:com.example.app\npackage:org.example.tv\n"), Some(vec!["com.example.app".into(), "org.example.tv".into()]));
        assert_eq!(packages("Error: permission denied"), None);
        assert_eq!(packages(""), Some(vec![]));
    }
}
