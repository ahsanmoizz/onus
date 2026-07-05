pub mod audit;
pub mod cli;
pub mod config_env;
pub mod daemon;
pub mod ipc;
pub mod mcp;
pub mod memory;
pub mod policy;
pub mod prompt_intake;
pub mod quality;
pub mod scope;
pub mod security;
pub mod semantic;
pub mod task_contract;
pub mod workspace;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Onus release version, set during build.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default socket path for Onus Core IPC.
pub fn default_socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
            let user = std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "unknown".to_string());
            format!("/tmp/onus-{}", user)
        });
        PathBuf::from(runtime_dir).join("onus.sock")
    }
    #[cfg(windows)]
    {
        PathBuf::from(r"\\\\.\\pipe\\onus-ipc")
    }
}

/// Default config directory.
pub fn config_dir() -> PathBuf {
    #[cfg(unix)]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home).join(".config").join("onus")
    }
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        PathBuf::from(appdata).join("onus")
    }
}

/// Default data directory (for SQLite audit trail).
pub fn data_dir() -> PathBuf {
    if let Ok(override_dir) = std::env::var("ONUS_DATA_DIR") {
        return PathBuf::from(override_dir);
    }
    #[cfg(unix)]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("onus")
    }
    #[cfg(windows)]
    {
        config_dir().join("data")
    }
}

/// Decision returned by the policy engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Proceed. Logged silently.
    Allow,
    /// Proceed, but surface a warning.
    Warn,
    /// Halt. Return correction to agent.
    Block,
    /// Halt. Require human approval.
    Escalate,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verdict::Allow => write!(f, "allow"),
            Verdict::Warn => write!(f, "warn"),
            Verdict::Block => write!(f, "block"),
            Verdict::Escalate => write!(f, "escalate"),
        }
    }
}

impl Verdict {
    pub fn exit_code(&self) -> i32 {
        match self {
            Verdict::Allow => 0,
            Verdict::Warn => 1,
            Verdict::Block => 2,
            Verdict::Escalate => 3,
        }
    }

    pub fn from_exit_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Verdict::Allow),
            1 => Some(Verdict::Warn),
            2 => Some(Verdict::Block),
            3 => Some(Verdict::Escalate),
            _ => None,
        }
    }
}

/// Action types that Onus can intercept.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Shell,
    FileWrite,
    FileDelete,
    FileRead,
    Git,
    ApiCall,
    DbMutation,
    Network,
    #[serde(rename = "mcp")]
    MCP,
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::Shell => write!(f, "shell"),
            ActionType::FileWrite => write!(f, "file_write"),
            ActionType::FileDelete => write!(f, "file_delete"),
            ActionType::FileRead => write!(f, "file_read"),
            ActionType::Git => write!(f, "git"),
            ActionType::ApiCall => write!(f, "api_call"),
            ActionType::DbMutation => write!(f, "db_mutation"),
            ActionType::Network => write!(f, "network"),
            ActionType::MCP => write!(f, "mcp"),
        }
    }
}

/// Reversibility classification (Phase 3).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    Reversible,
    Compensable,
    Irreversible,
}

/// Recovery model classes for checkpoint and rollback planning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryClass {
    R0ReadOnly,
    R1AutomaticallyReversible,
    R2SnapshotReversible,
    R3Compensatable,
    R4IrreversibleOrMitigationOnly,
}

impl RecoveryClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::R0ReadOnly => "R0_READ_ONLY",
            Self::R1AutomaticallyReversible => "R1_AUTOMATICALLY_REVERSIBLE",
            Self::R2SnapshotReversible => "R2_SNAPSHOT_REVERSIBLE",
            Self::R3Compensatable => "R3_COMPENSATABLE",
            Self::R4IrreversibleOrMitigationOnly => "R4_IRREVERSIBLE_OR_MITIGATION_ONLY",
        }
    }
}

impl From<&Reversibility> for RecoveryClass {
    fn from(value: &Reversibility) -> Self {
        match value {
            Reversibility::Reversible => Self::R1AutomaticallyReversible,
            Reversibility::Compensable => Self::R3Compensatable,
            Reversibility::Irreversible => Self::R4IrreversibleOrMitigationOnly,
        }
    }
}

pub mod approval;
pub mod approval_broker;
pub mod authority;
pub mod rollback;
pub mod handoff;
pub mod lease;

/// Evaluate an action request and return (verdict, rule_id, rule_name, correction).
/// Loads the policy engine fresh — ok for one-off evaluation (CLI) but not hot-path.
pub fn evaluate_request(
    request: &ipc::ActionRequest,
) -> (Verdict, Option<String>, Option<String>, Option<String>) {
    let rules_path = config_dir().join("rules").join("default.toml");
    let rules = match policy::rule::load_rules(&rules_path) {
        Ok(r) => r,
        Err(e) => {
            log::error!("Failed to load rules for evaluate_request: {}", e);
            return (
                Verdict::Block,
                Some("ONUS_EVALUATOR_FAILURE".into()),
                Some("rules-load-failed".into()),
                Some("Onus could not load its rules, so the action was blocked instead of silently allowed.".into()),
            );
        }
    };
    let engine = policy::engine::PolicyEngine::new(rules);
    let verdict = engine.evaluate(request);
    let last_match = engine.last_match();
    match last_match {
        Some(m) => (verdict, Some(m.id), Some(m.name), Some(m.correction)),
        None => (verdict, None, None, None),
    }
}
