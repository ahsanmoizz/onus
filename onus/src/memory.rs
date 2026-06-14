use crate::security;
use ring::aead::{self, Aad, LessSafeKey, Nonce, UnboundKey};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const NONCE_LEN: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Session,
    Project,
    Policy,
    Incident,
    UserCapability,
}

impl std::fmt::Display for MemoryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Session => write!(f, "session"),
            Self::Project => write!(f, "project"),
            Self::Policy => write!(f, "policy"),
            Self::Incident => write!(f, "incident"),
            Self::UserCapability => write!(f, "user_capability"),
        }
    }
}

impl MemoryKind {
    fn from_db(value: &str) -> Option<Self> {
        match value {
            "session" => Some(Self::Session),
            "project" => Some(Self::Project),
            "policy" => Some(Self::Policy),
            "incident" => Some(Self::Incident),
            "user_capability" => Some(Self::UserCapability),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryScope {
    pub tenant_id: String,
    pub project_id: String,
    pub session_id: Option<String>,
}

impl MemoryScope {
    pub fn for_workspace(workspace_root: &str, session_id: Option<String>) -> Self {
        Self {
            tenant_id: tenant_id(),
            project_id: project_id(workspace_root),
            session_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryProvenance {
    pub actor_type: String,
    pub actor_id: String,
    pub source: String,
    pub reason: String,
}

impl MemoryProvenance {
    pub fn system(source: &str, reason: &str) -> Self {
        Self {
            actor_type: "system".to_string(),
            actor_id: "onus".to_string(),
            source: source.to_string(),
            reason: reason.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInput {
    pub kind: MemoryKind,
    pub scope: MemoryScope,
    pub key: String,
    pub value: Value,
    pub sensitive: bool,
    pub provenance: MemoryProvenance,
    pub retention_days: Option<u32>,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContext {
    pub kind: MemoryKind,
    pub key: String,
    pub summary: String,
    pub provenance: MemoryProvenance,
    pub version: u32,
}

impl std::fmt::Display for MemoryContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{} v{} [{}:{}] {}",
            self.kind,
            self.key,
            self.version,
            self.provenance.actor_type,
            self.provenance.source,
            self.summary
        )
    }
}

pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        ensure_schema(&conn)?;
        Ok(Self { conn })
    }

    pub fn put(&mut self, input: MemoryInput) -> anyhow::Result<String> {
        insert_memory(&self.conn, input)
    }

    pub fn delete_scope(
        &mut self,
        tenant_id: &str,
        project_id: &str,
        session_id: Option<&str>,
    ) -> anyhow::Result<usize> {
        let now = now_timestamp();
        let count = if let Some(session_id) = session_id {
            self.conn.execute(
                "UPDATE onus_memory
                 SET deleted_at = ?1, updated_at = ?1
                 WHERE tenant_id = ?2 AND project_id = ?3 AND session_id = ?4 AND deleted_at IS NULL",
                params![now, tenant_id, project_id, session_id],
            )?
        } else {
            self.conn.execute(
                "UPDATE onus_memory
                 SET deleted_at = ?1, updated_at = ?1
                 WHERE tenant_id = ?2 AND project_id = ?3 AND deleted_at IS NULL",
                params![now, tenant_id, project_id],
            )?
        };
        Ok(count)
    }

    pub fn retrieve_relevant(
        &self,
        scope: &MemoryScope,
        query: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<MemoryContext>> {
        retrieve_relevant_conn(&self.conn, scope, query, limit)
    }
}

pub fn ensure_schema(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS onus_memory (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            session_id TEXT,
            key TEXT NOT NULL,
            value_ciphertext TEXT NOT NULL,
            value_redacted TEXT NOT NULL,
            value_hash TEXT NOT NULL,
            classification TEXT NOT NULL,
            sensitive INTEGER NOT NULL DEFAULT 0,
            provenance_json TEXT NOT NULL,
            version INTEGER NOT NULL,
            review_status TEXT NOT NULL,
            retention_expires_at INTEGER,
            deleted_at INTEGER,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            mutable_by_agent INTEGER NOT NULL DEFAULT 0
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_onus_memory_identity
        ON onus_memory(kind, tenant_id, project_id, COALESCE(session_id, ''), key, version);

        CREATE INDEX IF NOT EXISTS idx_onus_memory_scope
        ON onus_memory(kind, tenant_id, project_id, session_id, deleted_at, retention_expires_at);",
    )?;
    Ok(())
}

pub fn insert_memory(conn: &Connection, input: MemoryInput) -> anyhow::Result<String> {
    ensure_schema(conn)?;
    validate_memory_input(&input)?;
    let now = now_timestamp();
    let retention_expires_at = input
        .retention_days
        .map(|days| now + (days as i64 * 86_400));
    let provenance_json = security::canonical_json(&serde_json::to_value(&input.provenance)?);
    let classified = security::classify_payload(&input.value);
    let encrypted = encrypt_text(&classified.redacted)?;
    let version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) + 1
             FROM onus_memory
             WHERE kind = ?1 AND tenant_id = ?2 AND project_id = ?3
               AND COALESCE(session_id, '') = COALESCE(?4, '') AND key = ?5",
            params![
                input.kind.to_string(),
                input.scope.tenant_id,
                input.scope.project_id,
                input.scope.session_id,
                input.key,
            ],
            |row| row.get::<_, u32>(0),
        )
        .unwrap_or(1);
    let id = format!(
        "mem-{}",
        &security::sha256_hex(&format!(
            "{}|{}|{}|{}|{}|{}",
            input.kind,
            input.scope.tenant_id,
            input.scope.project_id,
            input.scope.session_id.clone().unwrap_or_default(),
            input.key,
            version
        ))[..32]
    );

    conn.execute(
        "INSERT INTO onus_memory
            (id, kind, tenant_id, project_id, session_id, key, value_ciphertext,
             value_redacted, value_hash, classification, sensitive, provenance_json,
             version, review_status, retention_expires_at, deleted_at, created_at,
             updated_at, mutable_by_agent)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, NULL, ?16, ?16, ?17)",
        params![
            id,
            input.kind.to_string(),
            input.scope.tenant_id,
            input.scope.project_id,
            input.scope.session_id,
            input.key,
            encrypted,
            classified.redacted,
            classified.payload_hash,
            classified.classification,
            if input.sensitive { 1 } else { 0 },
            provenance_json,
            version,
            input.review_status,
            retention_expires_at,
            now,
            if input.provenance.actor_type == "agent" { 0 } else { 1 },
        ],
    )?;
    Ok(id)
}

pub fn retrieve_relevant_conn(
    conn: &Connection,
    scope: &MemoryScope,
    query: &str,
    limit: usize,
) -> anyhow::Result<Vec<MemoryContext>> {
    ensure_schema(conn)?;
    let now = now_timestamp();
    let mut stmt = conn.prepare(
        "SELECT kind, key, value_redacted, provenance_json, version
         FROM onus_memory
         WHERE tenant_id = ?1
           AND project_id = ?2
           AND deleted_at IS NULL
           AND (retention_expires_at IS NULL OR retention_expires_at > ?3)
           AND (
             kind IN ('project', 'policy', 'incident', 'user_capability')
             OR (kind = 'session' AND session_id = ?4)
           )
         ORDER BY updated_at DESC, version DESC
         LIMIT 200",
    )?;
    let rows = stmt.query_map(
        params![
            scope.tenant_id,
            scope.project_id,
            now,
            scope.session_id.clone().unwrap_or_default()
        ],
        |row| {
            let kind_raw: String = row.get(0)?;
            let provenance_json: String = row.get(3)?;
            let provenance = serde_json::from_str::<MemoryProvenance>(&provenance_json)
                .unwrap_or_else(|_| MemoryProvenance::system("unknown", "unparseable provenance"));
            Ok(MemoryContext {
                kind: MemoryKind::from_db(&kind_raw).unwrap_or(MemoryKind::Project),
                key: row.get(1)?,
                summary: row.get(2)?,
                provenance,
                version: row.get(4)?,
            })
        },
    )?;

    let terms = relevant_terms(query);
    let mut contexts = Vec::new();
    for item in rows {
        let item = item?;
        if is_relevant(&item, &terms) {
            contexts.push(item);
        }
        if contexts.len() >= limit {
            break;
        }
    }
    Ok(contexts)
}

pub fn remember_session_intake(
    conn: &Connection,
    scope: MemoryScope,
    original_prompt: &str,
    normalized_objective: &str,
) -> anyhow::Result<()> {
    insert_memory(
        conn,
        MemoryInput {
            kind: MemoryKind::Session,
            scope,
            key: "task_intake".to_string(),
            value: serde_json::json!({
                "original_prompt": original_prompt,
                "normalized_objective": normalized_objective,
            }),
            sensitive: false,
            provenance: MemoryProvenance::system("prompt_intake", "task intake persisted"),
            retention_days: Some(30),
            review_status: "system_recorded".to_string(),
        },
    )
    .map(|_| ())
}

pub fn remember_incident(
    conn: &Connection,
    scope: MemoryScope,
    key: &str,
    value: Value,
) -> anyhow::Result<()> {
    insert_memory(
        conn,
        MemoryInput {
            kind: MemoryKind::Incident,
            scope,
            key: key.to_string(),
            value,
            sensitive: false,
            provenance: MemoryProvenance::system(
                "action_evaluation",
                "blocked_or_escalated_action",
            ),
            retention_days: Some(90),
            review_status: "system_recorded".to_string(),
        },
    )
    .map(|_| ())
}

fn validate_memory_input(input: &MemoryInput) -> anyhow::Result<()> {
    if input.scope.tenant_id.trim().is_empty()
        || input.scope.project_id.trim().is_empty()
        || input.key.trim().is_empty()
    {
        anyhow::bail!("memory scope and key must be explicit");
    }
    if input.kind == MemoryKind::Policy
        && input.provenance.actor_type == "agent"
        && input.review_status != "approved"
    {
        anyhow::bail!("policy memory cannot be changed silently by an agent");
    }
    Ok(())
}

fn encrypt_text(text: &str) -> anyhow::Result<String> {
    let key_bytes = memory_key();
    let unbound = UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
        .map_err(|_| anyhow::anyhow!("failed to initialize memory encryption key"))?;
    let key = LessSafeKey::new(unbound);
    let nonce_bytes = nonce_bytes();
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut in_out = text.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| anyhow::anyhow!("failed to encrypt memory field"))?;
    Ok(format!(
        "{}:{}",
        hex::encode(nonce_bytes),
        hex::encode(in_out)
    ))
}

fn memory_key() -> [u8; 32] {
    let material = std::env::var("ONUS_MEMORY_KEY")
        .unwrap_or_else(|_| format!("onus-memory:{}", security::environment_identity()));
    Sha256::digest(material.as_bytes()).into()
}

fn nonce_bytes() -> [u8; NONCE_LEN] {
    let seed = format!("{}:{}", now_timestamp(), uuid::Uuid::new_v4());
    let digest = Sha256::digest(seed.as_bytes());
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&digest[..NONCE_LEN]);
    nonce
}

fn relevant_terms(query: &str) -> Vec<String> {
    query
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-' && c != '/')
        .map(|item| item.to_ascii_lowercase())
        .filter(|item| item.len() >= 3)
        .take(24)
        .collect()
}

fn is_relevant(context: &MemoryContext, terms: &[String]) -> bool {
    if terms.is_empty() {
        return true;
    }
    let haystack = format!("{} {}", context.key, context.summary).to_ascii_lowercase();
    terms.iter().any(|term| haystack.contains(term))
}

pub fn tenant_id() -> String {
    std::env::var("ONUS_TENANT_ID").unwrap_or_else(|_| "local".to_string())
}

pub fn project_id(workspace_root: &str) -> String {
    std::env::var("ONUS_PROJECT_ID")
        .unwrap_or_else(|_| security::sha256_hex(&format!("project:{}", workspace_root)))
}

fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("onus-memory-{}-{}.db", name, uuid::Uuid::new_v4()))
    }

    fn scope(project: &str, session: Option<&str>) -> MemoryScope {
        MemoryScope {
            tenant_id: "tenant-a".to_string(),
            project_id: project.to_string(),
            session_id: session.map(|s| s.to_string()),
        }
    }

    fn input(kind: MemoryKind, scope: MemoryScope, key: &str, value: Value) -> MemoryInput {
        MemoryInput {
            kind,
            scope,
            key: key.to_string(),
            value,
            sensitive: false,
            provenance: MemoryProvenance {
                actor_type: "system".to_string(),
                actor_id: "test".to_string(),
                source: "unit-test".to_string(),
                reason: "prove memory behavior".to_string(),
            },
            retention_days: Some(30),
            review_status: "system_recorded".to_string(),
        }
    }

    #[test]
    fn session_memory_is_isolated() {
        let path = temp_db("session-isolation");
        let mut store = MemoryStore::open(&path).unwrap();
        store
            .put(input(
                MemoryKind::Session,
                scope("project-a", Some("s1")),
                "auth_bug",
                serde_json::json!({"note": "login retry bug"}),
            ))
            .unwrap();
        store
            .put(input(
                MemoryKind::Session,
                scope("project-a", Some("s2")),
                "billing_bug",
                serde_json::json!({"note": "invoice bug"}),
            ))
            .unwrap();

        let got = store
            .retrieve_relevant(&scope("project-a", Some("s1")), "login", 10)
            .unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].key, "auth_bug");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn project_memory_is_isolated() {
        let path = temp_db("project-isolation");
        let mut store = MemoryStore::open(&path).unwrap();
        store
            .put(input(
                MemoryKind::Project,
                scope("project-a", None),
                "auth_architecture",
                serde_json::json!({"note": "auth uses sessions"}),
            ))
            .unwrap();
        store
            .put(input(
                MemoryKind::Project,
                scope("project-b", None),
                "auth_architecture",
                serde_json::json!({"note": "auth uses tokens"}),
            ))
            .unwrap();

        let got = store
            .retrieve_relevant(&scope("project-a", Some("s1")), "auth", 10)
            .unwrap();
        assert_eq!(got.len(), 1);
        assert!(got[0].summary.contains("sessions"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn secrets_are_redacted_and_sensitive_payload_is_encrypted() {
        let path = temp_db("redaction");
        let mut store = MemoryStore::open(&path).unwrap();
        store
            .put(MemoryInput {
                sensitive: true,
                ..input(
                    MemoryKind::Project,
                    scope("project-a", None),
                    "deployment_note",
                    serde_json::json!({"content": "AWS_SECRET_ACCESS_KEY=\"abc123\""}),
                )
            })
            .unwrap();
        let raw = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(!raw.contains("abc123"));

        let conn = Connection::open(&path).unwrap();
        let row: (String, String) = conn
            .query_row(
                "SELECT value_redacted, value_ciphertext FROM onus_memory LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert!(row.0.contains(security::REDACTED));
        assert!(!row.1.contains(security::REDACTED));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn policy_memory_cannot_be_changed_silently_by_agent() {
        let path = temp_db("policy");
        let mut store = MemoryStore::open(&path).unwrap();
        let result = store.put(MemoryInput {
            kind: MemoryKind::Policy,
            provenance: MemoryProvenance {
                actor_type: "agent".to_string(),
                actor_id: "agent-1".to_string(),
                source: "runtime".to_string(),
                reason: "silently weaken rule".to_string(),
            },
            review_status: "proposed".to_string(),
            ..input(
                MemoryKind::Policy,
                scope("project-a", None),
                "allow_prod",
                serde_json::json!({"policy": "allow production writes"}),
            )
        });
        assert!(result.is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn deletion_hides_memory_from_retrieval() {
        let path = temp_db("delete");
        let mut store = MemoryStore::open(&path).unwrap();
        store
            .put(input(
                MemoryKind::Project,
                scope("project-a", None),
                "auth_architecture",
                serde_json::json!({"note": "auth uses sessions"}),
            ))
            .unwrap();
        assert_eq!(
            store
                .retrieve_relevant(&scope("project-a", Some("s1")), "auth", 10)
                .unwrap()
                .len(),
            1
        );
        store.delete_scope("tenant-a", "project-a", None).unwrap();
        assert!(store
            .retrieve_relevant(&scope("project-a", Some("s1")), "auth", 10)
            .unwrap()
            .is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn provenance_and_versioning_are_reviewable() {
        let path = temp_db("provenance");
        let mut store = MemoryStore::open(&path).unwrap();
        store
            .put(input(
                MemoryKind::Project,
                scope("project-a", None),
                "architecture",
                serde_json::json!({"note": "v1"}),
            ))
            .unwrap();
        store
            .put(input(
                MemoryKind::Project,
                scope("project-a", None),
                "architecture",
                serde_json::json!({"note": "v2"}),
            ))
            .unwrap();
        let got = store
            .retrieve_relevant(&scope("project-a", Some("s1")), "architecture", 10)
            .unwrap();
        assert!(got.iter().any(|item| item.version == 2));
        assert_eq!(got[0].provenance.source, "unit-test");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn only_relevant_memory_reaches_reviewer_context() {
        let path = temp_db("relevance");
        let mut store = MemoryStore::open(&path).unwrap();
        store
            .put(input(
                MemoryKind::Project,
                scope("project-a", None),
                "auth_architecture",
                serde_json::json!({"note": "login controller owns auth sessions"}),
            ))
            .unwrap();
        store
            .put(input(
                MemoryKind::Project,
                scope("project-a", None),
                "billing_architecture",
                serde_json::json!({"note": "invoices use billing service"}),
            ))
            .unwrap();
        let got = store
            .retrieve_relevant(&scope("project-a", Some("s1")), "fix login auth", 10)
            .unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].key, "auth_architecture");
        let _ = std::fs::remove_file(path);
    }
}
