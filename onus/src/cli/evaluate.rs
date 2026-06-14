//! `onus evaluate` — single-action evaluation, called by Claude Code preToolUse hook.
//!
//! Reads JSON action from stdin, evaluates via Onus Core, prints verdict JSON to stdout.
//! Exit codes: 0=allow, 1=warn, 2=block, 3=escalate.

use crate::audit::AuditTrail;
use crate::ipc::server::handle_action;
use crate::ipc::{Action, ActionRequest};
use crate::policy::PolicyEngine;
use clap::Args;
use serde::Deserialize;
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct EvaluateArgs {
    /// Path to rules TOML file (default: ~/.config/onus/rules/default.toml)
    #[arg(long)]
    pub rules: Option<PathBuf>,

    /// Path to audit database (default: ~/.local/share/onus/audit.db)
    #[arg(long)]
    pub db: Option<PathBuf>,
}

pub fn run(args: EvaluateArgs) -> anyhow::Result<()> {
    // Read JSON action from stdin.
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Try parsing as our native ActionRequest first, then fall back to
    // the Claude Code hook format and translate.
    let request: ActionRequest = match serde_json::from_str(&input) {
        Ok(r) => r,
        Err(_) => {
            // Not a native ActionRequest — try the Claude Code hook format.
            // Hook sends: { tool: "Bash", input: { command: "..." },
            //              session_id: "...", cwd: "...", agent: "...", agent_version: "..." }
            let hook: HookInput = serde_json::from_str(&input).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse stdin as ActionRequest or hook format: {}",
                    e
                )
            })?;
            translate_hook(hook)
        }
    };

    // Load policy engine.
    let rules_path = args
        .rules
        .unwrap_or_else(|| crate::config_dir().join("rules").join("default.toml"));

    if !rules_path.exists() {
        anyhow::bail!(
            "Rules file not found at {}. Run 'onus rules init' to create default rules.",
            rules_path.display()
        );
    }

    let rules = crate::policy::rule::load_rules(&rules_path)
        .map_err(|e| anyhow::anyhow!("Failed to load rules: {}", e))?;

    let policy_engine = PolicyEngine::new(rules);

    // Open audit trail.
    let db_path = args
        .db
        .unwrap_or_else(|| crate::data_dir().join("audit.db"));

    let audit_trail = AuditTrail::open(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open audit database: {}", e))?;

    let state = crate::ipc::server::ServerState::new(policy_engine, audit_trail);

    // Evaluate.
    let response = handle_action(&state, request);

    // Write JSON response to stdout.
    let output = serde_json::to_string(&response)?;
    io::stdout().write_all(output.as_bytes())?;
    io::stdout().write_all(b"\n")?;
    io::stdout().flush()?;

    // Exit with appropriate code.
    std::process::exit(response.decision.exit_code());
}

// ── Claude Code hook format adapter ──────────────────────────────────────

/// Raw input from Claude Code's preToolUse hook.
/// Hook sends flat fields: { tool, input, session_id, cwd, agent, agent_version }
#[derive(Debug, Deserialize)]
struct HookInput {
    tool: String,
    #[serde(default)]
    input: serde_json::Value,
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    cwd: String,
    #[serde(default)]
    #[allow(dead_code)]
    agent: String,
    #[serde(default)]
    #[allow(dead_code)]
    agent_version: String,
}

/// Translate Claude Code hook format into our internal ActionRequest.
fn translate_hook(hook: HookInput) -> ActionRequest {
    let action_type = match hook.tool.to_lowercase().as_str() {
        "bash" => crate::ActionType::Shell,
        "write" | "edit" => crate::ActionType::FileWrite,
        "delete" | "remove" => crate::ActionType::FileDelete,
        "read" | "view" => crate::ActionType::FileRead,
        "git" | "github" => crate::ActionType::Git,
        "web_fetch" | "websearch" | "web" => crate::ActionType::Network,
        _ => crate::ActionType::Shell,
    };

    // Build a payload that always includes the full input, plus
    // command/cwd extracted for rules that inspect them directly.
    let mut payload = hook
        .input
        .as_object()
        .cloned()
        .unwrap_or_else(serde_json::Map::new);
    if !hook.cwd.is_empty() {
        payload.insert("cwd".into(), serde_json::Value::String(hook.cwd));
    }

    ActionRequest {
        version: 1,
        session_id: if hook.session_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            hook.session_id
        },
        sequence: 0,
        action: Action {
            action_type,
            tool: hook.tool,
            payload: serde_json::Value::Object(payload),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_bash_hook() {
        let input = serde_json::json!({
            "tool": "Bash",
            "input": { "command": "rm -rf /" },
            "session_id": "hook-test-001",
            "cwd": "/home/user/project",
            "agent": "claude-code",
            "agent_version": "1.2.3"
        });
        let hook: HookInput = serde_json::from_value(input).unwrap();
        let req = translate_hook(hook);

        assert_eq!(req.version, 1);
        assert_eq!(req.session_id, "hook-test-001");
        assert_eq!(req.action.tool, "Bash");
        assert_eq!(req.action.payload["command"], "rm -rf /");
        assert_eq!(req.action.payload["cwd"], "/home/user/project");
    }

    #[test]
    fn test_translate_write_hook() {
        let input = serde_json::json!({
            "tool": "Write",
            "input": { "file_path": "/etc/passwd", "content": "hacker" },
            "session_id": "hook-test-002",
            "cwd": "/home/user/project"
        });
        let hook: HookInput = serde_json::from_value(input).unwrap();
        let req = translate_hook(hook);

        assert_eq!(req.action.action_type, crate::ActionType::FileWrite);
        assert_eq!(req.action.payload["file_path"], "/etc/passwd");
        assert!(req.action.payload.get("cwd").is_some());
    }

    #[test]
    fn test_translate_no_session_id_generates_uuid() {
        let input = serde_json::json!({
            "tool": "Bash",
            "input": { "command": "ls" }
        });
        let hook: HookInput = serde_json::from_value(input).unwrap();
        let req = translate_hook(hook);
        // UUID v4 is 36 chars
        assert_eq!(req.session_id.len(), 36);
    }
}
