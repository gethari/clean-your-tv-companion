use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::{json, Value};

pub const URI: &str = "content://com.gathin.tvhome.companion/v1";

pub fn new_id() -> Result<String, String> {
    let mut bytes = [0; 16];
    getrandom::fill(&mut bytes).map_err(|_| "Could not create a secure operation identifier.")?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub enum Method {
    Capabilities,
    State,
    Stage,
    Validate,
    Activate,
    Result,
    Discard,
}

impl Method {
    fn name(&self) -> &'static str {
        match self {
            Self::Capabilities => "capabilities",
            Self::State => "state",
            Self::Stage => "stage",
            Self::Validate => "validate",
            Self::Activate => "activate",
            Self::Result => "result",
            Self::Discard => "discard",
        }
    }
}

pub fn decode_reply(output: &str, request_id: &str) -> Result<Value, String> {
    let encoded = output
        .split("reply=")
        .nth(1)
        .map(|value| {
            value
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
                .collect::<String>()
        })
        .ok_or("Open Clean Your TV on the TV once, then refresh. This launcher may not support companion editing.")?;
    if encoded.len() > 349526 {
        return Err("The launcher response exceeded the protocol limit.".into());
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| "The launcher sent an invalid response.")?;
    let reply: Value = serde_json::from_slice(&bytes)
        .map_err(|_| "The launcher sent an invalid response.")?;
    if bytes.len() > 262144
        || reply["protocol_version"] != 1
        || reply["request_id"].as_str() != Some(request_id)
    {
        return Err("The launcher response did not match this operation.".into());
    }
    if reply["ok"] != true {
        let message = match reply["error"]["code"].as_str() {
            Some("revision_conflict") => "The TV’s setup changed. Refresh and review your changes again.",
            Some("storage_full") => "The TV needs more free storage before this collection can be applied.",
            Some("busy") => "The TV is handling another operation. Wait for it to finish and retry.",
            Some("missing_assets") => "Some images have not finished transferring. Reconnect to resume.",
            Some("unsupported_protocol" | "unsupported_feature") => "This launcher does not support the requested feature.",
            Some("invalid_asset") => "The TV rejected an image. Check its format and try again.",
            _ => "The launcher could not complete this operation. Refresh the TV before retrying.",
        };
        return Err(message.into());
    }
    reply.get("data").cloned().ok_or_else(|| "The launcher response was incomplete.".into())
}

pub async fn call(
    app: &tauri::AppHandle,
    serial: &str,
    method: Method,
    request: Value,
) -> Result<Value, String> {
    if !crate::serial_valid(serial) { return Err("Choose a valid TV before continuing.".into()); }
    let user = crate::adb(app, &["-s", serial, "shell", "am", "get-current-user"], None).await?;
    call_for_user(app, serial, &user, method, request).await
}

pub async fn call_for_user(
    app: &tauri::AppHandle,
    serial: &str,
    user: &str,
    method: Method,
    mut request: Value,
) -> Result<Value, String> {
    if !crate::serial_valid(serial) {
        return Err("Choose a valid TV before continuing.".into());
    }
    let id = match request.get("request_id") {
        Some(Value::String(id)) if crate::launcher_protocol::hex_id(id, 32) => id.clone(),
        None => new_id()?,
        _ => return Err("Invalid operation identifier.".into()),
    };
    let object = request.as_object_mut().ok_or("Invalid launcher request.")?;
    object.insert("protocol_version".into(), json!(1));
    object.insert("request_id".into(), json!(id));
    let bytes = serde_json::to_vec(&request).map_err(|_| "Could not encode launcher request.")?;
    if bytes.len() > 262144 {
        return Err("This collection exceeds the configuration limit.".into());
    }
    let payload = URL_SAFE_NO_PAD.encode(bytes);
    if user.is_empty() || user.len() > 5 || !user.bytes().all(|c| c.is_ascii_digit()) {
        return Err("Could not determine the active TV user.".into());
    }
    let output = crate::adb(
        app,
        &["-s", serial, "shell", "content", "call", "--user", user, "--uri", URI,
            "--method", method.name(), "--arg", &payload],
        None,
    ).await?;
    decode_reply(&output, &id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(value: Value) -> String {
        format!("Result: Bundle[{{reply={}}}]", URL_SAFE_NO_PAD.encode(serde_json::to_vec(&value).unwrap()))
    }

    #[test]
    fn reply_must_match_the_request_and_never_echo_provider_error_text() {
        let id = "a".repeat(32);
        assert_eq!(decode_reply(&envelope(json!({"protocol_version":1,"request_id":id,"ok":true,"data":{"value":1}})), &id).unwrap(), json!({"value":1}));
        assert!(decode_reply(&envelope(json!({"protocol_version":1,"request_id":"b".repeat(32),"ok":true,"data":{}})), &id).is_err());
        let error = decode_reply(&envelope(json!({"protocol_version":1,"request_id":id,"ok":false,"error":{"code":"internal_error","message":"fixture-secret"}})), &id).unwrap_err();
        assert!(!error.contains("fixture-secret"));
        assert!(decode_reply("Could not find provider", &id).is_err());
    }

    #[test]
    fn identifiers_are_random_and_shell_safe() {
        let first = new_id().unwrap();
        assert!(crate::launcher_protocol::hex_id(&first, 32));
        assert_ne!(first, new_id().unwrap());
    }
}
