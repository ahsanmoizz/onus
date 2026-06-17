//! `onus cursor-hook` — Cursor IDE PreToolUse hook subcommand.
//!
//! Cursor hook protocol: reads JSON from stdin, returns
//! `{ "allowed": bool, "messages": [...] }`.
//!
//! L1 BEST-EFFORT — cooperative hook model.

use clap::Args;
use serde_json::Value;

#[derive(Args)]
pub struct CursorHookArgs;

/// Run the cursor-hook command: read a tool-use JSON from stdin, evaluate it
/// against Onus policy, and print the verdict as JSON.
pub fn run(_args: CursorHookArgs) -> anyhow::Result<()> {
    let stdin = std::io::read_to_string(std::io::stdin())?;
    let input: Value = serde_json::from_str(&stdin)
        .map_err(|e| anyhow::anyhow!("Invalid JSON on stdin: {e}"))?;

    let tool_name = input
        .get("tool")
        .and_then(Value::as_str)
        .unwrap_or("unknown");

    let args = input
        .get("args")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

    // Build an evaluation payload and send it through Onus Core.
    let verdict = evaluate_tool_use(tool_name, &args);

    // Return verdict in Cursor hook format.
    let response = serde_json::json!({
        "allowed": verdict.allowed,
        "messages": [{
            "type": "text",
            "text": &verdict.reason
        }]
    });

    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

struct Verdict {
    allowed: bool,
    reason: String,
}

fn evaluate_tool_use(tool_name: &str, _args: &str) -> Verdict {
    // L1 BEST-EFFORT: cooperative hook model. Allowed by default.
    // Future: route through Onus Core evaluator for L2 enforcement.
    //
    // Block only tools that are always denied (e.g., dangerous by policy).
    match tool_name {
        "bash" | "execute_command" => Verdict {
            allowed: true,
            reason: format!("L1 BEST-EFFORT: tool '{tool_name}' allowed by cooperative hook"),
        },
        _ => Verdict {
            allowed: true,
            reason: format!("L1 BEST-EFFORT: tool '{tool_name}' allowed by cooperative hook"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_hook_allows_unknown_tool() {
        let v = evaluate_tool_use("unknown_tool", "");
        assert!(v.allowed);
    }

    #[test]
    fn test_cursor_hook_allows_bash() {
        let v = evaluate_tool_use("bash", "ls -la");
        assert!(v.allowed);
    }

    #[test]
    fn test_cursor_hook_returns_reason() {
        let v = evaluate_tool_use("bash", "");
        assert!(!v.reason.is_empty());
        assert!(v.reason.contains("L1 BEST-EFFORT"));
    }
}
