//! `onus claude-hook` — Claude Code PreToolUse adapter.
//!
//! This is an L1 cooperative hook. It is BEST-EFFORT: Claude Code must be
//! configured to call it, and direct tool execution outside the hook bypasses it.

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Args)]
pub struct ClaudeHookArgs {
    /// Path to rules TOML file passed to `onus evaluate`
    #[arg(long)]
    pub rules: Option<PathBuf>,

    /// Path to audit database passed to `onus evaluate`
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Evaluator executable. Defaults to the current Onus binary.
    #[arg(long)]
    pub evaluator: Option<PathBuf>,

    /// Arguments for evaluator executable. Defaults to `evaluate` when omitted.
    #[arg(long = "evaluator-arg")]
    pub evaluator_args: Vec<String>,

    /// Timeout for evaluator process.
    #[arg(long, default_value_t = 5000)]
    pub timeout_ms: u64,

    /// What to do when the hook is explicitly disabled.
    #[arg(long, value_enum, default_value_t = DisabledBehavior::Allow)]
    pub disabled_behavior: DisabledBehavior,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum DisabledBehavior {
    Allow,
    Ask,
    Deny,
}

#[derive(Debug, Deserialize)]
struct ClaudeHookInput {
    #[serde(default)]
    hook_event_name: String,
    #[serde(default, alias = "tool")]
    tool_name: String,
    #[serde(default, alias = "input")]
    tool_input: Value,
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    cwd: String,
    #[serde(default)]
    agent: String,
    #[serde(default)]
    agent_version: String,
    #[serde(default)]
    agent_type: String,
    #[serde(default)]
    transcript_path: String,
}

#[derive(Debug, Serialize)]
struct ClaudeHookOutput {
    #[serde(rename = "hookSpecificOutput")]
    hook_specific_output: ClaudeHookSpecificOutput,
    #[serde(rename = "suppressOutput")]
    suppress_output: bool,
    #[serde(rename = "systemMessage", skip_serializing_if = "Option::is_none")]
    system_message: Option<String>,
}

#[derive(Debug, Serialize)]
struct ClaudeHookSpecificOutput {
    #[serde(rename = "hookEventName")]
    hook_event_name: String,
    #[serde(rename = "permissionDecision")]
    permission_decision: String,
    #[serde(
        rename = "permissionDecisionReason",
        skip_serializing_if = "Option::is_none"
    )]
    permission_decision_reason: Option<String>,
}

pub fn run(args: ClaudeHookArgs) -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let output = match evaluate_claude_hook(&input, &args) {
        Ok(output) => output,
        Err(err) => decision(
            "deny",
            format!("Onus Claude Code hook rejected malformed input: {}", err),
            Some("Onus Claude Code hook could not parse the hook payload.".to_string()),
        ),
    };
    let json = serde_json::to_string(&output)?;
    io::stdout().write_all(json.as_bytes())?;
    io::stdout().write_all(b"\n")?;
    io::stdout().flush()?;
    Ok(())
}

fn evaluate_claude_hook(input: &str, args: &ClaudeHookArgs) -> anyhow::Result<ClaudeHookOutput> {
    if hook_disabled() {
        return Ok(match args.disabled_behavior {
            DisabledBehavior::Allow => decision(
                "allow",
                "Onus Claude Code hook is disabled; this L1 BEST-EFFORT hook is bypassed.",
                Some("Onus Claude Code hook disabled.".to_string()),
            ),
            DisabledBehavior::Ask => decision(
                "ask",
                "Onus Claude Code hook is disabled; ask the user before proceeding.",
                Some("Onus Claude Code hook disabled.".to_string()),
            ),
            DisabledBehavior::Deny => decision(
                "deny",
                "Onus Claude Code hook is disabled and configured to fail closed.",
                Some("Onus Claude Code hook disabled; fail closed.".to_string()),
            ),
        });
    }

    let hook: ClaudeHookInput = serde_json::from_str(input)?;
    let Some(action_type) = action_type_for_tool(&hook.tool_name) else {
        return Ok(decision(
            "ask",
            format!(
                "Onus Claude Code hook does not yet support tool '{}'. Ask before proceeding.",
                hook.tool_name
            ),
            Some("Unsupported Claude Code tool for Onus BEST-EFFORT hook.".to_string()),
        ));
    };

    let request = normalized_request(&hook, action_type);
    let evaluator_output = run_evaluator(&request, args)?;
    Ok(decision_from_evaluator(&evaluator_output))
}

fn hook_disabled() -> bool {
    std::env::var("ONUS_CLAUDE_HOOK_DISABLED")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn action_type_for_tool(tool_name: &str) -> Option<&'static str> {
    match tool_name.to_ascii_lowercase().as_str() {
        "bash" => Some("shell"),
        "write" | "edit" | "multiedit" | "notebookedit" => Some("file_write"),
        "delete" | "remove" => Some("file_delete"),
        "read" | "view" | "glob" | "grep" | "ls" | "notebookread" => Some("file_read"),
        "webfetch" | "web_fetch" | "websearch" | "web_search" => Some("network"),
        "task" | "taskcreate" => Some("mcp"),
        tool if tool.starts_with("mcp__") => Some("mcp"),
        _ => None,
    }
}

fn normalized_request(hook: &ClaudeHookInput, action_type: &str) -> Value {
    let mut payload = hook
        .tool_input
        .as_object()
        .cloned()
        .unwrap_or_else(Map::new);
    if !hook.cwd.is_empty() {
        payload.insert("cwd".to_string(), Value::String(hook.cwd.clone()));
    }
    payload.insert(
        "claude_hook".to_string(),
        serde_json::json!({
            "hook_event_name": if hook.hook_event_name.is_empty() { "PreToolUse" } else { &hook.hook_event_name },
            "tool_name": hook.tool_name,
            "agent": hook.agent,
            "agent_version": hook.agent_version,
            "agent_type": hook.agent_type,
            "transcript_path_hash": if hook.transcript_path.is_empty() {
                Value::Null
            } else {
                Value::String(crate::security::sha256_hex(&hook.transcript_path))
            },
            "integration_level": "L1_BEST_EFFORT",
        }),
    );

    serde_json::json!({
        "version": 1,
        "session_id": if hook.session_id.is_empty() {
            format!("claude-hook-{}", uuid::Uuid::new_v4())
        } else {
            hook.session_id.clone()
        },
        "sequence": 0,
        "action": {
            "type": action_type,
            "tool": hook.tool_name,
            "payload": Value::Object(payload),
        }
    })
}

fn run_evaluator(request: &Value, args: &ClaudeHookArgs) -> anyhow::Result<Value> {
    let evaluator = args.evaluator.clone().unwrap_or(std::env::current_exe()?);
    let mut command = Command::new(evaluator);
    if args.evaluator_args.is_empty() {
        command.arg("evaluate");
    } else {
        command.args(&args.evaluator_args);
    }
    if let Some(rules) = &args.rules {
        command.arg("--rules").arg(rules);
    }
    if let Some(db) = &args.db {
        command.arg("--db").arg(db);
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|e| {
        anyhow::anyhow!(
            "Onus evaluator unavailable; Claude Code tool is denied in fail-closed mode: {}",
            e
        )
    })?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to open evaluator stdin"))?;
        stdin.write_all(serde_json::to_string(request)?.as_bytes())?;
    }

    let start = Instant::now();
    loop {
        if let Some(_status) = child.try_wait()? {
            break;
        }
        if start.elapsed() >= Duration::from_millis(args.timeout_ms.max(1)) {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("Onus evaluator timed out after {} ms", args.timeout_ms);
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    let output = child.wait_with_output()?;
    if output.stdout.is_empty() {
        anyhow::bail!(
            "Onus evaluator produced no JSON output: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| {
        anyhow::anyhow!(
            "Onus evaluator produced malformed output: {}; stdout={}",
            e,
            String::from_utf8_lossy(&output.stdout)
        )
    })?;
    Ok(value)
}

fn decision_from_evaluator(response: &Value) -> ClaudeHookOutput {
    let decision_value = response
        .get("decision")
        .and_then(Value::as_str)
        .unwrap_or("block");
    let correction = response
        .get("correction")
        .and_then(Value::as_str)
        .or_else(|| response.get("approval_reason").and_then(Value::as_str))
        .unwrap_or("Onus denied or escalated this action.");
    let rule = response
        .get("rule_id")
        .and_then(Value::as_str)
        .unwrap_or("ONUS_POLICY");
    match decision_value {
        "allow" | "warn" => decision(
            "allow",
            format!("Onus allowed this Claude Code tool call ({decision_value})."),
            None,
        ),
        "escalate" => decision(
            "ask",
            format!("Onus requires approval before this Claude Code action proceeds: {rule} - {correction}"),
            Some("Onus requires human approval for this Claude Code action.".to_string()),
        ),
        _ => decision(
            "deny",
            format!("Onus blocked this Claude Code action: {rule} - {correction}"),
            Some("Onus blocked this Claude Code action and returned a correction.".to_string()),
        ),
    }
}

fn decision(
    permission_decision: impl Into<String>,
    reason: impl Into<String>,
    system_message: Option<String>,
) -> ClaudeHookOutput {
    ClaudeHookOutput {
        hook_specific_output: ClaudeHookSpecificOutput {
            hook_event_name: "PreToolUse".to_string(),
            permission_decision: permission_decision.into(),
            permission_decision_reason: Some(reason.into()),
        },
        suppress_output: true,
        system_message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_official_claude_hook_shape() {
        let hook: ClaudeHookInput = serde_json::from_value(serde_json::json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "echo ok"},
            "session_id": "claude-session",
            "cwd": "/repo",
            "agent": "claude-code",
            "agent_version": "2.1.177"
        }))
        .unwrap();
        let request = normalized_request(&hook, "shell");
        assert_eq!(request["action"]["type"], "shell");
        assert_eq!(request["action"]["tool"], "Bash");
        assert_eq!(request["action"]["payload"]["command"], "echo ok");
        assert_eq!(
            request["action"]["payload"]["claude_hook"]["integration_level"],
            "L1_BEST_EFFORT"
        );
    }

    #[test]
    fn unsupported_tool_asks() {
        assert!(action_type_for_tool("ImaginaryTool").is_none());
    }

    #[test]
    fn evaluator_block_maps_to_deny() {
        let out = decision_from_evaluator(&serde_json::json!({
            "decision": "block",
            "rule_id": "SAFETY_001",
            "correction": "Nope"
        }));
        assert_eq!(
            out.hook_specific_output.permission_decision.as_str(),
            "deny"
        );
    }
}
