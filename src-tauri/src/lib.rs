use serde::Serialize;
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
async fn adb(app: &tauri::AppHandle, args: &[&str], input: Option<&str>) -> Result<String, String> {
    let path = adb_path(app)?;
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
#[tauri::command]
async fn discover(app: tauri::AppHandle) -> Result<Vec<Device>, String> {
    let devices = adb(&app, &["devices"], None).await?;
    let mut result = vec![];
    for line in devices.lines().skip(1) {
        let mut fields = line.split_whitespace();
        if let (Some(serial), Some(status)) = (fields.next(), fields.next()) {
            if serial_valid(serial) { result.push(Device { endpoint: serial.into(), status: status.into(), kind: "Connected device".into() }); }
        }
    }
    // ADB's mDNS service list may be unavailable; manual connection remains usable.
    if let Ok(services) = adb(&app, &["mdns", "services"], None).await {
        for line in services.lines().skip(1) {
            let fields: Vec<_> = line.split_whitespace().collect();
            if let Some(address) = fields.last() {
                if endpoint_valid(address) && !result.iter().any(|d| d.endpoint == *address) {
                    let pairing = line.contains("_adb-tls-pairing");
                    result.push(Device { endpoint: (*address).into(), status: "Advertised · not yet inspected".into(),
                        kind: if pairing { "Pairing service".into() } else { "Connection service".into() } });
                }
            }
        }
    }
    Ok(result)
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
    tauri::Builder::default().invoke_handler(tauri::generate_handler![discover, connect_device, pair_device, inspect_device])
        .run(tauri::generate_context!()).expect("Could not start Clean Your TV");
}
#[cfg(test)]
mod tests {
    use super::*;
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
