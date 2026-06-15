//! Narrow L4 authority proof for a disposable SQLite database.
//!
//! This is intentionally small: Onus owns the database authority and the
//! long-lived broker secret, while the agent receives only a short-lived scoped
//! capability bound to one canonical payload hash.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

const AUTHORITY_DIR: &str = "authority";
const METADATA_FILE: &str = "authority.json";
const SECRET_FILE: &str = "authority.secret";
const CAPABILITIES_FILE: &str = "capabilities.json";
const RECEIPTS_FILE: &str = "receipts.jsonl";
const POLICY_VERSION: &str = "l4-disposable-sqlite-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityMetadata {
    pub schema_version: u32,
    pub authority_id: String,
    pub environment_identity: String,
    pub db_path: PathBuf,
    pub credential_hash: String,
    pub created_at_unix: i64,
    pub status: String,
    pub supported_operation: String,
    pub l4_claim: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRecord {
    pub capability_id: String,
    pub token_hash: String,
    pub authority_id: String,
    pub session_id: String,
    pub action_id: String,
    pub canonical_payload_hash: String,
    pub policy_version: String,
    pub environment_identity: String,
    pub approver: String,
    pub issued_at_unix: i64,
    pub expires_at_unix: i64,
    pub revoked_at_unix: Option<i64>,
    pub used_at_unix: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityReceipt {
    pub schema_version: u32,
    pub receipt_id: String,
    pub authority_id: String,
    pub capability_id: Option<String>,
    pub action_id: Option<String>,
    pub decision: String,
    pub reason: String,
    pub environment_identity: String,
    pub canonical_payload_hash: Option<String>,
    pub redacted_payload: Option<Value>,
    pub operation: Option<String>,
    pub row_id: Option<String>,
    pub compensation: Option<CompensationReceipt>,
    pub created_at_unix: i64,
    pub previous_receipt_hash: String,
    pub receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationReceipt {
    pub supported: bool,
    pub executed: bool,
    pub operation: String,
    pub verification: String,
}

#[derive(Debug, Clone)]
pub struct InitDisposableDbOptions {
    pub authority_id: String,
    pub db_path: PathBuf,
    pub environment_identity: String,
}

#[derive(Debug, Clone)]
pub struct AuthorizeOptions {
    pub authority_id: String,
    pub session_id: String,
    pub payload_path: PathBuf,
    pub approver: String,
    pub ttl_seconds: i64,
    pub human_approved: bool,
}

#[derive(Debug, Clone)]
pub struct ExecuteOptions {
    pub authority_id: String,
    pub capability_token: String,
    pub payload_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RevokeOptions {
    pub authority_id: String,
    pub capability_token: String,
}

#[derive(Debug, Clone)]
pub struct CompensateOptions {
    pub authority_id: String,
    pub receipt_id: String,
}

pub fn authority_store_root() -> PathBuf {
    crate::data_dir().join(AUTHORITY_DIR)
}

pub fn init_disposable_db(options: InitDisposableDbOptions) -> Result<AuthorityMetadata> {
    validate_id(&options.authority_id, "authority id")?;
    validate_environment(&options.environment_identity)?;
    let root = authority_root(&options.authority_id);
    if root.exists() {
        anyhow::bail!("authority already exists: {}", options.authority_id);
    }
    fs::create_dir_all(&root)?;

    let db_path = absolute_normalized(&options.db_path)?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(&db_path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS l4_items (
            row_id TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            created_by TEXT NOT NULL,
            created_at_unix INTEGER NOT NULL
        );",
    )?;

    let secret = format!("onus-l4-authority-{}", Uuid::new_v4());
    let secret_path = root.join(SECRET_FILE);
    fs::write(&secret_path, &secret)?;
    let metadata = AuthorityMetadata {
        schema_version: 1,
        authority_id: options.authority_id,
        environment_identity: options.environment_identity,
        db_path,
        credential_hash: crate::security::sha256_hex(&secret),
        created_at_unix: crate::security::now_unix(),
        status: "active".to_string(),
        supported_operation: "insert_l4_item".to_string(),
        l4_claim: "ONUS_CONTROLLED_AUTHORITY_DISPOSABLE_SQLITE".to_string(),
    };
    save_metadata(&metadata)?;
    save_capabilities(&metadata.authority_id, &BTreeMap::new())?;
    Ok(metadata)
}

pub fn authorize(options: AuthorizeOptions) -> Result<(CapabilityRecord, String)> {
    if !options.human_approved {
        anyhow::bail!("human approval is required for this L4 capability");
    }
    if options.ttl_seconds <= 0 || options.ttl_seconds > 3600 {
        anyhow::bail!("ttl must be between 1 and 3600 seconds");
    }
    validate_id(&options.session_id, "session id")?;
    let metadata = load_metadata(&options.authority_id)?;
    verify_authority_secret(&metadata)?;
    let payload = read_payload(&options.payload_path)?;
    validate_payload(&metadata, &payload)?;
    let classified = crate::security::classify_payload(&payload);
    let capability_id = format!("cap-{}", Uuid::new_v4());
    let capability_token = format!("onus-cap-{}", Uuid::new_v4());
    let now = crate::security::now_unix();
    let action_id = crate::security::approval_action_id(
        &options.session_id,
        "authority.disposable_sqlite.insert_l4_item",
        &classified.payload_hash,
    );
    let record = CapabilityRecord {
        capability_id,
        token_hash: crate::security::sha256_hex(&capability_token),
        authority_id: metadata.authority_id.clone(),
        session_id: options.session_id,
        action_id,
        canonical_payload_hash: classified.payload_hash.clone(),
        policy_version: POLICY_VERSION.to_string(),
        environment_identity: metadata.environment_identity.clone(),
        approver: options.approver,
        issued_at_unix: now,
        expires_at_unix: now + options.ttl_seconds,
        revoked_at_unix: None,
        used_at_unix: None,
        status: "active".to_string(),
    };
    let mut capabilities = load_capabilities(&metadata.authority_id)?;
    capabilities.insert(record.token_hash.clone(), record.clone());
    save_capabilities(&metadata.authority_id, &capabilities)?;
    append_receipt(AuthorityReceiptInput {
        metadata: &metadata,
        capability_id: Some(record.capability_id.clone()),
        action_id: Some(record.action_id.clone()),
        decision: "CAPABILITY_ISSUED",
        reason: "short-lived exact-payload capability issued after human approval",
        payload: Some(&payload),
        operation: Some("insert_l4_item"),
        row_id: payload
            .get("row_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        compensation: None,
    })?;
    Ok((record, capability_token))
}

pub fn execute(options: ExecuteOptions) -> Result<AuthorityReceipt> {
    let metadata = load_metadata(&options.authority_id)?;
    verify_authority_secret(&metadata)?;
    let payload = read_payload(&options.payload_path)?;
    validate_payload(&metadata, &payload)?;
    let classified = crate::security::classify_payload(&payload);
    let token_hash = crate::security::sha256_hex(&options.capability_token);
    let mut capabilities = load_capabilities(&metadata.authority_id)?;
    let capability = capabilities
        .get_mut(&token_hash)
        .ok_or_else(|| anyhow::anyhow!("capability not found"))?;
    validate_capability(capability, &metadata, &classified.payload_hash)?;

    let row_id = payload
        .get("row_id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("payload missing row_id"))?
        .to_string();
    let value = payload
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("payload missing value"))?
        .to_string();
    let conn = Connection::open(&metadata.db_path)?;
    conn.execute(
        "INSERT INTO l4_items(row_id, value, created_by, created_at_unix) VALUES (?1, ?2, ?3, ?4)",
        params![
            row_id,
            value,
            capability.action_id,
            crate::security::now_unix()
        ],
    )
    .with_context(|| "broker-executed SQLite insert failed")?;

    capability.used_at_unix = Some(crate::security::now_unix());
    capability.status = "used".to_string();
    let capability_id = capability.capability_id.clone();
    let action_id = capability.action_id.clone();
    save_capabilities(&metadata.authority_id, &capabilities)?;

    append_receipt(AuthorityReceiptInput {
        metadata: &metadata,
        capability_id: Some(capability_id),
        action_id: Some(action_id),
        decision: "EXECUTED",
        reason: "broker executed exact authorized payload without exposing long-lived credential",
        payload: Some(&payload),
        operation: Some("insert_l4_item"),
        row_id: Some(row_id),
        compensation: Some(CompensationReceipt {
            supported: true,
            executed: false,
            operation: "delete_l4_item_by_row_id".to_string(),
            verification: "pending".to_string(),
        }),
    })
}

pub fn revoke(options: RevokeOptions) -> Result<CapabilityRecord> {
    let metadata = load_metadata(&options.authority_id)?;
    verify_authority_secret(&metadata)?;
    let token_hash = crate::security::sha256_hex(&options.capability_token);
    let mut capabilities = load_capabilities(&metadata.authority_id)?;
    let capability = capabilities
        .get_mut(&token_hash)
        .ok_or_else(|| anyhow::anyhow!("capability not found"))?;
    if capability.used_at_unix.is_some() {
        anyhow::bail!(
            "used capability cannot be revoked; it can only be compensated when supported"
        );
    }
    capability.revoked_at_unix = Some(crate::security::now_unix());
    capability.status = "revoked".to_string();
    let record = capability.clone();
    save_capabilities(&metadata.authority_id, &capabilities)?;
    append_receipt(AuthorityReceiptInput {
        metadata: &metadata,
        capability_id: Some(record.capability_id.clone()),
        action_id: Some(record.action_id.clone()),
        decision: "REVOKED",
        reason: "short-lived capability revoked before use",
        payload: None,
        operation: None,
        row_id: None,
        compensation: None,
    })?;
    Ok(record)
}

pub fn compensate(options: CompensateOptions) -> Result<AuthorityReceipt> {
    let metadata = load_metadata(&options.authority_id)?;
    verify_authority_secret(&metadata)?;
    let receipt = find_receipt(&metadata.authority_id, &options.receipt_id)?
        .ok_or_else(|| anyhow::anyhow!("receipt not found"))?;
    if receipt.decision != "EXECUTED" || receipt.operation.as_deref() != Some("insert_l4_item") {
        anyhow::bail!("only executed insert_l4_item receipts can be compensated");
    }
    let row_id = receipt
        .row_id
        .clone()
        .ok_or_else(|| anyhow::anyhow!("receipt missing row_id"))?;
    let conn = Connection::open(&metadata.db_path)?;
    let deleted = conn.execute("DELETE FROM l4_items WHERE row_id = ?1", params![row_id])?;
    let remaining: i64 = conn.query_row(
        "SELECT COUNT(*) FROM l4_items WHERE row_id = ?1",
        params![receipt.row_id],
        |row| row.get(0),
    )?;
    append_receipt(AuthorityReceiptInput {
        metadata: &metadata,
        capability_id: receipt.capability_id,
        action_id: receipt.action_id,
        decision: "COMPENSATED",
        reason: if deleted > 0 {
            "compensation deleted the inserted disposable row"
        } else {
            "compensation was idempotent; row was already absent"
        },
        payload: None,
        operation: Some("delete_l4_item_by_row_id"),
        row_id: Some(row_id),
        compensation: Some(CompensationReceipt {
            supported: true,
            executed: true,
            operation: "delete_l4_item_by_row_id".to_string(),
            verification: if remaining == 0 {
                "row_absent".to_string()
            } else {
                "row_still_present".to_string()
            },
        }),
    })
}

pub fn inspect(authority_id: &str) -> Result<AuthorityMetadata> {
    let metadata = load_metadata(authority_id)?;
    verify_authority_secret(&metadata)?;
    Ok(metadata)
}

pub fn load_receipts(authority_id: &str) -> Result<Vec<AuthorityReceipt>> {
    validate_id(authority_id, "authority id")?;
    let path = authority_root(authority_id).join(RECEIPTS_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut receipts = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if !line.trim().is_empty() {
            receipts.push(serde_json::from_str(&line)?);
        }
    }
    Ok(receipts)
}

fn validate_capability(
    capability: &CapabilityRecord,
    metadata: &AuthorityMetadata,
    payload_hash: &str,
) -> Result<()> {
    if capability.status != "active" {
        anyhow::bail!("capability is not active: {}", capability.status);
    }
    if capability.environment_identity != metadata.environment_identity {
        anyhow::bail!("capability environment mismatch");
    }
    if capability.canonical_payload_hash != payload_hash {
        anyhow::bail!(
            "altered payload denied: capability is bound to a different canonical payload hash"
        );
    }
    let now = crate::security::now_unix();
    if capability.expires_at_unix <= now {
        anyhow::bail!("capability expired");
    }
    if capability.revoked_at_unix.is_some() {
        anyhow::bail!("capability revoked");
    }
    if capability.used_at_unix.is_some() {
        anyhow::bail!("capability already used");
    }
    Ok(())
}

fn validate_payload(metadata: &AuthorityMetadata, payload: &Value) -> Result<()> {
    let obj = payload
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("payload must be a JSON object"))?;
    expect_str(obj, "operation", "insert_l4_item")?;
    expect_str(obj, "environment_identity", &metadata.environment_identity)?;
    let row_id = obj
        .get("row_id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("payload missing row_id"))?;
    validate_id(row_id, "row_id")?;
    let value = obj
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("payload missing value"))?;
    if value.is_empty() || value.len() > 4096 {
        anyhow::bail!("payload value must be 1..4096 bytes");
    }
    Ok(())
}

fn expect_str(map: &serde_json::Map<String, Value>, key: &str, expected: &str) -> Result<()> {
    let value = map
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("payload missing {}", key))?;
    if value != expected {
        anyhow::bail!("payload {} mismatch", key);
    }
    Ok(())
}

fn validate_environment(value: &str) -> Result<()> {
    if !(value.starts_with("disposable-") || value.starts_with("staging-")) {
        anyhow::bail!("L4 proof only supports disposable-* or staging-* environments");
    }
    validate_id(value, "environment identity")
}

fn validate_id(value: &str, label: &str) -> Result<()> {
    let ok = !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok {
        Ok(())
    } else {
        anyhow::bail!("invalid {}", label);
    }
}

fn load_metadata(authority_id: &str) -> Result<AuthorityMetadata> {
    validate_id(authority_id, "authority id")?;
    let path = authority_root(authority_id).join(METADATA_FILE);
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("authority metadata not found: {}", path.display()))?;
    let metadata: AuthorityMetadata = serde_json::from_str(&raw)?;
    if metadata.authority_id != authority_id {
        anyhow::bail!("authority metadata id mismatch");
    }
    Ok(metadata)
}

fn save_metadata(metadata: &AuthorityMetadata) -> Result<()> {
    let root = authority_root(&metadata.authority_id);
    fs::create_dir_all(&root)?;
    fs::write(
        root.join(METADATA_FILE),
        serde_json::to_string_pretty(metadata)?,
    )?;
    Ok(())
}

fn load_capabilities(authority_id: &str) -> Result<BTreeMap<String, CapabilityRecord>> {
    let path = authority_root(authority_id).join(CAPABILITIES_FILE);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn save_capabilities(
    authority_id: &str,
    capabilities: &BTreeMap<String, CapabilityRecord>,
) -> Result<()> {
    fs::write(
        authority_root(authority_id).join(CAPABILITIES_FILE),
        serde_json::to_string_pretty(capabilities)?,
    )?;
    Ok(())
}

fn verify_authority_secret(metadata: &AuthorityMetadata) -> Result<()> {
    let secret = fs::read_to_string(authority_root(&metadata.authority_id).join(SECRET_FILE))
        .with_context(|| "broker-held authority credential is missing")?;
    if crate::security::sha256_hex(&secret) != metadata.credential_hash {
        anyhow::bail!("broker-held authority credential hash mismatch");
    }
    Ok(())
}

fn read_payload(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

struct AuthorityReceiptInput<'a> {
    metadata: &'a AuthorityMetadata,
    capability_id: Option<String>,
    action_id: Option<String>,
    decision: &'a str,
    reason: &'a str,
    payload: Option<&'a Value>,
    operation: Option<&'a str>,
    row_id: Option<String>,
    compensation: Option<CompensationReceipt>,
}

fn append_receipt(input: AuthorityReceiptInput<'_>) -> Result<AuthorityReceipt> {
    let receipts = load_receipts(&input.metadata.authority_id)?;
    let previous_hash = receipts
        .last()
        .map(|receipt| receipt.receipt_hash.clone())
        .unwrap_or_default();
    let classified = input.payload.map(crate::security::classify_payload);
    let mut receipt = AuthorityReceipt {
        schema_version: 1,
        receipt_id: format!("l4r-{}", Uuid::new_v4()),
        authority_id: input.metadata.authority_id.clone(),
        capability_id: input.capability_id,
        action_id: input.action_id,
        decision: input.decision.to_string(),
        reason: input.reason.to_string(),
        environment_identity: input.metadata.environment_identity.clone(),
        canonical_payload_hash: classified.as_ref().map(|c| c.payload_hash.clone()),
        redacted_payload: classified
            .as_ref()
            .and_then(|c| serde_json::from_str(&c.redacted).ok()),
        operation: input.operation.map(str::to_string),
        row_id: input.row_id,
        compensation: input.compensation,
        created_at_unix: crate::security::now_unix(),
        previous_receipt_hash: previous_hash,
        receipt_hash: String::new(),
    };
    let canonical = crate::security::canonical_json(&serde_json::to_value(&receipt)?);
    receipt.receipt_hash = crate::security::sha256_hex(&canonical);
    let path = authority_root(&input.metadata.authority_id).join(RECEIPTS_FILE);
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", serde_json::to_string(&receipt)?)?;
    Ok(receipt)
}

fn find_receipt(authority_id: &str, receipt_id: &str) -> Result<Option<AuthorityReceipt>> {
    Ok(load_receipts(authority_id)?
        .into_iter()
        .find(|receipt| receipt.receipt_id == receipt_id))
}

fn authority_root(authority_id: &str) -> PathBuf {
    authority_store_root().join(authority_id)
}

fn absolute_normalized(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("onus-l4-{}-{}", name, Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn payload(path: &Path, env: &str, row: &str, value: &str) {
        fs::write(
            path,
            serde_json::to_string(&serde_json::json!({
                "operation": "insert_l4_item",
                "environment_identity": env,
                "row_id": row,
                "value": value
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn disposable_l4_authority_executes_once_and_compensates() {
        let root = temp_root("proof");
        std::env::set_var("ONUS_DATA_DIR", root.join("data"));
        let db = root.join("db.sqlite");
        let authority_id = format!("auth-{}", Uuid::new_v4());
        let metadata = init_disposable_db(InitDisposableDbOptions {
            authority_id: authority_id.clone(),
            db_path: db.clone(),
            environment_identity: "disposable-test".to_string(),
        })
        .unwrap();
        let secret = fs::read_to_string(authority_root(&authority_id).join(SECRET_FILE)).unwrap();
        let payload_path = root.join("payload.json");
        payload(
            &payload_path,
            &metadata.environment_identity,
            "row-1",
            "value",
        );

        let (_cap, token) = authorize(AuthorizeOptions {
            authority_id: authority_id.clone(),
            session_id: "session-1".to_string(),
            payload_path: payload_path.clone(),
            approver: "human".to_string(),
            ttl_seconds: 60,
            human_approved: true,
        })
        .unwrap();
        let receipt = execute(ExecuteOptions {
            authority_id: authority_id.clone(),
            capability_token: token.clone(),
            payload_path: payload_path.clone(),
        })
        .unwrap();
        assert_eq!(receipt.decision, "EXECUTED");
        assert!(execute(ExecuteOptions {
            authority_id: authority_id.clone(),
            capability_token: token,
            payload_path: payload_path.clone(),
        })
        .is_err());

        let compensation = compensate(CompensateOptions {
            authority_id: authority_id.clone(),
            receipt_id: receipt.receipt_id,
        })
        .unwrap();
        assert_eq!(compensation.decision, "COMPENSATED");
        let all_receipts = load_receipts(&authority_id).unwrap();
        let raw_receipts = all_receipts
            .iter()
            .map(|r| serde_json::to_string(r).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!raw_receipts.contains(&secret));
        let _ = fs::remove_dir_all(root);
    }
}
