//! Audit trail — SQLite-backed tamper-evident record of each governed action.
//! Each action is hash-chained for tamper evidence; sessions track summary stats.
//!
//! Schema:
//!   actions(id INTEGER PRIMARY KEY, action_id TEXT UNIQUE, session_id TEXT,
//!           sequence INTEGER, action_type TEXT, payload TEXT, tool_name TEXT,
//!           verdict TEXT, rule_id TEXT, correction TEXT, prev_hash TEXT,
//!           hash TEXT, created_at TEXT)
//!   sessions(session_id TEXT PRIMARY KEY, agent_name TEXT, agent_version TEXT,
//!            task_description TEXT, workspace_root TEXT, status TEXT,
//!            started_at TEXT, ended_at TEXT, total_actions INTEGER DEFAULT 0,
//!            blocked_actions INTEGER DEFAULT 0, escalated_actions INTEGER DEFAULT 0)

use crate::quality::FindingSeverity;
use crate::security::{self, ApprovalBinding};
use crate::task_contract::{CompletionEvidence, CompletionStatus, TaskContract};
use crate::Verdict;
use rusqlite::{params, Connection};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single recorded action.
#[derive(Debug, Clone)]
pub struct ActionRecord {
    pub id: String,
    pub session_id: String,
    pub sequence: u64,
    pub action_type: String,
    pub tool_name: Option<String>,
    pub payload: String,
    pub payload_hash: String,
    pub payload_classification: String,
    pub verdict: String,
    pub rule_id: Option<String>,
    pub correction: Option<String>,
    pub approval_decision: Option<String>,
    pub guardian_mode: Option<String>,
    pub obligations_json: Option<String>,
    pub approval_reason: Option<String>,
    pub prev_hash: String,
    pub hash: String,
    pub created_at: String,
}

/// A session summary.
#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub id: String,
    pub agent_name: String,
    pub agent_version: Option<String>,
    pub task_description: String,
    pub workspace_root: String,
    pub status: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub total_actions: u64,
    pub blocked_actions: u64,
    pub escalated_actions: u64,
}

/// Daemon status summary.
#[derive(Debug, Clone)]
pub struct StatusSummary {
    pub total_actions: u64,
    pub blocked_actions: u64,
    pub escalated_actions: u64,
}

pub struct AuditTrail {
    conn: Connection,
}

fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn now_iso8601() -> String {
    let secs = now_timestamp();
    // Simple ISO 8601 without external dep
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Days since epoch to year/month/day (simplified)
    let mut y = 1970i64;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let months_days: &[i64] = if is_leap(y) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1i64;
    for &md in months_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

impl AuditTrail {
    /// Open (or create) the SQLite database at the given path.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        // If path is a directory (legacy JSONL layout), remove it first
        if path.is_dir() {
            std::fs::remove_dir_all(path)
                .map_err(|e| anyhow::anyhow!("Failed to remove legacy audit dir: {}", e))?;
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        // Enable WAL mode for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Create tables
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                agent_name TEXT NOT NULL,
                agent_version TEXT,
                task_description TEXT NOT NULL,
                workspace_root TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                started_at INTEGER NOT NULL,
                ended_at INTEGER,
                total_actions INTEGER NOT NULL DEFAULT 0,
                blocked_actions INTEGER NOT NULL DEFAULT 0,
                escalated_actions INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS actions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                action_id TEXT NOT NULL UNIQUE,
                session_id TEXT NOT NULL REFERENCES sessions(session_id),
                sequence INTEGER NOT NULL,
                action_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                payload_hash TEXT NOT NULL DEFAULT '',
                payload_classification TEXT NOT NULL DEFAULT '{}',
                tool_name TEXT,
                verdict TEXT NOT NULL,
                rule_id TEXT,
                correction TEXT,
                approval_decision TEXT,
                guardian_mode TEXT,
                obligations_json TEXT,
                approval_reason TEXT,
                prev_hash TEXT NOT NULL DEFAULT '',
                hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                created_at_unix INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_actions_session_id ON actions(session_id);
            CREATE INDEX IF NOT EXISTS idx_actions_created_at ON actions(created_at_unix);
            CREATE INDEX IF NOT EXISTS idx_actions_verdict ON actions(verdict);

            CREATE TABLE IF NOT EXISTS task_contracts (
                session_id TEXT PRIMARY KEY,
                contract_hash TEXT NOT NULL,
                contract_json TEXT NOT NULL,
                original_prompt TEXT NOT NULL,
                normalized_objective TEXT NOT NULL,
                environment_identity TEXT NOT NULL,
                policy_version TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                completed_at INTEGER,
                completion_status TEXT
            );

            CREATE TABLE IF NOT EXISTS pending_approvals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                action_id TEXT NOT NULL UNIQUE,
                session_id TEXT NOT NULL,
                action_type TEXT NOT NULL,
                tool_name TEXT,
                payload TEXT NOT NULL,
                canonical_payload_hash TEXT NOT NULL DEFAULT '',
                task_contract_hash TEXT NOT NULL DEFAULT '',
                policy_version TEXT NOT NULL DEFAULT '',
                environment_identity TEXT NOT NULL DEFAULT '',
                expires_at INTEGER NOT NULL DEFAULT 0,
                approver TEXT,
                rule_id TEXT NOT NULL,
                rule_name TEXT NOT NULL,
                correction TEXT NOT NULL,
                approval_decision TEXT,
                guardian_mode TEXT,
                obligations_json TEXT,
                approval_reason TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at INTEGER NOT NULL,
                resolved_at INTEGER
            );

            CREATE INDEX IF NOT EXISTS idx_pending_status ON pending_approvals(status);
            ",
        )?;
        ensure_column(&conn, "actions", "payload_hash", "TEXT NOT NULL DEFAULT ''")?;
        ensure_column(
            &conn,
            "actions",
            "payload_classification",
            "TEXT NOT NULL DEFAULT '{}'",
        )?;
        ensure_column(&conn, "actions", "approval_decision", "TEXT")?;
        ensure_column(&conn, "actions", "guardian_mode", "TEXT")?;
        ensure_column(&conn, "actions", "obligations_json", "TEXT")?;
        ensure_column(&conn, "actions", "approval_reason", "TEXT")?;
        ensure_column(
            &conn,
            "pending_approvals",
            "canonical_payload_hash",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        ensure_column(
            &conn,
            "pending_approvals",
            "task_contract_hash",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        ensure_column(
            &conn,
            "pending_approvals",
            "policy_version",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        ensure_column(
            &conn,
            "pending_approvals",
            "environment_identity",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        ensure_column(
            &conn,
            "pending_approvals",
            "expires_at",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        ensure_column(&conn, "pending_approvals", "approver", "TEXT")?;
        ensure_column(&conn, "pending_approvals", "approval_decision", "TEXT")?;
        ensure_column(&conn, "pending_approvals", "guardian_mode", "TEXT")?;
        ensure_column(&conn, "pending_approvals", "obligations_json", "TEXT")?;
        ensure_column(&conn, "pending_approvals", "approval_reason", "TEXT")?;
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_pending_binding
             ON pending_approvals(session_id, action_id, canonical_payload_hash, task_contract_hash, policy_version, environment_identity, status);",
        )?;
        crate::memory::ensure_schema(&conn)?;

        Ok(Self { conn })
    }

    /// Start a new agent session.
    pub fn start_session(
        &mut self,
        session_id: &str,
        agent_name: &str,
        agent_version: Option<&str>,
        task_description: &str,
        workspace_root: &str,
    ) -> anyhow::Result<()> {
        let now = now_timestamp();
        self.conn.execute(
            "INSERT OR REPLACE INTO sessions
                (session_id, agent_name, agent_version, task_description,
                 workspace_root, status, started_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'active', ?6)",
            params![
                session_id,
                agent_name,
                agent_version,
                task_description,
                workspace_root,
                now
            ],
        )?;
        Ok(())
    }

    pub fn remember_session_intake(
        &mut self,
        session_id: &str,
        workspace_root: &str,
        original_prompt: &str,
        normalized_objective: &str,
    ) -> anyhow::Result<()> {
        crate::memory::remember_session_intake(
            &self.conn,
            crate::memory::MemoryScope::for_workspace(workspace_root, Some(session_id.to_string())),
            original_prompt,
            normalized_objective,
        )
    }

    pub fn relevant_memory_context(
        &self,
        workspace_root: &str,
        session_id: Option<&str>,
        query: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<crate::memory::MemoryContext>> {
        crate::memory::retrieve_relevant_conn(
            &self.conn,
            &crate::memory::MemoryScope::for_workspace(
                workspace_root,
                session_id.map(|s| s.to_string()),
            ),
            query,
            limit,
        )
    }

    pub fn remember_incident(
        &mut self,
        session_id: &str,
        workspace_root: &str,
        key: &str,
        value: serde_json::Value,
    ) -> anyhow::Result<()> {
        crate::memory::remember_incident(
            &self.conn,
            crate::memory::MemoryScope::for_workspace(workspace_root, Some(session_id.to_string())),
            key,
            value,
        )
    }

    /// End a session.
    pub fn end_session(&mut self, session_id: &str) -> anyhow::Result<()> {
        let now = now_timestamp();
        self.conn.execute(
            "UPDATE sessions SET status = 'completed', ended_at = ?1 WHERE session_id = ?2",
            params![now, session_id],
        )?;
        Ok(())
    }

    /// Update session status.
    pub fn update_session_status(&mut self, session_id: &str, status: &str) -> anyhow::Result<()> {
        self.conn.execute(
            "UPDATE sessions SET status = ?1 WHERE session_id = ?2",
            params![status, session_id],
        )?;
        Ok(())
    }

    /// Record a single action in the audit trail with hash chaining.
    #[allow(clippy::too_many_arguments)]
    pub fn record_action(
        &mut self,
        session_id: &str,
        sequence: u64,
        action_type: &str,
        tool_name: &str,
        payload: &str,
        verdict: &Verdict,
        rule_id: Option<&str>,
        correction: Option<&str>,
        _latency_us: u64,
    ) -> anyhow::Result<(String, String)> {
        self.record_action_with_broker_decision(
            session_id,
            sequence,
            action_type,
            tool_name,
            payload,
            verdict,
            rule_id,
            correction,
            _latency_us,
            None,
            None,
            &[],
            None,
        )
    }

    /// Record a single action with Approval Decision Broker metadata.
    #[allow(clippy::too_many_arguments)]
    pub fn record_action_with_broker_decision(
        &mut self,
        session_id: &str,
        sequence: u64,
        action_type: &str,
        tool_name: &str,
        payload: &str,
        verdict: &Verdict,
        rule_id: Option<&str>,
        correction: Option<&str>,
        _latency_us: u64,
        approval_decision: Option<&str>,
        guardian_mode: Option<&str>,
        obligations: &[String],
        approval_reason: Option<&str>,
    ) -> anyhow::Result<(String, String)> {
        // Generate a unique action ID
        use uuid::Uuid;
        let action_id = Uuid::new_v4().to_string();
        let now = now_timestamp();
        let now_iso = now_iso8601();

        // Auto-create session if it doesn't exist
        let session_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE session_id = ?1",
                params![session_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;

        if !session_exists {
            self.conn.execute(
                "INSERT OR IGNORE INTO sessions
                    (session_id, agent_name, task_description, workspace_root, status, started_at)
                 VALUES (?1, 'onus-cli', 'Direct evaluation via onus evaluate', 'unknown', 'active', ?2)",
                params![session_id, now_timestamp()],
            )?;
        }

        // Get the previous action's hash for chaining
        let prev_hash = self
            .conn
            .query_row(
                "SELECT hash FROM actions WHERE session_id = ?1 ORDER BY id DESC LIMIT 1",
                params![session_id],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_default();

        let classified = security::classify_payload_str(payload);
        let stored_payload = classified.redacted;
        let payload_hash = classified.payload_hash;
        let payload_classification = classified.classification;
        let verdict_str = format!("{}", verdict);

        // Compute hash: action_id|session_id|sequence|action_type|redacted_payload|verdict|prev_hash
        let hash_input = format!(
            "{}|{}|{}|{}|{}|{}|{}",
            action_id, session_id, sequence, action_type, stored_payload, verdict_str, prev_hash
        );
        let hash = hex::encode(Sha256::digest(hash_input.as_bytes()));

        let obligations_json = if obligations.is_empty() {
            None
        } else {
            Some(serde_json::to_string(obligations)?)
        };

        self.conn.execute(
            "INSERT INTO actions
                (action_id, session_id, sequence, action_type, payload, payload_hash,
                 payload_classification, tool_name,
                 verdict, rule_id, correction, approval_decision, guardian_mode,
                 obligations_json, approval_reason, prev_hash, hash, created_at, created_at_unix)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                action_id,
                session_id,
                sequence,
                action_type,
                stored_payload,
                payload_hash,
                payload_classification,
                tool_name,
                verdict_str,
                rule_id,
                correction,
                approval_decision,
                guardian_mode,
                obligations_json,
                approval_reason,
                prev_hash,
                hash,
                now_iso,
                now
            ],
        )?;

        // Update session counters
        let increment = match verdict {
            Verdict::Block => "blocked_actions = blocked_actions + 1",
            Verdict::Escalate => "escalated_actions = escalated_actions + 1",
            _ => "",
        };
        self.conn.execute(
            &format!(
                "UPDATE sessions SET total_actions = total_actions + 1 {} WHERE session_id = ?1",
                if increment.is_empty() {
                    String::new()
                } else {
                    format!(", {}", increment)
                }
            ),
            params![session_id],
        )?;

        Ok((action_id, payload_hash))
    }

    /// Get recent actions across all sessions.
    pub fn get_recent_actions(&self, limit: u32) -> anyhow::Result<Vec<ActionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT action_id, session_id, sequence, action_type, payload, payload_hash,
                    payload_classification, tool_name,
                    verdict, rule_id, correction, approval_decision, guardian_mode,
                    obligations_json, approval_reason, prev_hash, hash, created_at
             FROM actions ORDER BY id DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            Ok(ActionRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                sequence: row.get::<_, i64>(2)? as u64,
                action_type: row.get(3)?,
                payload: row.get(4)?,
                payload_hash: row.get(5)?,
                payload_classification: row.get(6)?,
                tool_name: row.get(7)?,
                verdict: row.get(8)?,
                rule_id: row.get(9)?,
                correction: row.get(10)?,
                approval_decision: row.get(11)?,
                guardian_mode: row.get(12)?,
                obligations_json: row.get(13)?,
                approval_reason: row.get(14)?,
                prev_hash: row.get(15)?,
                hash: row.get(16)?,
                created_at: row.get(17)?,
            })
        })?;

        let mut actions = Vec::new();
        for row in rows {
            actions.push(row?);
        }
        Ok(actions)
    }

    /// Get actions for a specific session.
    pub fn get_session_actions(&self, session_id: &str) -> anyhow::Result<Vec<ActionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT action_id, session_id, sequence, action_type, payload, payload_hash,
                    payload_classification, tool_name,
                    verdict, rule_id, correction, approval_decision, guardian_mode,
                    obligations_json, approval_reason, prev_hash, hash, created_at
             FROM actions WHERE session_id = ?1 ORDER BY id ASC",
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            Ok(ActionRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                sequence: row.get::<_, i64>(2)? as u64,
                action_type: row.get(3)?,
                payload: row.get(4)?,
                payload_hash: row.get(5)?,
                payload_classification: row.get(6)?,
                tool_name: row.get(7)?,
                verdict: row.get(8)?,
                rule_id: row.get(9)?,
                correction: row.get(10)?,
                approval_decision: row.get(11)?,
                guardian_mode: row.get(12)?,
                obligations_json: row.get(13)?,
                approval_reason: row.get(14)?,
                prev_hash: row.get(15)?,
                hash: row.get(16)?,
                created_at: row.get(17)?,
            })
        })?;

        let mut actions = Vec::new();
        for row in rows {
            actions.push(row?);
        }
        Ok(actions)
    }

    /// Get a session by ID.
    pub fn get_session(&self, session_id: &str) -> anyhow::Result<Option<SessionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_name, agent_version, task_description,
                    workspace_root, status, started_at, ended_at,
                    total_actions, blocked_actions, escalated_actions
             FROM sessions WHERE session_id = ?1",
        )?;

        let mut rows = stmt.query_map(params![session_id], |row| {
            Ok(SessionRecord {
                id: row.get(0)?,
                agent_name: row.get(1)?,
                agent_version: row.get(2)?,
                task_description: row.get(3)?,
                workspace_root: row.get(4)?,
                status: row.get(5)?,
                started_at: row.get(6)?,
                ended_at: row.get(7)?,
                total_actions: row.get::<_, i64>(8)? as u64,
                blocked_actions: row.get::<_, i64>(9)? as u64,
                escalated_actions: row.get::<_, i64>(10)? as u64,
            })
        })?;

        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            _ => Ok(None),
        }
    }

    /// List all sessions.
    pub fn list_sessions(&self) -> anyhow::Result<Vec<SessionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_name, agent_version, task_description,
                    workspace_root, status, started_at, ended_at,
                    total_actions, blocked_actions, escalated_actions
             FROM sessions ORDER BY started_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(SessionRecord {
                id: row.get(0)?,
                agent_name: row.get(1)?,
                agent_version: row.get(2)?,
                task_description: row.get(3)?,
                workspace_root: row.get(4)?,
                status: row.get(5)?,
                started_at: row.get(6)?,
                ended_at: row.get(7)?,
                total_actions: row.get::<_, i64>(8)? as u64,
                blocked_actions: row.get::<_, i64>(9)? as u64,
                escalated_actions: row.get::<_, i64>(10)? as u64,
            })
        })?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    /// Get daemon status summary.
    pub fn get_status(&self) -> anyhow::Result<StatusSummary> {
        let total_actions: i64 = self
            .conn
            .query_row("SELECT COALESCE(COUNT(*), 0) FROM actions", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        let blocked_actions: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(COUNT(*), 0) FROM actions WHERE verdict = 'block'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let escalated_actions: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(COUNT(*), 0) FROM actions WHERE verdict = 'escalate'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(StatusSummary {
            total_actions: total_actions as u64,
            blocked_actions: blocked_actions as u64,
            escalated_actions: escalated_actions as u64,
        })
    }

    /// Verify the hash chain for a session. Returns list of action IDs where hash doesn't match.
    pub fn verify_chain(&self, session_id: &str) -> anyhow::Result<Vec<(String, String)>> {
        let actions = self.get_session_actions(session_id)?;
        let mut bad: Vec<(String, String)> = Vec::new();
        let mut expected_prev = String::new();

        for action in &actions {
            // Recompute expected hash
            let hash_input = format!(
                "{}|{}|{}|{}|{}|{}|{}",
                action.id,
                action.session_id,
                action.sequence,
                action.action_type,
                action.payload,
                action.verdict,
                expected_prev
            );
            let expected_hash = hex::encode(Sha256::digest(hash_input.as_bytes()));

            if action.hash != expected_hash {
                bad.push((
                    action.id.clone(),
                    format!(
                        "hash mismatch: stored={}, expected={}",
                        action.hash, expected_hash
                    ),
                ));
            }

            if action.prev_hash != expected_prev {
                bad.push((
                    action.id.clone(),
                    format!(
                        "prev_hash mismatch: stored={}, expected={}",
                        action.prev_hash, expected_prev
                    ),
                ));
            }

            expected_prev = action.hash.clone();
        }

        Ok(bad)
    }

    /// Verify hash chain for ALL actions (session-agnostic).
    pub fn verify_all_actions(&self) -> anyhow::Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT action_id, session_id, sequence, action_type, payload, verdict, prev_hash, hash
             FROM actions ORDER BY id ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)? as u64,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })?;

        let mut bad: Vec<(String, String)> = Vec::new();
        let mut expected_prev = String::new();

        for row in rows {
            let (action_id, session_id, sequence, action_type, payload, verdict, prev_hash, hash) =
                row?;
            let hash_input = format!(
                "{}|{}|{}|{}|{}|{}|{}",
                action_id, session_id, sequence, action_type, payload, verdict, expected_prev
            );
            let expected_hash = hex::encode(Sha256::digest(hash_input.as_bytes()));

            if hash != expected_hash {
                bad.push((
                    action_id.clone(),
                    format!("hash mismatch: stored={}, expected={}", hash, expected_hash),
                ));
            }
            if prev_hash != expected_prev {
                bad.push((
                    action_id.clone(),
                    format!(
                        "prev_hash mismatch: stored={}, expected={}",
                        prev_hash, expected_prev
                    ),
                ));
            }

            expected_prev = hash;
        }

        Ok(bad)
    }

    /// Get the total count of actions.
    pub fn action_count(&self) -> anyhow::Result<u64> {
        let count: i64 =
            self.conn
                .query_row("SELECT COALESCE(COUNT(*), 0) FROM actions", [], |row| {
                    row.get(0)
                })?;
        Ok(count as u64)
    }

    pub fn save_task_contract(&mut self, contract: &TaskContract) -> anyhow::Result<TaskContract> {
        let finalized = contract.clone().finalized();
        if !finalized.verify_hash() {
            anyhow::bail!("Task contract canonical hash verification failed");
        }
        let now = now_timestamp();
        let contract_json = serde_json::to_string(&finalized)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO task_contracts
                (session_id, contract_hash, contract_json, original_prompt, normalized_objective,
                 environment_identity, policy_version, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                finalized.session_id,
                finalized.canonical_hash,
                contract_json,
                finalized.original_prompt,
                finalized.normalized_objective,
                finalized.environment_identity,
                finalized.policy_version,
                now
            ],
        )?;
        Ok(finalized)
    }

    pub fn get_task_contract(&self, session_id: &str) -> anyhow::Result<Option<TaskContract>> {
        let json: Option<String> = self
            .conn
            .query_row(
                "SELECT contract_json FROM task_contracts WHERE session_id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .ok();
        match json {
            Some(raw) => {
                let contract: TaskContract = serde_json::from_str(&raw)?;
                if !contract.verify_hash() {
                    anyhow::bail!("Stored task contract hash does not match canonical content");
                }
                Ok(Some(contract))
            }
            None => Ok(None),
        }
    }

    pub fn session_touched_paths(&self, session_id: &str) -> anyhow::Result<Vec<String>> {
        let actions = self.get_session_actions(session_id)?;
        let mut paths = Vec::new();
        for action in actions {
            let payload: serde_json::Value =
                serde_json::from_str(&action.payload).unwrap_or(serde_json::Value::Null);
            let action_type = match action.action_type.as_str() {
                "file_write" => crate::ActionType::FileWrite,
                "file_delete" => crate::ActionType::FileDelete,
                "file_read" => crate::ActionType::FileRead,
                "db_mutation" => crate::ActionType::DbMutation,
                _ => crate::ActionType::Shell,
            };
            let synthetic = crate::ipc::Action {
                action_type,
                tool: action.tool_name.unwrap_or_default(),
                payload,
            };
            if let Some(path) = crate::task_contract::extract_action_path(&synthetic) {
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
        }
        Ok(paths)
    }

    pub fn complete_task_contract(
        &mut self,
        session_id: &str,
        evidence: &[CompletionEvidence],
    ) -> anyhow::Result<CompletionStatus> {
        let contract = self
            .get_task_contract(session_id)?
            .ok_or_else(|| anyhow::anyhow!("No task contract found for session {}", session_id))?;
        let session = self.get_session(session_id)?;
        let workspace = session
            .as_ref()
            .map(|s| std::path::PathBuf::from(&s.workspace_root))
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let actions = self.get_session_actions(session_id)?;
        let pending_approvals = self
            .get_pending_approvals()?
            .into_iter()
            .filter(|approval| approval.session_id == session_id)
            .collect::<Vec<_>>();
        let verification = crate::quality::verify_completion(
            &contract,
            &workspace,
            &actions,
            &pending_approvals,
            evidence,
        );
        let status = completion_status_from_verification(
            session.as_ref().map(|s| s.status.as_str()),
            verification,
        );
        let status_text = status.as_str().to_string();
        self.conn.execute(
            "UPDATE task_contracts
             SET completed_at = ?1, completion_status = ?2
             WHERE session_id = ?3",
            params![now_timestamp(), status_text, session_id],
        )?;
        Ok(status)
    }

    #[cfg(test)]
    pub fn break_actions_table_for_test(&mut self) -> anyhow::Result<()> {
        self.conn.execute("DROP TABLE actions", [])?;
        Ok(())
    }

    // --- Approval system (Phase 5) ---

    /// Create a pending approval record for an escalated action.
    #[allow(clippy::too_many_arguments)]
    pub fn create_pending_approval(
        &mut self,
        binding: &ApprovalBinding,
        action_type: &str,
        tool_name: Option<&str>,
        payload: &str,
        rule_id: &str,
        rule_name: &str,
        correction: &str,
    ) -> anyhow::Result<()> {
        self.create_pending_approval_with_broker_decision(
            binding,
            action_type,
            tool_name,
            payload,
            rule_id,
            rule_name,
            correction,
            None,
            None,
            &[],
            None,
        )
    }

    /// Create a pending approval record with Approval Decision Broker metadata.
    #[allow(clippy::too_many_arguments)]
    pub fn create_pending_approval_with_broker_decision(
        &mut self,
        binding: &ApprovalBinding,
        action_type: &str,
        tool_name: Option<&str>,
        payload: &str,
        rule_id: &str,
        rule_name: &str,
        correction: &str,
        approval_decision: Option<&str>,
        guardian_mode: Option<&str>,
        obligations: &[String],
        approval_reason: Option<&str>,
    ) -> anyhow::Result<()> {
        let now = now_timestamp();
        let redacted_payload = security::classify_payload_str(payload).redacted;
        let obligations_json = if obligations.is_empty() {
            None
        } else {
            Some(serde_json::to_string(obligations)?)
        };
        self.conn.execute(
            "INSERT OR IGNORE INTO pending_approvals
                (action_id, session_id, action_type, tool_name, payload,
                 canonical_payload_hash, task_contract_hash, policy_version,
                 environment_identity, expires_at, rule_id, rule_name, correction,
                 approval_decision, guardian_mode, obligations_json, approval_reason, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, 'pending', ?18)",
            params![
                binding.action_id,
                binding.session_id,
                action_type,
                tool_name,
                redacted_payload,
                binding.canonical_payload_hash,
                binding.task_contract_hash,
                binding.policy_version,
                binding.environment_identity,
                binding.expires_at,
                rule_id,
                rule_name,
                correction,
                approval_decision,
                guardian_mode,
                obligations_json,
                approval_reason,
                now
            ],
        )?;
        Ok(())
    }

    /// Approve a pending action.
    pub fn approve_action(&mut self, action_id: &str, approver: &str) -> anyhow::Result<bool> {
        let now = now_timestamp();
        let rows = self.conn.execute(
            "UPDATE pending_approvals
             SET status = 'approved', resolved_at = ?1, approver = ?2
             WHERE action_id = ?3 AND status = 'pending' AND expires_at > ?1",
            params![now, approver, action_id],
        )?;
        Ok(rows > 0)
    }

    /// Reject a pending action.
    pub fn reject_action(&mut self, action_id: &str) -> anyhow::Result<bool> {
        let now = now_timestamp();
        let rows = self.conn.execute(
            "UPDATE pending_approvals SET status = 'rejected', resolved_at = ?1 WHERE action_id = ?2 AND status = 'pending'",
            params![now, action_id],
        )?;
        Ok(rows > 0)
    }

    /// Check if an action has been approved.
    pub fn is_action_approved(&self, action_id: &str) -> anyhow::Result<bool> {
        let status: Option<String> = self
            .conn
            .query_row(
                "SELECT status FROM pending_approvals WHERE action_id = ?1",
                params![action_id],
                |row| row.get(0),
            )
            .ok();
        Ok(status.as_deref() == Some("approved"))
    }

    pub fn find_approved_approval(
        &self,
        binding: &ApprovalBinding,
    ) -> anyhow::Result<Option<PendingApproval>> {
        let now = now_timestamp();
        let mut stmt = self.conn.prepare(
            "SELECT id, action_id, session_id, action_type, tool_name, payload,
                    canonical_payload_hash, task_contract_hash, policy_version,
                    environment_identity, expires_at, approver,
                    rule_id, rule_name, correction, approval_decision, guardian_mode,
                    obligations_json, approval_reason, status, created_at, resolved_at
             FROM pending_approvals
             WHERE session_id = ?1
               AND action_id = ?2
               AND canonical_payload_hash = ?3
               AND task_contract_hash = ?4
               AND policy_version = ?5
               AND environment_identity = ?6
               AND status = 'approved'
               AND expires_at > ?7
             ORDER BY resolved_at DESC
             LIMIT 1",
        )?;

        let mut rows = stmt.query_map(
            params![
                binding.session_id,
                binding.action_id,
                binding.canonical_payload_hash,
                binding.task_contract_hash,
                binding.policy_version,
                binding.environment_identity,
                now
            ],
            pending_from_row,
        )?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// Get all pending approvals.
    pub fn get_pending_approvals(&self) -> anyhow::Result<Vec<PendingApproval>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, action_id, session_id, action_type, tool_name, payload,
                    canonical_payload_hash, task_contract_hash, policy_version,
                    environment_identity, expires_at, approver,
                    rule_id, rule_name, correction, approval_decision, guardian_mode,
                    obligations_json, approval_reason, status, created_at, resolved_at
             FROM pending_approvals ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], pending_from_row)?;

        let mut approvals = Vec::new();
        for row in rows {
            approvals.push(row?);
        }
        Ok(approvals)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PendingApproval {
    pub id: i64,
    pub action_id: String,
    pub session_id: String,
    pub action_type: String,
    pub tool_name: Option<String>,
    pub payload: String,
    pub canonical_payload_hash: String,
    pub task_contract_hash: String,
    pub policy_version: String,
    pub environment_identity: String,
    pub expires_at: i64,
    pub approver: Option<String>,
    pub rule_id: String,
    pub rule_name: String,
    pub correction: String,
    pub approval_decision: Option<String>,
    pub guardian_mode: Option<String>,
    pub obligations_json: Option<String>,
    pub approval_reason: Option<String>,
    pub status: String,
    pub created_at: Option<i64>,
    pub resolved_at: Option<i64>,
}

fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> anyhow::Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for existing in columns {
        if existing? == column {
            return Ok(());
        }
    }
    conn.execute(
        &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition),
        [],
    )?;
    Ok(())
}

fn pending_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PendingApproval> {
    Ok(PendingApproval {
        id: row.get(0)?,
        action_id: row.get(1)?,
        session_id: row.get(2)?,
        action_type: row.get(3)?,
        tool_name: row.get(4)?,
        payload: row.get(5)?,
        canonical_payload_hash: row.get(6)?,
        task_contract_hash: row.get(7)?,
        policy_version: row.get(8)?,
        environment_identity: row.get(9)?,
        expires_at: row.get(10)?,
        approver: row.get(11)?,
        rule_id: row.get(12)?,
        rule_name: row.get(13)?,
        correction: row.get(14)?,
        approval_decision: row.get(15)?,
        guardian_mode: row.get(16)?,
        obligations_json: row.get(17)?,
        approval_reason: row.get(18)?,
        status: row.get(19)?,
        created_at: row.get(20)?,
        resolved_at: row.get(21)?,
    })
}

impl Drop for AuditTrail {
    fn drop(&mut self) {
        let _ = self.conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", []);
    }
}

fn completion_status_from_verification(
    session_status: Option<&str>,
    verification: crate::quality::CompletionVerification,
) -> CompletionStatus {
    if session_status == Some("terminated") {
        return CompletionStatus::Terminated {
            findings: verification.findings,
        };
    }
    if session_status == Some("rolled_back") {
        return CompletionStatus::RolledBack {
            findings: verification.findings,
        };
    }

    if !verification.missing_evidence.is_empty() {
        return CompletionStatus::HumanReviewRequired {
            missing_evidence: verification.missing_evidence,
            findings: verification.findings,
        };
    }

    let has_critical = verification
        .findings
        .iter()
        .any(|finding| finding.severity == FindingSeverity::Critical);
    if has_critical {
        return CompletionStatus::FailedSafely {
            findings: verification.findings,
        };
    }

    let has_blocking = verification
        .findings
        .iter()
        .any(|finding| finding.severity == FindingSeverity::Blocking);
    if has_blocking {
        return CompletionStatus::HumanReviewRequired {
            missing_evidence: Vec::new(),
            findings: verification.findings,
        };
    }

    if !verification.findings.is_empty() {
        return CompletionStatus::CompletedWithExceptions {
            exceptions: verification
                .findings
                .into_iter()
                .map(|finding| finding.message)
                .collect(),
        };
    }

    CompletionStatus::CompletedVerified
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("onus-{}-{}.db", name, uuid::Uuid::new_v4()))
    }

    fn binding(payload_hash: &str) -> ApprovalBinding {
        ApprovalBinding {
            session_id: "session-1".to_string(),
            task_contract_hash: "task-hash".to_string(),
            action_id: security::approval_action_id("session-1", "Write", payload_hash),
            canonical_payload_hash: payload_hash.to_string(),
            policy_version: "policy-v1".to_string(),
            environment_identity: "env-1".to_string(),
            expires_at: security::now_unix() + 60,
            approver: None,
        }
    }

    #[test]
    fn record_action_redacts_payload_before_persistence() {
        let path = temp_db("redaction");
        let mut audit = AuditTrail::open(&path).unwrap();
        let payload = serde_json::json!({
            "path": "demo.txt",
            "token": "raw-token-value",
            "content": "AWS_SECRET_ACCESS_KEY=\"abc123\""
        })
        .to_string();

        let (_action_id, payload_hash) = audit
            .record_action(
                "session-1",
                1,
                "file_write",
                "Write",
                &payload,
                &Verdict::Allow,
                None,
                None,
                10,
            )
            .unwrap();

        let action = audit.get_session_actions("session-1").unwrap().remove(0);
        assert_eq!(action.payload_hash, payload_hash);
        assert!(!action.payload.contains("raw-token-value"));
        assert!(!action.payload.contains("abc123"));
        assert!(action.payload.contains(security::REDACTED));
        assert!(action
            .payload_classification
            .contains("\"contains_sensitive\":true"));
        assert!(audit.verify_chain("session-1").unwrap().is_empty());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn approval_lookup_requires_exact_binding_and_rejects_changed_payload() {
        let path = temp_db("approval-binding");
        let mut audit = AuditTrail::open(&path).unwrap();
        let approved = binding("payload-hash-1");

        audit
            .create_pending_approval(
                &approved,
                "mcp",
                Some("Write"),
                r#"{"content":"safe"}"#,
                "RULE_1",
                "needs-approval",
                "Approve this action",
            )
            .unwrap();
        assert!(audit.approve_action(&approved.action_id, "alice").unwrap());

        let exact = audit.find_approved_approval(&approved).unwrap().unwrap();
        assert_eq!(exact.approver.as_deref(), Some("alice"));

        let mut changed_payload = approved.clone();
        changed_payload.canonical_payload_hash = "payload-hash-2".to_string();
        changed_payload.action_id = security::approval_action_id(
            "session-1",
            "Write",
            &changed_payload.canonical_payload_hash,
        );
        assert!(audit
            .find_approved_approval(&changed_payload)
            .unwrap()
            .is_none());

        let mut changed_policy = approved.clone();
        changed_policy.policy_version = "policy-v2".to_string();
        assert!(audit
            .find_approved_approval(&changed_policy)
            .unwrap()
            .is_none());

        let mut changed_env = approved.clone();
        changed_env.environment_identity = "env-2".to_string();
        assert!(audit
            .find_approved_approval(&changed_env)
            .unwrap()
            .is_none());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn expired_approval_cannot_be_approved_or_reused() {
        let path = temp_db("expired-approval");
        let mut audit = AuditTrail::open(&path).unwrap();
        let mut expired = binding("payload-hash-expired");
        expired.expires_at = security::now_unix() - 1;

        audit
            .create_pending_approval(
                &expired,
                "mcp",
                Some("Write"),
                r#"{"content":"safe"}"#,
                "RULE_1",
                "needs-approval",
                "Approve this action",
            )
            .unwrap();
        assert!(!audit.approve_action(&expired.action_id, "alice").unwrap());
        assert!(audit.find_approved_approval(&expired).unwrap().is_none());

        let _ = std::fs::remove_file(path);
    }
}
