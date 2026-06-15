//! MCP proxy implementation.
//!
//! Runs as a stdio-to-stdio bridge between an MCP client (agent) and an MCP server.
//! For each `tools/call`, the payload is evaluated through Onus Core before forwarding.
//! Escalate verdicts block and create pending approvals; retries check exact approval binding.

use crate::audit::AuditTrail;
use crate::ipc::server::ServerState;
use crate::ipc::{ActionRequest, ActionResponse};
use crate::policy::PolicyEngine;
use crate::scope::ScopeTracker;
use crate::{security, Verdict};
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

fn read_json_message(reader: &mut dyn Read) -> Option<String> {
    let mut content_length: Option<usize> = None;
    let mut reader = BufReader::new(reader);

    loop {
        let mut raw_line = String::new();
        if reader.read_line(&mut raw_line).ok()? == 0 {
            return None;
        }
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
            content_length = Some(len_str.parse::<usize>().ok()?);
        }
    }

    let len = content_length?;
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).ok()?;
    String::from_utf8(body).ok()
}

fn write_json_message(writer: &mut dyn Write, json: &str) {
    let bytes = json.as_bytes();
    let header = format!("Content-Length: {}\r\n\r\n", bytes.len());
    let _ = writer.write_all(header.as_bytes());
    let _ = writer.write_all(bytes);
    let _ = writer.flush();
}

fn forward_message_to_server(
    msg: &str,
    server_stdin: &mut dyn Write,
    server_reader: &mut dyn Read,
) -> Option<String> {
    write_json_message(server_stdin, msg);
    read_json_message(server_reader)
}

fn build_approval_binding(
    audit: &Arc<Mutex<AuditTrail>>,
    session_id: &str,
    tool_name: &str,
    payload_hash: &str,
) -> security::ApprovalBinding {
    let task_contract_hash = audit
        .lock()
        .ok()
        .and_then(|audit| audit.get_task_contract(session_id).ok().flatten())
        .map(|contract| contract.canonical_hash)
        .unwrap_or_else(|| security::task_contract_hash(session_id));

    security::ApprovalBinding {
        session_id: session_id.to_string(),
        task_contract_hash,
        action_id: security::approval_action_id(session_id, tool_name, payload_hash),
        canonical_payload_hash: payload_hash.to_string(),
        policy_version: security::policy_version(),
        environment_identity: security::environment_identity(),
        expires_at: security::now_unix() + security::approval_ttl_secs(),
        approver: None,
    }
}

fn evaluate_mcp_action(state: &ServerState, request: ActionRequest) -> ActionResponse {
    crate::ipc::server::handle_action(state, request)
}

/// Run the MCP proxy. Spawns the real MCP server as a subprocess and bridges
/// stdio between the agent and the server.
pub fn run_proxy(
    server_bin: &str,
    server_args: &[String],
    db_path: Option<std::path::PathBuf>,
    approval_port: Option<u16>,
    session_id: Option<String>,
) -> anyhow::Result<()> {
    let mut server = Command::new(server_bin)
        .args(server_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn MCP server '{}': {}", server_bin, e))?;

    let mut server_stdin = server
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to open server stdin"))?;
    let mut server_stdout = server
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to open server stdout"))?;

    let session_id = session_id.unwrap_or_else(|| format!("mcp-{}", uuid::Uuid::new_v4()));
    let resolved_path = db_path.unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let audit = Arc::new(Mutex::new(
        AuditTrail::open(&resolved_path).expect("Failed to open audit database"),
    ));
    let rules =
        crate::policy::rule::load_rules(&crate::config_dir().join("rules").join("default.toml"))
            .map_err(|e| anyhow::anyhow!("Failed to load Onus rules for MCP proxy: {}", e))?;
    let state = ServerState {
        policy_engine: PolicyEngine::new(rules),
        scope_tracker: Arc::new(Mutex::new(ScopeTracker::new())),
        audit_trail: audit.clone(),
        shutting_down: Arc::new(AtomicBool::new(false)),
    };
    let mut sequence: u32 = 0;

    if let Some(port) = approval_port {
        let state = crate::approval::ApprovalState::new(audit.clone());
        let token = security::local_token();
        let token_for_thread = token.clone();
        std::thread::spawn(move || {
            if let Err(e) = crate::approval::serve(state, port, token_for_thread) {
                log::error!("Approval server exited: {}", e);
            }
        });
        log::info!(
            "Approval UI available at http://127.0.0.1:{}?token={}",
            port,
            token
        );
    }

    loop {
        let Some(msg) = read_json_message(&mut std::io::stdin().lock()) else {
            break;
        };

        let parsed: Value = match serde_json::from_str(&msg) {
            Ok(v) => v,
            Err(_) => {
                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": "Blocked by Onus: invalid MCP JSON could not be evaluated"
                    }
                });
                write_json_message(&mut std::io::stdout(), error_response.to_string().as_str());
                continue;
            }
        };

        if parsed.get("method").and_then(|m| m.as_str()) != Some("tools/call") {
            if let Some(resp) =
                forward_message_to_server(&msg, &mut server_stdin, &mut server_stdout)
            {
                write_json_message(&mut std::io::stdout(), &resp);
            }
            continue;
        }

        let tool_name = parsed["params"]["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let arguments = parsed["params"]["arguments"].clone();
        let classified = security::classify_payload(&arguments);
        let approval_binding =
            build_approval_binding(&audit, &session_id, &tool_name, &classified.payload_hash);

        sequence = sequence.saturating_add(1);
        let request = ActionRequest {
            version: 1,
            session_id: session_id.clone(),
            sequence,
            action: crate::ipc::Action {
                action_type: crate::ActionType::MCP,
                tool: tool_name.clone(),
                payload: arguments,
            },
        };

        let action_response = evaluate_mcp_action(&state, request);
        let verdict = action_response.decision.clone();
        let rule_id = action_response.rule_id.clone();
        let rule_name = action_response.rule_name.clone();
        let correction = action_response.correction.clone();

        match verdict {
            Verdict::Allow | Verdict::Warn => {
                if let Some(resp) =
                    forward_message_to_server(&msg, &mut server_stdin, &mut server_stdout)
                {
                    write_json_message(&mut std::io::stdout(), &resp);
                }
            }
            Verdict::Escalate => {
                let rid = rule_id.clone().unwrap_or_else(|| "unknown".to_string());
                let rname = rule_name.clone().unwrap_or_else(|| "unknown".to_string());
                let corr = correction.clone().unwrap_or_default();

                let approved = {
                    let audit_lock = audit.lock().unwrap();
                    audit_lock
                        .find_approved_approval(&approval_binding)
                        .ok()
                        .flatten()
                };

                if approved.is_some() {
                    if let Some(resp) =
                        forward_message_to_server(&msg, &mut server_stdin, &mut server_stdout)
                    {
                        write_json_message(&mut std::io::stdout(), &resp);
                    }
                    continue;
                }

                let rejected = {
                    let audit_lock = audit.lock().unwrap();
                    audit_lock
                        .get_pending_approvals()
                        .ok()
                        .and_then(|approvals| {
                            approvals.into_iter().find(|a| {
                                a.action_id == approval_binding.action_id && a.status == "rejected"
                            })
                        })
                };

                if rejected.is_some() {
                    let error_response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": parsed["id"],
                        "error": {
                            "code": -32002,
                            "message": format!("Rejected by user: {} - {}", rname, corr)
                        }
                    });
                    write_json_message(&mut std::io::stdout(), error_response.to_string().as_str());
                    continue;
                }

                {
                    let mut audit_lock = audit.lock().unwrap();
                    let _ = audit_lock.create_pending_approval_with_broker_decision(
                        &approval_binding,
                        "mcp",
                        Some(&tool_name),
                        &classified.canonical,
                        &rid,
                        &rname,
                        &corr,
                        action_response
                            .approval_decision
                            .as_ref()
                            .map(|decision| decision.as_str()),
                        action_response
                            .guardian_mode
                            .as_ref()
                            .map(|mode| mode.as_str()),
                        &action_response.obligations,
                        action_response.approval_reason.as_deref(),
                    );
                }

                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": parsed["id"],
                    "error": {
                        "code": -32000,
                        "message": format!("Pending human approval: {} - {}", rname, corr)
                    }
                });
                write_json_message(&mut std::io::stdout(), error_response.to_string().as_str());
            }
            Verdict::Block => {
                let rid = rule_id.clone().unwrap_or_else(|| "unknown".to_string());
                let rname = rule_name.clone().unwrap_or_else(|| "unknown".to_string());
                let corr = correction.clone().unwrap_or_default();

                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": parsed["id"],
                    "error": {
                        "code": -32001,
                        "message": format!("Blocked by Onus: {} - {} - {}", rname, rid, corr)
                    }
                });
                write_json_message(&mut std::io::stdout(), error_response.to_string().as_str());
            }
        }
    }

    let _ = server.stdin.take();
    let _ = server.wait();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_contract::{ChangeBudget, TaskContract};
    use std::path::PathBuf;

    fn temp_db(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("onus-mcp-{}-{}.db", name, uuid::Uuid::new_v4()))
    }

    fn test_state(
        db_path: &std::path::Path,
        session_id: &str,
    ) -> (ServerState, Arc<Mutex<AuditTrail>>) {
        let mut audit = AuditTrail::open(db_path).unwrap();
        audit
            .start_session(
                session_id,
                "mcp-test",
                None,
                "MCP contract test",
                "/workspace",
            )
            .unwrap();
        audit
            .save_task_contract(&TaskContract {
                schema_version: 1,
                session_id: session_id.to_string(),
                original_prompt: "Use an MCP tool with approval.".to_string(),
                normalized_objective: "Require approval for MCP tool calls.".to_string(),
                allowed_paths: vec![],
                allowed_resources: vec![],
                protected_paths: vec![],
                protected_resources: vec![],
                required_evidence: vec![],
                forbidden_actions: vec![],
                approval_required_actions: vec!["mcp".to_string()],
                change_budget: ChangeBudget {
                    max_files_changed: 5,
                    max_actions: 20,
                },
                environment_identity: "test-env".to_string(),
                policy_version: "test-policy".to_string(),
                canonical_hash: String::new(),
            })
            .unwrap();
        let audit = Arc::new(Mutex::new(audit));
        let state = ServerState {
            policy_engine: PolicyEngine::new(vec![]),
            scope_tracker: Arc::new(Mutex::new(ScopeTracker::new())),
            audit_trail: audit.clone(),
            shutting_down: Arc::new(AtomicBool::new(false)),
        };
        (state, audit)
    }

    #[test]
    fn mcp_tool_call_uses_persisted_task_contract_and_exact_approval_binding() {
        let db_path = temp_db("contract");
        let session_id = "mcp-test-session";
        let (state, audit) = test_state(&db_path, session_id);
        let payload = serde_json::json!({"query": "select 1"});
        let classified = security::classify_payload(&payload);
        let binding =
            build_approval_binding(&audit, session_id, "db.query", &classified.payload_hash);

        let response = evaluate_mcp_action(
            &state,
            ActionRequest {
                version: 1,
                session_id: session_id.to_string(),
                sequence: 1,
                action: crate::ipc::Action {
                    action_type: crate::ActionType::MCP,
                    tool: "db.query".to_string(),
                    payload: payload.clone(),
                },
            },
        );
        assert_eq!(response.decision, Verdict::Escalate);
        assert_eq!(
            response.rule_id.as_deref(),
            Some("ONUS_CONTRACT_APPROVAL_REQUIRED")
        );

        {
            let mut audit = audit.lock().unwrap();
            audit
                .create_pending_approval(
                    &binding,
                    "mcp",
                    Some("db.query"),
                    &classified.canonical,
                    "ONUS_CONTRACT_APPROVAL_REQUIRED",
                    "contract-approval-required",
                    "approval required",
                )
                .unwrap();
            assert!(audit.approve_action(&binding.action_id, "alice").unwrap());
            assert!(audit.find_approved_approval(&binding).unwrap().is_some());
        }

        let changed = serde_json::json!({"query": "drop table users"});
        let changed_classified = security::classify_payload(&changed);
        let changed_binding = build_approval_binding(
            &audit,
            session_id,
            "db.query",
            &changed_classified.payload_hash,
        );
        assert!(audit
            .lock()
            .unwrap()
            .find_approved_approval(&changed_binding)
            .unwrap()
            .is_none());

        let _ = std::fs::remove_file(db_path);
    }
}
