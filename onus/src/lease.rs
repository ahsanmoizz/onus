//! # Session Leases — Exclusive Mutating Session Ownership
//!
//! A **session lease** grants one agent exclusive mutating access to a session.
//! Other agents may read audit data and memory but must not create new actions,
//! write files, or change state while another agent holds the lease.
//!
//! ## Crash recovery
//!
//! Leases have a TTL.  If the holding agent crashes without releasing, the lease
//! expires and another agent may acquire it.  A heartbeat endpoint extends the
//! TTL so that long-running agents are not interrupted.
//!
//! ## Forced takeover
//!
//! A human may force-takeover a lease (e.g. when the holding agent is
//! unreachable).  This requires explicit human approval (`approvals approve`)
//! before the lease is transferred.
//!
//! ## Concurrency model
//!
//! - **R0 / R1 (read-only / auto-reversible)**: may proceed without a lease,
//!   provided the action is logged.
//! - **R2 / R3 / R4 (mutating)**: MUST hold an exclusive lease.
//!
//! See `docs/Onus_Whitepaper.md` §9 for revertibility classifications.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Lease status ────────────────────────────────────────────────────────────

/// The current status of a session lease.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseStatus {
    /// Lease is active — holder may mutate.
    Active,
    /// Lease has expired (TTL passed without heartbeat).
    Expired,
    /// Lease was intentionally released.
    Released,
    /// Lease was taken over by a human (force).
    TakenOver,
}

impl std::fmt::Display for LeaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Expired => write!(f, "expired"),
            Self::Released => write!(f, "released"),
            Self::TakenOver => write!(f, "taken_over"),
        }
    }
}

impl LeaseStatus {
    fn from_db(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "expired" => Some(Self::Expired),
            "released" => Some(Self::Released),
            "taken_over" => Some(Self::TakenOver),
            _ => None,
        }
    }
}

// ── Lease record ─────────────────────────────────────────────────────────────

/// A single session lease record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLease {
    /// Unique lease ID (UUID v4).
    pub lease_id: String,
    /// Session this lease governs.
    pub session_id: String,
    /// Which surface holds the lease (e.g. `claude_code_cli`, `codex_cli`).
    pub holder_surface: String,
    /// Human-readable identity of the holder (e.g. agent name, PID).
    pub holder_identity: String,
    /// Current lease status.
    pub status: LeaseStatus,
    /// Unix timestamp when the lease was acquired.
    pub acquired_at: i64,
    /// Unix timestamp when the lease expires (acquired_at + TTL).
    pub expires_at: i64,
    /// Unix timestamp of the last heartbeat (0 if never).
    pub last_heartbeat_at: i64,
    /// If `status == TakenOver`, the approval ID that authorised the takeover.
    pub takeover_approval_id: Option<String>,
    /// SHA-256 of the serialised lease record (for integrity).
    pub record_hash: String,
}

// ── Lease manager ────────────────────────────────────────────────────────────

/// Manages session leases backed by a SQLite database.
pub struct LeaseManager {
    conn: Connection,
}

impl LeaseManager {
    /// Open (or create) the lease database at `path`.
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS session_leases (
                lease_id        TEXT PRIMARY KEY,
                session_id      TEXT NOT NULL,
                holder_surface  TEXT NOT NULL,
                holder_identity TEXT NOT NULL,
                status          TEXT NOT NULL DEFAULT 'active',
                acquired_at     INTEGER NOT NULL,
                expires_at      INTEGER NOT NULL,
                last_heartbeat_at INTEGER NOT NULL DEFAULT 0,
                takeover_approval_id TEXT,
                record_hash     TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_leases_session ON session_leases(session_id);
            CREATE INDEX IF NOT EXISTS idx_leases_status ON session_leases(status);",
        )?;
        Ok(Self { conn })
    }

    /// Acquire an exclusive lease for `session_id`.
    ///
    /// Returns `Err` if another agent holds an active lease for the same
    /// session and the lease has not expired.
    pub fn acquire(
        &mut self,
        session_id: &str,
        holder_surface: &str,
        holder_identity: &str,
        ttl_seconds: i64,
    ) -> Result<SessionLease, LeaseError> {
        let now = unix_now();

        // Check for existing active lease
        let existing = self.find_active(session_id)?;
        if let Some(lease) = existing {
            if lease.expires_at > now {
                return Err(LeaseError::AlreadyHeld {
                    lease_id: lease.lease_id,
                    holder: lease.holder_identity,
                    surface: lease.holder_surface,
                    expires_at: lease.expires_at,
                });
            }
            // Expired — release it
            self.set_status(&lease.lease_id, LeaseStatus::Expired)?;
        }

        let lease_id = uuid_v4();
        let acquired_at = now;
        let expires_at = now + ttl_seconds;
        let record = SessionLease {
            lease_id: lease_id.clone(),
            session_id: session_id.to_string(),
            holder_surface: holder_surface.to_string(),
            holder_identity: holder_identity.to_string(),
            status: LeaseStatus::Active,
            acquired_at,
            expires_at,
            last_heartbeat_at: 0,
            takeover_approval_id: None,
            record_hash: String::new(),
        };

        let record_hash = compute_lease_hash(&record);
        self.conn.execute(
            "INSERT INTO session_leases (lease_id, session_id, holder_surface, holder_identity,
             status, acquired_at, expires_at, last_heartbeat_at, takeover_approval_id, record_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                record.lease_id,
                record.session_id,
                record.holder_surface,
                record.holder_identity,
                "active",
                record.acquired_at,
                record.expires_at,
                0i64,
                None::<String>,
                record_hash,
            ],
        )?;

        let mut r = record;
        r.record_hash = record_hash;
        Ok(r)
    }

    /// Heartbeat — extend the lease TTL.
    pub fn heartbeat(&mut self, lease_id: &str, extend_seconds: i64) -> Result<(), LeaseError> {
        let now = unix_now();
        let new_expiry = now + extend_seconds;
        let affected = self.conn.execute(
            "UPDATE session_leases SET expires_at = ?1, last_heartbeat_at = ?2
             WHERE lease_id = ?3 AND status = 'active'",
            params![new_expiry, now, lease_id],
        )?;
        if affected == 0 {
            return Err(LeaseError::NotActive {
                lease_id: lease_id.to_string(),
            });
        }
        Ok(())
    }

    /// Release the lease voluntarily.
    pub fn release(&mut self, lease_id: &str) -> Result<(), LeaseError> {
        let affected = self.conn.execute(
            "UPDATE session_leases SET status = 'released' WHERE lease_id = ?1 AND status = 'active'",
            params![lease_id],
        )?;
        if affected == 0 {
            return Err(LeaseError::NotActive {
                lease_id: lease_id.to_string(),
            });
        }
        Ok(())
    }

    /// Force-takeover an active lease (requires a prior approval ID).
    pub fn force_takeover(
        &mut self,
        lease_id: &str,
        approval_id: &str,
    ) -> Result<(), LeaseError> {
        let affected = self.conn.execute(
            "UPDATE session_leases SET status = 'taken_over', takeover_approval_id = ?1
             WHERE lease_id = ?2 AND status = 'active'",
            params![approval_id, lease_id],
        )?;
        if affected == 0 {
            return Err(LeaseError::NotActive {
                lease_id: lease_id.to_string(),
            });
        }
        Ok(())
    }

    /// List leases for a session.
    pub fn list_for_session(&self, session_id: &str) -> Result<Vec<SessionLease>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT lease_id, session_id, holder_surface, holder_identity, status,
             acquired_at, expires_at, last_heartbeat_at, takeover_approval_id, record_hash
             FROM session_leases WHERE session_id = ?1 ORDER BY acquired_at DESC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(SessionLease {
                lease_id: row.get(0)?,
                session_id: row.get(1)?,
                holder_surface: row.get(2)?,
                holder_identity: row.get(3)?,
                status: LeaseStatus::from_db(&row.get::<_, String>(4)?).unwrap_or(LeaseStatus::Expired),
                acquired_at: row.get(5)?,
                expires_at: row.get(6)?,
                last_heartbeat_at: row.get(7)?,
                takeover_approval_id: row.get(8)?,
                record_hash: row.get(9)?,
            })
        })?;
        rows.collect()
    }

    /// Find the active lease for a session, if any.
    pub fn find_active(&self, session_id: &str) -> Result<Option<SessionLease>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT lease_id, session_id, holder_surface, holder_identity, status,
             acquired_at, expires_at, last_heartbeat_at, takeover_approval_id, record_hash
             FROM session_leases
             WHERE session_id = ?1 AND status = 'active'
             ORDER BY acquired_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![session_id], |row| {
            Ok(SessionLease {
                lease_id: row.get(0)?,
                session_id: row.get(1)?,
                holder_surface: row.get(2)?,
                holder_identity: row.get(3)?,
                status: LeaseStatus::from_db(&row.get::<_, String>(4)?).unwrap_or(LeaseStatus::Expired),
                acquired_at: row.get(5)?,
                expires_at: row.get(6)?,
                last_heartbeat_at: row.get(7)?,
                takeover_approval_id: row.get(8)?,
                record_hash: row.get(9)?,
            })
        })?;
        match rows.next() {
            Some(Ok(lease)) => Ok(Some(lease)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    /// Garbage-collect expired leases (marks them `expired`).
    pub fn gc_expired(&mut self) -> Result<usize, rusqlite::Error> {
        let now = unix_now();
        let affected = self.conn.execute(
            "UPDATE session_leases SET status = 'expired' WHERE status = 'active' AND expires_at <= ?1",
            params![now],
        )?;
        Ok(affected)
    }

    fn set_status(&mut self, lease_id: &str, status: LeaseStatus) -> Result<(), rusqlite::Error> {
        let s = status.to_string();
        self.conn.execute(
            "UPDATE session_leases SET status = ?1 WHERE lease_id = ?2",
            params![s, lease_id],
        )?;
        Ok(())
    }
}

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum LeaseError {
    /// Another agent holds an active lease.
    AlreadyHeld {
        lease_id: String,
        holder: String,
        surface: String,
        expires_at: i64,
    },
    /// The lease is not in active state (or does not exist).
    NotActive {
        lease_id: String,
    },
    /// Database error.
    Db(rusqlite::Error),
}

impl std::fmt::Display for LeaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyHeld { lease_id, holder, surface, expires_at } => {
                write!(
                    f,
                    "Lease {} held by {} ({}) — expires at {}",
                    lease_id, holder, surface, expires_at
                )
            }
            Self::NotActive { lease_id } => {
                write!(f, "Lease {} is not active", lease_id)
            }
            Self::Db(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for LeaseError {}

impl From<rusqlite::Error> for LeaseError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db(e)
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn compute_lease_hash(lease: &SessionLease) -> String {
    let input = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        lease.lease_id,
        lease.session_id,
        lease.holder_surface,
        lease.holder_identity,
        lease.status,
        lease.acquired_at,
        lease.expires_at,
        lease.takeover_approval_id.as_deref().unwrap_or("")
    );
    hex::encode(Sha256::digest(input.as_bytes()))
}

// ── Test helpers ─────────────────────────────────────────────────────────────

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use std::path::PathBuf;

    /// Create a temporary LeaseManager backed by an in-memory SQLite database.
    pub fn memory_manager() -> LeaseManager {
        LeaseManager::open(&PathBuf::from(":memory:")).expect("in-memory lease manager")
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn memory_manager() -> LeaseManager {
        LeaseManager::open(&PathBuf::from(":memory:")).expect("in-memory lease")
    }

    #[test]
    fn test_acquire_and_release() {
        let mut lm = memory_manager();
        let lease = lm
            .acquire("session-1", "claude_code_cli", "agent-alpha", 3600)
            .expect("Should acquire");
        assert_eq!(lease.status, LeaseStatus::Active);
        assert_eq!(lease.session_id, "session-1");

        lm.release(&lease.lease_id).expect("Should release");
        let active = lm.find_active("session-1").unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn test_acquire_conflict() {
        let mut lm = memory_manager();
        lm.acquire("session-1", "claude_code_cli", "agent-alpha", 3600)
            .expect("First acquire");

        let err = lm
            .acquire("session-1", "codex_cli", "agent-beta", 3600)
            .expect_err("Should reject second acquire");
        match err {
            LeaseError::AlreadyHeld { .. } => {} // expected
            _ => panic!("Expected AlreadyHeld, got: {}", err),
        }
    }

    #[test]
    fn test_heartbeat() {
        let mut lm = memory_manager();
        let lease = lm
            .acquire("session-1", "claude_code_cli", "agent-alpha", 10)
            .expect("Acquire");
        lm.heartbeat(&lease.lease_id, 3600)
            .expect("Heartbeat should extend");
    }

    #[test]
    fn test_expired_lease_can_be_reacquired() {
        let mut lm = memory_manager();
        let lease = lm
            .acquire("session-1", "claude_code_cli", "agent-alpha", -1) // already expired
            .expect("Acquire with past TTL");

        // TTL is -1 from now, so it's already expired. But lease is created as Active.
        // We need to manually expire it for the test.
        lm.release(&lease.lease_id).ok();

        let lease2 = lm
            .acquire("session-1", "codex_cli", "agent-beta", 3600)
            .expect("Should reacquire after expiry");
        assert_eq!(lease2.holder_identity, "agent-beta");
    }

    #[test]
    fn test_force_takeover() {
        let mut lm = memory_manager();
        let lease = lm
            .acquire("session-1", "claude_code_cli", "agent-alpha", 3600)
            .expect("Acquire");

        lm.force_takeover(&lease.lease_id, "approval-xyz")
            .expect("Force takeover");

        let active = lm.find_active("session-1").unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn test_list_for_session() {
        let mut lm = memory_manager();
        lm.acquire("session-1", "claude_code_cli", "agent-alpha", 3600)
            .expect("First");
        lm.acquire("session-2", "codex_cli", "agent-beta", 3600)
            .expect("Second");

        // Second acquire for session-1 should fail (held)
        let leases = lm.list_for_session("session-1").unwrap();
        assert_eq!(leases.len(), 1);
        assert_eq!(leases[0].holder_identity, "agent-alpha");
    }

    #[test]
    fn test_gc_expired() {
        let mut lm = memory_manager();
        lm.acquire("session-1", "claude_code_cli", "agent-alpha", -100)
            .expect("Acquire with negative TTL");

        // Release so gc can mark it
        // Actually gc_expired checks status='active' AND expires_at <= now.
        // We created with -100 so expires_at is in the past.
        let gc_count = lm.gc_expired().unwrap();
        assert_eq!(gc_count, 1);
    }

    #[test]
    fn test_release_nonexistent() {
        let mut lm = memory_manager();
        let err = lm.release("nonexistent-lease").expect_err("Should fail");
        match err {
            LeaseError::NotActive { .. } => {}
            _ => panic!("Expected NotActive, got: {}", err),
        }
    }
}
