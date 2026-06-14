use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub const REDACTED: &str = "[REDACTED]";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedPayload {
    pub canonical: String,
    pub redacted: String,
    pub classification: String,
    pub payload_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalBinding {
    pub session_id: String,
    pub task_contract_hash: String,
    pub action_id: String,
    pub canonical_payload_hash: String,
    pub policy_version: String,
    pub environment_identity: String,
    pub expires_at: i64,
    pub approver: Option<String>,
}

pub fn classify_payload(payload: &Value) -> ClassifiedPayload {
    let canonical = canonical_json(payload);
    let payload_hash = sha256_hex(&canonical);
    let mut paths = Vec::new();
    let redacted_value = redact_value(payload, "$", &mut paths);
    let redacted = canonical_json(&redacted_value);
    let classification = canonical_json(&serde_json::json!({
        "schema": 1,
        "redacted_paths": paths,
        "contains_sensitive": !paths.is_empty(),
    }));

    ClassifiedPayload {
        canonical,
        redacted,
        classification,
        payload_hash,
    }
}

pub fn classify_payload_str(payload: &str) -> ClassifiedPayload {
    match serde_json::from_str::<Value>(payload) {
        Ok(value) => classify_payload(&value),
        Err(_) => {
            let value = Value::String(payload.to_string());
            classify_payload(&value)
        }
    }
}

pub fn mask_text_for_display(text: &str) -> String {
    classify_payload_str(text).redacted
}

pub fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(v) => v.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string()),
        Value::Array(items) => {
            let inner = items
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{}]", inner)
        }
        Value::Object(map) => {
            let sorted: BTreeMap<_, _> = map.iter().collect();
            let inner = sorted
                .into_iter()
                .map(|(k, v)| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(k).unwrap_or_else(|_| "\"\"".to_string()),
                        canonical_json(v)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{}}}", inner)
        }
    }
}

pub fn sha256_hex(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}

pub fn strict_mode_enabled() -> bool {
    std::env::var("ONUS_STRICT")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn policy_version() -> String {
    std::env::var("ONUS_POLICY_VERSION").unwrap_or_else(|_| crate::VERSION.to_string())
}

pub fn task_contract_hash(session_id: &str) -> String {
    std::env::var("ONUS_TASK_CONTRACT_HASH")
        .unwrap_or_else(|_| sha256_hex(&format!("session:{}", session_id)))
}

pub fn environment_identity() -> String {
    std::env::var("ONUS_ENVIRONMENT_IDENTITY").unwrap_or_else(|_| {
        let cwd = std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        sha256_hex(&format!("local:{}:{}", user, cwd))
    })
}

pub fn approval_ttl_secs() -> i64 {
    std::env::var("ONUS_APPROVAL_TTL_SECS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(300)
}

pub fn approval_action_id(session_id: &str, tool_name: &str, payload_hash: &str) -> String {
    format!(
        "approval-{}",
        &sha256_hex(&format!("{}|{}|{}", session_id, tool_name, payload_hash))[..32]
    )
}

pub fn local_token() -> String {
    std::env::var("ONUS_LOCAL_UI_TOKEN").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string())
}

pub fn token_from_url(url: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    query
        .split('&')
        .filter_map(|part| part.split_once('='))
        .find_map(|(k, v)| {
            if k == "token" {
                Some(v.to_string())
            } else {
                None
            }
        })
}

pub fn authorized_url(url: &str, expected_token: &str) -> bool {
    token_from_url(url).as_deref() == Some(expected_token)
}

fn redact_value(value: &Value, path: &str, redacted_paths: &mut Vec<String>) -> Value {
    match value {
        Value::Object(map) => {
            let mut redacted = Map::new();
            for (key, child) in map {
                let child_path = format!("{}.{}", path, key);
                if is_sensitive_key(key) {
                    redacted_paths.push(child_path);
                    redacted.insert(key.clone(), Value::String(REDACTED.to_string()));
                } else {
                    redacted.insert(
                        key.clone(),
                        redact_value(child, &child_path, redacted_paths),
                    );
                }
            }
            Value::Object(redacted)
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .enumerate()
                .map(|(idx, child)| {
                    redact_value(child, &format!("{}[{}]", path, idx), redacted_paths)
                })
                .collect(),
        ),
        Value::String(s) => redact_string(s, path, redacted_paths),
        other => other.clone(),
    }
}

fn redact_string(input: &str, path: &str, redacted_paths: &mut Vec<String>) -> Value {
    if looks_like_secret(input) {
        redacted_paths.push(path.to_string());
        Value::String(REDACTED.to_string())
    } else {
        Value::String(input.to_string())
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "secret",
        "token",
        "password",
        "passwd",
        "authorization",
        "cookie",
        "credential",
        "api_key",
        "apikey",
        "private_key",
        "access_key",
        "refresh",
        "session_key",
    ]
    .iter()
    .any(|needle| key.contains(needle))
}

fn looks_like_secret(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "aws_secret_access_key",
        "api_key=",
        "apikey=",
        "authorization:",
        "bearer ",
        "password=",
        "secret=",
        "token=",
        "private key",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_json_sorts_keys() {
        let a = serde_json::json!({"b": 2, "a": {"d": 4, "c": 3}});
        let b = serde_json::json!({"a": {"c": 3, "d": 4}, "b": 2});
        assert_eq!(canonical_json(&a), canonical_json(&b));
        assert_eq!(
            sha256_hex(&canonical_json(&a)),
            sha256_hex(&canonical_json(&b))
        );
    }

    #[test]
    fn redacts_sensitive_keys_and_values() {
        let payload = serde_json::json!({
            "token": "abc",
            "nested": {"content": "AWS_SECRET_ACCESS_KEY=\"abc123\""}
        });
        let classified = classify_payload(&payload);
        assert!(!classified.redacted.contains("abc123"));
        assert!(!classified.redacted.contains("\"abc\""));
        assert!(classified.redacted.contains(REDACTED));
        assert!(classified.classification.contains("contains_sensitive"));
    }

    #[test]
    fn auth_token_is_required_in_url() {
        assert!(authorized_url("/api/actions?token=abc", "abc"));
        assert!(!authorized_url("/api/actions", "abc"));
        assert!(!authorized_url("/api/actions?token=def", "abc"));
    }
}
