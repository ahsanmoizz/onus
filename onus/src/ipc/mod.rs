pub mod client;
pub mod protocol;
pub mod server;

use serde::{Deserialize, Serialize};
use crate::{ActionType, Reversibility, Verdict};

/// An action intercepted by Onus, sent from an integration surface to Onus Core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub version: u8,
    pub session_id: String,
    pub sequence: u32,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub tool: String,
    pub payload: serde_json::Value,
}

/// Verdict returned by Onus Core to the integration surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResponse {
    pub version: u8,
    pub session_id: String,
    pub sequence: u32,
    pub decision: Verdict,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction: Option<String>,
    pub latency_us: u64,
    /// Reversibility classification of the triggered rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reversibility: Option<Reversibility>,
}

/// Session management requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum SessionCommand {
    /// Start a new agent session with scope information.
    #[serde(rename = "start")]
    Start {
        session_id: String,
        agent_name: String,
        agent_version: Option<String>,
        task_description: String,
        workspace_root: String,
        /// Files the agent declared it will modify (from plan/scope declaration).
        #[serde(default)]
        declared_files: Vec<String>,
        /// Additional directories/files explicitly allowed.
        #[serde(default)]
        allowed_paths: Vec<String>,
    },
    /// End an existing session.
    #[serde(rename = "end")]
    End {
        session_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Server-level commands (status, rules, shutdown).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum ServerCommand {
    /// Get daemon status summary.
    #[serde(rename = "status")]
    Status,
    /// List loaded rules.
    #[serde(rename = "rules")]
    Rules,
    /// Gracefully shut down the daemon.
    #[serde(rename = "shutdown")]
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub daemon_running: bool,
    pub active_sessions: usize,
    pub total_actions: u64,
    pub total_blocks: u64,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesResponse {
    pub rules: Vec<crate::policy::rule::RuleSummary>,
}

/// Top-level message: the daemon dispatches based on which variant is present.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_request: Option<ActionRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_command: Option<SessionCommand>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_command: Option<ServerCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_response: Option<ActionResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_response: Option<SessionResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_response: Option<StatusResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules_response: Option<RulesResponse>,
}
