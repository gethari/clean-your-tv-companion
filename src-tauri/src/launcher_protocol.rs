//! Versioned companion contract. No legacy file-write fallback is permitted.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const PROTOCOL_VERSION: u32 = 1;
pub const REQUIRED_FEATURES: [&str; 4] = [
    "collections_v1", "atomic_activation_v1", "revision_checks_v1", "readback_v1",
];
pub const SETTING_KEYS: [&str; 10] = [
    "wallpaper", "wallpaper_rotation_minutes", "wallpaper_dim", "wallpaper_order",
    "active_collection", "theme_schedule", "background_color", "accent_color", "reduced_motion",
    // Reserved safe setting for explicitly disabling a schedule without deleting it.
    "theme_schedule_enabled",
];

#[derive(Debug, Deserialize, Serialize)]
pub struct Limits {
    pub asset_bytes: u64,
    pub collection_bytes: u64,
    pub asset_count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Capabilities {
    pub protocol_version: u32,
    pub features: Vec<String>,
    pub limits: Limits,
}

pub fn capabilities(value: Value) -> Result<Capabilities, &'static str> {
    let value: Capabilities = serde_json::from_value(value).map_err(|_| "invalid_capabilities")?;
    if value.protocol_version != PROTOCOL_VERSION { return Err("unsupported_protocol"); }
    if !REQUIRED_FEATURES.iter().all(|required| value.features.iter().any(|feature| feature == required)) {
        return Err("unsupported_feature");
    }
    if value.limits.asset_bytes == 0 || value.limits.asset_bytes > 20 * 1024 * 1024
        || value.limits.collection_bytes < value.limits.asset_bytes
        || value.limits.collection_bytes > 256 * 1024 * 1024
        || value.limits.asset_count == 0 || value.limits.asset_count > 200 {
        return Err("invalid_limits");
    }
    Ok(value)
}

pub fn hex_id(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsPatch {
    #[serde(default)]
    pub set: Map<String, Value>,
    #[serde(default)]
    pub remove: Vec<String>,
}

fn color(value: &Value) -> bool {
    value.as_str().is_some_and(|s| s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|c| c.is_ascii_hexdigit()))
}

pub fn validate_patch(patch: &SettingsPatch, capabilities: &Capabilities) -> Result<(), &'static str> {
    for key in patch.set.keys().chain(patch.remove.iter()) {
        if !SETTING_KEYS.contains(&key.as_str()) { return Err("unsupported_setting"); }
        if matches!(key.as_str(), "theme_schedule" | "theme_schedule_enabled")
            && !capabilities.features.iter().any(|f| f == "theme_schedules_v1") {
            return Err("unsupported_feature");
        }
    }
    if patch.remove.iter().any(|key| patch.set.contains_key(key)) { return Err("invalid_patch"); }
    let unique: std::collections::HashSet<_> = patch.remove.iter().collect();
    if unique.len() != patch.remove.len() { return Err("invalid_patch"); }
    for (key, value) in &patch.set {
        let valid = match key.as_str() {
            "wallpaper" | "reduced_motion" | "theme_schedule_enabled" => value.is_boolean(),
            "wallpaper_rotation_minutes" => value.as_u64().is_some_and(|v| (1..=1440).contains(&v)),
            "wallpaper_dim" => value.as_u64().is_some_and(|v| v <= 100),
            "wallpaper_order" => value.as_str().is_some_and(|v| matches!(v, "sequential" | "shuffle")),
            "active_collection" => value.as_str().is_some_and(|v| hex_id(v, 32)),
            "background_color" | "accent_color" => color(value),
            // Schedule semantics are validated by the scheduler before activation.
            "theme_schedule" => value.is_object(),
            _ => false,
        };
        if !valid { return Err("invalid_setting_value"); }
    }
    Ok(())
}

/// Merge only a validated patch; unrelated configuration remains byte-value equivalent.
pub fn merge_settings(current: &Map<String, Value>, patch: &SettingsPatch, caps: &Capabilities) -> Result<Map<String, Value>, &'static str> {
    validate_patch(patch, caps)?;
    let mut result = current.clone();
    for key in &patch.remove { result.remove(key); }
    for (key, value) in &patch.set { result.insert(key.clone(), value.clone()); }
    Ok(result)
}

/// Private and future unknown settings must not cross the safe readback boundary.
pub fn safe_settings(current: &Map<String, Value>) -> Map<String, Value> {
    current.iter().filter(|(key, _)| SETTING_KEYS.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone())).collect()
}

pub fn validate_collections(collections: &[Value], caps: &Capabilities) -> Result<(), &'static str> {
    if collections.len() > 20 { return Err("too_many_collections"); }
    let mut ids = std::collections::HashSet::new();
    let mut count = 0;
    let mut bytes = 0u64;
    for collection in collections {
        let object = collection.as_object().ok_or("invalid_collection")?;
        if object.keys().any(|key| !["id", "assets", "order", "interval_minutes"].contains(&key.as_str())) {
            return Err("invalid_collection");
        }
        let id = collection["id"].as_str().filter(|id| hex_id(id, 32)).ok_or("invalid_collection_id")?;
        if !ids.insert(id) { return Err("duplicate_collection_id"); }
        if !collection["order"].as_str().is_some_and(|v| matches!(v, "shuffle" | "sequential"))
            || !collection["interval_minutes"].as_u64().is_some_and(|v| (1..=1440).contains(&v)) {
            return Err("invalid_playback_preferences");
        }
        let assets = collection["assets"].as_array().filter(|v| !v.is_empty()).ok_or("empty_collection")?;
        let mut hashes = std::collections::HashSet::new();
        for asset in assets {
            let object = asset.as_object().ok_or("invalid_asset")?;
            if object.keys().any(|key| !["sha256", "bytes", "mime"].contains(&key.as_str())) { return Err("invalid_asset"); }
            let hash = asset["sha256"].as_str().filter(|v| hex_id(v, 64)).ok_or("invalid_asset_hash")?;
            if !hashes.insert(hash) { return Err("duplicate_asset"); }
            let size = asset["bytes"].as_u64().filter(|v| *v > 0 && *v <= caps.limits.asset_bytes).ok_or("invalid_asset_size")?;
            if !asset["mime"].as_str().is_some_and(|v| matches!(v, "image/jpeg" | "image/png" | "image/webp")) {
                return Err("unsupported_image_format");
            }
            count += 1;
            bytes = bytes.checked_add(size).ok_or("collection_too_large")?;
            if count > caps.limits.asset_count || bytes > caps.limits.collection_bytes { return Err("collection_too_large"); }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn caps() -> Capabilities {
        capabilities(json!({"protocol_version":1,"features":REQUIRED_FEATURES,
            "limits":{"asset_bytes":20971520,"collection_bytes":268435456,"asset_count":200}})).unwrap()
    }

    #[test]
    fn fixture_capabilities_fail_closed_for_legacy_newer_and_incomplete_versions() {
        let fixtures: Vec<Value> = serde_json::from_str(include_str!("../../tests/fixtures/launcher-protocol/capabilities.json")).unwrap();
        for fixture in fixtures {
            assert_eq!(capabilities(fixture["response"].clone()).is_ok(), fixture["editable"].as_bool().unwrap(), "{}", fixture["name"]);
        }
    }

    #[test]
    fn patch_preserves_unknown_and_private_keys_without_exposing_them() {
        let current = json!({"wallpaper":false,"future_setting":{"nested":[1,2,3]},"provider_token":"fixture-secret","wallpaper_dim":30});
        let patch: SettingsPatch = serde_json::from_value(json!({"set":{"wallpaper":true},"remove":["wallpaper_dim"]})).unwrap();
        let merged = merge_settings(current.as_object().unwrap(), &patch, &caps()).unwrap();
        assert_eq!(merged["future_setting"], current["future_setting"]);
        assert_eq!(merged["provider_token"], current["provider_token"]);
        assert_eq!(safe_settings(&merged), json!({"wallpaper":true}).as_object().unwrap().clone());
        assert!(!merged.contains_key("wallpaper_dim"));
    }

    #[test]
    fn patches_reject_unknown_keys_conflicts_invalid_intervals_and_unsupported_schedules() {
        for value in [
            json!({"set":{"provider_token":"forbidden"}}),
            json!({"set":{"wallpaper":true},"remove":["wallpaper"]}),
            json!({"set":{"wallpaper_rotation_minutes":0}}),
            json!({"set":{"wallpaper_rotation_minutes":1.5}}),
            json!({"set":{"wallpaper_dim":101}}),
            json!({"set":{"active_collection":"../../outside"}}),
            json!({"set":{"theme_schedule":{}}}),
        ] {
            let patch: SettingsPatch = serde_json::from_value(value).unwrap();
            assert!(validate_patch(&patch, &caps()).is_err());
        }
        for minutes in [1,5,15,30,1440] {
            let patch: SettingsPatch = serde_json::from_value(json!({"set":{"wallpaper_rotation_minutes":minutes}})).unwrap();
            assert!(validate_patch(&patch, &caps()).is_ok());
        }
    }

    #[test]
    fn collection_manifests_reject_traversal_private_fields_and_excessive_sizes() {
        let valid = json!({"id":"a".repeat(32),"order":"shuffle","interval_minutes":15,"assets":[{"sha256":"b".repeat(64),"bytes":100,"mime":"image/png"}]});
        assert!(validate_collections(&[valid.clone()], &caps()).is_ok());
        for invalid in [
            { let mut v = valid.clone(); v["id"] = json!("../../outside"); v },
            { let mut v = valid.clone(); v["assets"][0]["bytes"] = json!(20971521); v },
            { let mut v = valid.clone(); v["credential"] = json!("fixture-secret"); v },
            { let mut v = valid.clone(); v["assets"][0]["path"] = json!("/private/file"); v },
        ] {
            assert!(validate_collections(&[invalid], &caps()).is_err());
        }
    }
}
