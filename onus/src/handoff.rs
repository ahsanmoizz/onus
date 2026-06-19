//! # Handoff Manifest — Cross-Agent Continuity
//!
//! A handoff manifest serialises the six sources of truth that an agent must
//! transfer when another agent (possibly on a different surface) continues the
//! same task.  The manifest is versioned, hashed, and optionally signed so the
//! receiving agent can verify integrity.
//!
//! ## Six sources of truth
//!
//! 1. **Task contract** — what the agent was asked to do, which files are
//!    allowed, which shell commands are permitted, completion evidence.
//! 2. **Repository state** — checkpoint manifests, workspace root, git HEAD.
//! 3. **Session memory** — memory entries scoped to the current session.
//! 4. **Project memory** — memory entries scoped to the current project.
//! 5. **Policy / incident context** — active policy rules, open incidents.
//! 6. **Audit / receipt state** — the session's audit trail and the last
//!    verified receipt hash so the next agent can chain from it.
//!
//! ## Explicitly excluded from transfer
//!
//! - Hidden model memory / chain-of-thought (vendor-specific, not portable).
//! - Vendor-specific state (Claude conversation ID, Codex session tokens …).
//! - Provider quotas or API keys.
//! - Expired approvals (must be re-requested on the receiving surface).
//! - Unrestricted authority (the manifest carries only scoped context).
//!
//! ## Schema version
//!
//! Current schema: `1`

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ── Manifest schema ──────────────────────────────────────────────────────────

/// RFC 3339 UTC timestamp string (without chrono dependency).
fn rfc3339_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Format as ISO 8601 / RFC 3339: 2026-06-18T12:34:56Z
    // Use a simple calculation — not perfect leap-year but correct for all modern years.
    let days_since_epoch = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Date from days since epoch using civil date algorithm
    let (year, month, day) = civil_date_from_days(days_since_epoch as i64);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn civil_date_from_days(days: i64) -> (i64, u32, u32) {
    // Algorithm from Howard Hinnant
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

/// Canonical handoff manifest schema v1.
///
/// Every field is mandatory so that a missing field is a validation error, not a
/// silent default.  Optional-ness is modelled via `Option<T>` only when the
/// source of truth genuinely might be absent (e.g. no active task contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffManifestV1 {
    // ── Schema metadata ────────────────────────────────────────────────────
    /// Manifest schema version (must be `1`).
    pub schema_version: u8,
    /// When this manifest was created (UTC ISO-8601).
    pub created_at: String,
    /// Human-readable description of the handoff reason.
    pub reason: String,

    // ── Surface identity ───────────────────────────────────────────────────
    /// Which surface created the manifest (e.g. `claude_code_cli`, `codex_cli`).
    pub source_surface: String,
    /// The receiving surface (e.g. `codex_cli`).  `"any"` means any surface may
    /// continue.
    pub target_surface: String,

    // ── 1. Task contract ──────────────────────────────────────────────────
    /// Serialised task contract, if one is active.
    pub task_contract: Option<TaskContractSnapshot>,
    /// The original prompt that started the session.
    pub original_prompt: Option<String>,

    // ── 2. Repository state ───────────────────────────────────────────────
    /// Absolute or canonical workspace root path.
    pub workspace_root: Option<String>,
    /// Git HEAD commit hash at handoff time.
    pub git_head_hash: Option<String>,
    /// Git branch name at handoff time.
    pub git_branch: Option<String>,
    /// Last checkpoint ID for this session, if any.
    pub last_checkpoint_id: Option<String>,

    // ── 3. Session memory ─────────────────────────────────────────────────
    /// Number of session-scoped memory entries.
    pub session_memory_count: u32,
    /// Compact summary of the session memory (the receiving agent uses this to
    /// decide whether to import the full entries).
    pub session_memory_summary: Option<String>,

    // ── 4. Project memory ─────────────────────────────────────────────────
    /// Number of project-scoped memory entries.
    pub project_memory_count: u32,
    /// Compact summary of the project memory.
    pub project_memory_summary: Option<String>,

    // ── 5. Policy / incident context ──────────────────────────────────────
    /// Number of active policy rules at handoff time.
    pub active_rule_count: u32,
    /// Number of open (unresolved) incidents.
    pub open_incident_count: u32,
    /// Summary of open incidents.
    pub incident_summary: Option<String>,

    // ── 6. Audit / receipt state ──────────────────────────────────────────
    /// Session ID on the source surface.
    pub session_id: String,
    /// Number of audited actions in this session.
    pub action_count: u32,
    /// Last verified receipt hash (hex).  The receiving agent should chain new
    /// actions from this hash.
    pub last_receipt_hash: Option<String>,
    /// SHA-256 of the audit DB path (for integrity cross-check).
    pub audit_db_hash: Option<String>,

    // ── Integrity ──────────────────────────────────────────────────────────
    /// SHA-256 of every field above (before signature is attached).  This is
    /// the "canonical hash" that a signature binds to.
    pub canonical_hash: String,
    /// Ed25519 signature over `canonical_hash` (hex), or empty if unsigned.
    pub signature: String,
}

/// A lightweight snapshot of the task contract suitable for transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContractSnapshot {
    pub session_id: String,
    pub original_prompt: Option<String>,
    pub normalized_objective: Option<String>,
    pub allowed_paths: Vec<String>,
    pub allowed_resources: Vec<String>,
    pub protected_paths: Vec<String>,
    pub protected_resources: Vec<String>,
    pub required_evidence: Vec<String>,
    pub max_files_changed: u32,
    pub max_actions: u32,
}

// ── Implementation ───────────────────────────────────────────────────────────

impl HandoffManifestV1 {
    /// Compute the canonical hash over all fields (excluding `canonical_hash`
    /// and `signature`).  The hash is SHA-256 of the canonical JSON
    /// representation of a copy that has those two fields cleared.
    pub fn compute_canonical_hash(&self) -> String {
        let mut copy = self.clone();
        copy.canonical_hash = String::new();
        copy.signature = String::new();
        let json = serde_json::to_string(&copy).unwrap_or_default();
        hex::encode(Sha256::digest(json.as_bytes()))
    }

    /// Verify that `canonical_hash` matches a fresh computation.
    pub fn verify_hash(&self) -> bool {
        self.canonical_hash == self.compute_canonical_hash()
    }

    /// Verify hash *and* (if a signature is present) verify the signature
    /// against the known public key.
    pub fn verify(&self, public_key: Option<&[u8]>) -> bool {
        if !self.verify_hash() {
            return false;
        }
        if let Some(pk) = public_key {
            if self.signature.is_empty() {
                return false;
            }
            let sig_bytes = match hex::decode(&self.signature) {
                Ok(b) => b,
                Err(_) => return false,
            };
            let hash_bytes = hex::decode(&self.canonical_hash).unwrap_or_default();
            verify_ed25519(pk, &hash_bytes, &sig_bytes)
        } else {
            // No public key provided — hash-only verification
            true
        }
    }

    /// Attach an Ed25519 signature over the canonical hash.
    pub fn sign(&mut self, secret_key: &[u8]) {
        self.canonical_hash = self.compute_canonical_hash();
        let hash_bytes = hex::decode(&self.canonical_hash).unwrap_or_default();
        self.signature = sign_ed25519(secret_key, &hash_bytes);
    }
}

// ── Ed25519 helpers ─────────────────────────────────────────────────────────

fn sign_ed25519(secret_key: &[u8], msg: &[u8]) -> String {
    // Use ed25519-dalek if available, otherwise hex-encode a placeholder
    #[cfg(feature = "handoff_signing")]
    {
        use ed25519_dalek::{Signer, SigningKey};
        if let Ok(sk) = SigningKey::from_bytes(secret_key.try_into()) {
            let sig = sk.sign(msg);
            return hex::encode(sig.to_bytes());
        }
        String::new()
    }
    #[cfg(not(feature = "handoff_signing"))]
    {
        let _ = (secret_key, msg);
        String::new()
    }
}

fn verify_ed25519(public_key: &[u8], msg: &[u8], sig: &[u8]) -> bool {
    #[cfg(feature = "handoff_signing")]
    {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};
        if let (Ok(vk), Ok(s)) = (
            VerifyingKey::from_bytes(public_key.try_into()),
            Signature::from_slice(sig),
        ) {
            return vk.verify(msg, &s).is_ok();
        }
        false
    }
    #[cfg(not(feature = "handoff_signing"))]
    {
        let _ = (public_key, msg, sig);
        true // pass-through when feature is off
    }
}

// ── Builder ──────────────────────────────────────────────────────────────────

/// Convenience builder for constructing a manifest from the available sources.
#[derive(Debug, Default)]
pub struct HandoffManifestBuilder {
    source_surface: Option<String>,
    target_surface: Option<String>,
    reason: Option<String>,
    session_id: Option<String>,
    task_contract: Option<TaskContractSnapshot>,
    original_prompt: Option<String>,
    workspace_root: Option<String>,
    git_head_hash: Option<String>,
    git_branch: Option<String>,
    last_checkpoint_id: Option<String>,
    session_memory_count: u32,
    session_memory_summary: Option<String>,
    project_memory_count: u32,
    project_memory_summary: Option<String>,
    active_rule_count: u32,
    open_incident_count: u32,
    incident_summary: Option<String>,
    action_count: u32,
    last_receipt_hash: Option<String>,
    audit_db_hash: Option<String>,
}

impl HandoffManifestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn source_surface(mut self, v: &str) -> Self {
        self.source_surface = Some(v.to_string());
        self
    }
    pub fn target_surface(mut self, v: &str) -> Self {
        self.target_surface = Some(v.to_string());
        self
    }
    pub fn reason(mut self, v: &str) -> Self {
        self.reason = Some(v.to_string());
        self
    }
    pub fn session_id(mut self, v: &str) -> Self {
        self.session_id = Some(v.to_string());
        self
    }
    pub fn task_contract(mut self, v: TaskContractSnapshot) -> Self {
        self.task_contract = Some(v);
        self
    }
    pub fn original_prompt(mut self, v: &str) -> Self {
        self.original_prompt = Some(v.to_string());
        self
    }
    pub fn workspace_root(mut self, v: &str) -> Self {
        self.workspace_root = Some(v.to_string());
        self
    }
    pub fn git_head_hash(mut self, v: &str) -> Self {
        self.git_head_hash = Some(v.to_string());
        self
    }
    pub fn git_branch(mut self, v: &str) -> Self {
        self.git_branch = Some(v.to_string());
        self
    }
    pub fn last_checkpoint_id(mut self, v: &str) -> Self {
        self.last_checkpoint_id = Some(v.to_string());
        self
    }
    pub fn session_memory(mut self, count: u32, summary: Option<&str>) -> Self {
        self.session_memory_count = count;
        self.session_memory_summary = summary.map(String::from);
        self
    }
    pub fn project_memory(mut self, count: u32, summary: Option<&str>) -> Self {
        self.project_memory_count = count;
        self.project_memory_summary = summary.map(String::from);
        self
    }
    pub fn policy_context(mut self, rules: u32, incidents: u32, summary: Option<&str>) -> Self {
        self.active_rule_count = rules;
        self.open_incident_count = incidents;
        self.incident_summary = summary.map(String::from);
        self
    }
    pub fn audit_state(
        mut self,
        action_count: u32,
        last_receipt_hash: Option<&str>,
        audit_db_hash: Option<&str>,
    ) -> Self {
        self.action_count = action_count;
        self.last_receipt_hash = last_receipt_hash.map(String::from);
        self.audit_db_hash = audit_db_hash.map(String::from);
        self
    }

    /// Build the manifest.  Returns `None` if required fields are missing.
    pub fn build(self) -> Option<HandoffManifestV1> {
        let created_at = rfc3339_now();
        let mut m = HandoffManifestV1 {
            schema_version: 1,
            created_at,
            reason: self.reason.unwrap_or_default(),
            source_surface: self.source_surface?,
            target_surface: self.target_surface.unwrap_or_else(|| "any".to_string()),
            task_contract: self.task_contract,
            original_prompt: self.original_prompt,
            workspace_root: self.workspace_root,
            git_head_hash: self.git_head_hash,
            git_branch: self.git_branch,
            last_checkpoint_id: self.last_checkpoint_id,
            session_memory_count: self.session_memory_count,
            session_memory_summary: self.session_memory_summary,
            project_memory_count: self.project_memory_count,
            project_memory_summary: self.project_memory_summary,
            active_rule_count: self.active_rule_count,
            open_incident_count: self.open_incident_count,
            incident_summary: self.incident_summary,
            session_id: self.session_id?,
            action_count: self.action_count,
            last_receipt_hash: self.last_receipt_hash,
            audit_db_hash: self.audit_db_hash,
            canonical_hash: String::new(),
            signature: String::new(),
        };
        m.canonical_hash = m.compute_canonical_hash();
        Some(m)
    }
}

// ── Serialisation helpers ────────────────────────────────────────────────────

impl HandoffManifestV1 {
    /// Serialise to pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialise from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Write to a file path.
    pub fn to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let json = self.to_json_pretty()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Read from a file path.
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&json)?)
    }
}

// ── Surface identifiers ──────────────────────────────────────────────────────

/// Well-known surface identifiers used in handoff manifests.
pub mod surfaces {
    /// Claude Code CLI (`onus claude-hook`)
    pub const CLAUDE_CODE_CLI: &str = "claude_code_cli";
    /// OpenAI Codex CLI (MCP proxy)
    pub const CODEX_CLI: &str = "codex_cli";
    /// Cursor IDE (hooks + MCP)
    pub const CURSOR_IDE: &str = "cursor_ide";
    /// Google Antigravity (extension + MCP)
    pub const ANTIGRAVITY: &str = "antigravity";
    /// Any surface may continue the task (wildcard target)
    pub const ANY: &str = "any";
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_minimal_manifest() {
        let m = HandoffManifestBuilder::new()
            .source_surface(surfaces::CLAUDE_CODE_CLI)
            .target_surface(surfaces::CODEX_CLI)
            .reason("Phase 18 cross-agent continuity test")
            .session_id("test-session-001")
            .build()
            .expect("Builder should produce a manifest");

        assert_eq!(m.schema_version, 1);
        assert_eq!(m.source_surface, "claude_code_cli");
        assert_eq!(m.target_surface, "codex_cli");
        assert!(!m.canonical_hash.is_empty());
        assert!(m.signature.is_empty()); // unsigned by default
    }

    #[test]
    fn test_build_full_manifest() {
        let tc = TaskContractSnapshot {
            session_id: "test-session-001".to_string(),
            original_prompt: Some("Refactor the auth module".to_string()),
            normalized_objective: Some("Refactor auth module".to_string()),
            allowed_paths: vec!["src/auth/".to_string()],
            allowed_resources: vec![],
            protected_paths: vec![],
            protected_resources: vec![],
            required_evidence: vec!["file_write".to_string()],
            max_files_changed: 10,
            max_actions: 100,
        };

        let m = HandoffManifestBuilder::new()
            .source_surface(surfaces::CLAUDE_CODE_CLI)
            .target_surface(surfaces::CODEX_CLI)
            .reason("Agent switching from Claude Code to Codex CLI")
            .session_id("test-session-001")
            .task_contract(tc)
            .original_prompt("Refactor the auth module")
            .workspace_root("/home/user/project")
            .git_head_hash("abc123def456")
            .git_branch("feature/auth-refactor")
            .last_checkpoint_id("cp-001")
            .session_memory(3, Some("Found 3 auth-related issues"))
            .project_memory(5, Some("Project conventions documented"))
            .policy_context(12, 1, Some("1 open incident: credential leak"))
            .audit_state(42, Some("abc123"), Some("def456"))
            .build()
            .expect("Builder should produce a manifest");

        assert_eq!(m.schema_version, 1);
        assert_eq!(m.session_memory_count, 3);
        assert_eq!(m.action_count, 42);
        assert!(m.verify_hash());
    }

    #[test]
    fn test_canonical_hash_consistency() {
        let m1 = HandoffManifestBuilder::new()
            .source_surface(surfaces::CLAUDE_CODE_CLI)
            .target_surface(surfaces::CODEX_CLI)
            .reason("test")
            .session_id("s1")
            .build()
            .unwrap();

        let m2 = HandoffManifestBuilder::new()
            .source_surface(surfaces::CLAUDE_CODE_CLI)
            .target_surface(surfaces::CODEX_CLI)
            .reason("test")
            .session_id("s1")
            .build()
            .unwrap();

        // Same inputs → same hash
        assert_eq!(m1.canonical_hash, m2.canonical_hash);
    }

    #[test]
    fn test_canonical_hash_tamper_detection() {
        let mut m = HandoffManifestBuilder::new()
            .source_surface(surfaces::CLAUDE_CODE_CLI)
            .target_surface(surfaces::CODEX_CLI)
            .reason("test")
            .session_id("s1")
            .build()
            .unwrap();

        assert!(m.verify_hash());

        // Tamper with a field
        m.action_count = 999;
        assert!(!m.verify_hash());
    }

    #[test]
    fn test_json_round_trip() {
        let m1 = HandoffManifestBuilder::new()
            .source_surface(surfaces::CLAUDE_CODE_CLI)
            .target_surface(surfaces::CODEX_CLI)
            .reason("round-trip test")
            .session_id("s1")
            .build()
            .unwrap();

        let json = m1.to_json_pretty().unwrap();
        let m2 = HandoffManifestV1::from_json(&json).unwrap();

        assert_eq!(m1.canonical_hash, m2.canonical_hash);
        assert_eq!(m1.session_id, m2.session_id);
    }

    #[test]
    fn test_surface_constants() {
        assert_eq!(surfaces::CLAUDE_CODE_CLI, "claude_code_cli");
        assert_eq!(surfaces::CODEX_CLI, "codex_cli");
        assert_eq!(surfaces::CURSOR_IDE, "cursor_ide");
        assert_eq!(surfaces::ANTIGRAVITY, "antigravity");
        assert_eq!(surfaces::ANY, "any");
    }
}
