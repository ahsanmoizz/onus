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
        || looks_like_jwt(value)
        || looks_like_connection_string(value)
        || looks_like_private_key(value)
        || high_entropy_string(value)
}

/// Detect JWTs: base64url-encoded header.payload[.sig] with typical lengths
fn looks_like_jwt(value: &str) -> bool {
    let trimmed = value.trim();
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    let total_len = trimmed.len();
    // JWTs are typically 150-2000 chars
    if !(80..=5000).contains(&total_len) {
        return false;
    }
    // Each part is valid base64url (no padding issues in header/payload)
    parts.iter().all(|p| {
        !p.is_empty()
            && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '=')
    })
}

/// Detect connection strings like postgres://user:pass@host/db
fn looks_like_connection_string(value: &str) -> bool {
    let trimmed = value.trim().to_ascii_lowercase();
    for prefix in &[
        "postgresql://",
        "postgres://",
        "mysql://",
        "mongodb://",
        "mongodb+srv://",
        "redis://",
        "rediss://",
        "amqp://",
        "rabbitmq://",
        "jdbc:",
        "sqlite://",
        "snowflake://",
        "bigquery://",
    ] {
        if trimmed.starts_with(prefix) {
            return true;
        }
    }
    false
}

/// Detect PEM-encoded or raw private key material
fn looks_like_private_key(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.starts_with("-----BEGIN") && trimmed.contains("PRIVATE KEY") {
        return true;
    }
    // Detect ed25519/curve25519 seed hex (64 hex chars, common key length)
    let cleaned: String = trimmed.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if cleaned.len() == 64 && trimmed.len() <= 70 {
        // Possible 32-byte hex seed — check it's not just a sha256 hash by looking at charset
        let has_non_hex = trimmed.chars().any(|c| !c.is_ascii_hexdigit());
        if !has_non_hex {
            return true;
        }
    }
    false
}

/// Shannon entropy heuristic: strings with high entropy (entropy ≥ 4.2) are likely secrets
fn high_entropy_string(value: &str) -> bool {
    let trimmed = value.trim();
    // Skip short strings, URLs, file paths, common identifiers
    if trimmed.len() < 16 {
        return false;
    }
    // Skip natural language (contains spaces, common words)
    let word_count = trimmed.split_whitespace().count();
    if word_count > 3 {
        return false;
    }
    // Skip URLs and file paths
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with('/')
        || trimmed.starts_with("./")
        || trimmed.starts_with("../")
    {
        return false;
    }
    // Skip simple booleans, numbers, UUIDs
    if trimmed.parse::<f64>().is_ok()
        || trimmed == "true"
        || trimmed == "false"
        || trimmed.len() == 36 && trimmed.chars().filter(|c| *c == '-').count() == 4
    {
        return false;
    }

    let entropy = shannon_entropy(trimmed);
    entropy > 4.0
}

/// Shannon entropy calculation over a string's byte values
fn shannon_entropy(s: &str) -> f64 {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return 0.0;
    }
    let len = bytes.len() as f64;
    let mut freq = [0u64; 256];
    for &b in bytes {
        freq[b as usize] += 1;
    }
    -freq
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            p * p.log2()
        })
        .sum::<f64>()
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

    #[test]
    fn detects_jwt_strings() {
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3j2k9Jj6Hv9K6hV5Y2D3eQ0fF0gG9l4s";
        assert!(looks_like_jwt(jwt));
        assert!(!looks_like_jwt("hello.world"));
        assert!(!looks_like_jwt("hello.world.foo.bar"));
    }

    #[test]
    fn detects_connection_strings() {
        assert!(looks_like_connection_string("postgres://user:pass@localhost:5432/db"));
        assert!(looks_like_connection_string("mongodb+srv://admin:secret@cluster.mongodb.net"));
        assert!(!looks_like_connection_string("https://example.com/api"));
    }

    #[test]
    fn detects_private_key_pem() {
        assert!(looks_like_private_key("-----BEGIN PRIVATE KEY-----\nMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg\n-----END PRIVATE KEY-----"));
        assert!(!looks_like_private_key("hello world"));
    }

    #[test]
    fn entropy_detects_high_entropy_strings() {
        let high_entropy = "aB3dE7fG1hI4jK9mL0nO2pQ5rS6tU8vW0xY1zC4";
        assert!(high_entropy_string(high_entropy));
        assert!(!high_entropy_string("hello world"));
        assert!(!high_entropy_string("true"));
        assert!(!high_entropy_string("https://example.com/path"));
    }

    #[test]
    fn content_aware_redaction_works() {
        let payload = serde_json::json!({
            "prompt": "what is the weather?",
            "jwt_token": "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3j2k9Jj6Hv9K6hV5Y2D3eQ0fF0gG9l4s",
            "db_url": "postgres://admin:supersecret@localhost:5432/prod",
        });
        let c = classify_payload(&payload);
        assert!(c.redacted.contains(REDACTED));
        assert!(!c.redacted.contains("supersecret"));
    }
}
