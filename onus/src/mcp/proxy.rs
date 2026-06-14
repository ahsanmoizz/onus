//! MCP proxy implementation.
//!
//! Runs as a stdio-to-stdio bridge between an MCP client (agent) and an MCP server.
//! For each `tools/call`, the payload is evaluated through Onus Core before forwarding.
//! Escalate verdicts block and create pending approvals; retries check approval status.

use crate::audit::AuditTrail;
use crate::ipc::ActionRequest;
use crate::Verdict;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

fn read_json_message(reader: &mut dyn Read) -> Option<String> {
    let mut content_length: Option<usize> = None;
    let mut reader = BufReader::new(reader);

    // Read headers
    loop {
        let mut raw_line = String::new();
        if reader.read_line(&mut raw_line).ok()? == 0 {
            return None;
        }
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            // End of headers
            break;
        }
        if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
            content_length = Some(len_str.parse::<usize>().ok()?);
        }
    }

    // Read body
    let len = content_length?;
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).ok()?;
    Some(String::from_utf8(body).ok()?)
}

fn write_json_message(writer: &mut dyn Write, json: &str) {
    let bytes = json.as_bytes();
    let header = format!("Content-Length: {}\r\n\r\n", bytes.len());
    let _ = writer.write_all(header.as_bytes());
    let _ = writer.write_all(bytes);
    let _ = writer.flush();
}

// Forward a non-tools/call message to server, return response
fn forward_message_to_server(
    msg: &str,
    server_stdin: &mut dyn Write,
    server_reader: &mut dyn Read,
) -> Option<String> {
    write_json_message(server_stdin, msg);
    read_json_message(server_reader)
}

/// Run the MCP proxy. Spawns the real MCP server as a subprocess and bridges
/// stdio between the agent (connected via stdin/stdout) and the server.
/// Optionally starts an approval web server on the given port for handling
/// Escalate verdicts.
pub fn run_proxy(
    server_bin: &str,
    server_args: &[String],
    db_path: Option<std::path::PathBuf>,
    approval_port: Option<u16>,
) -> anyhow::Result<()> {
    // 1. Spawn the real MCP server as a subprocess.
    let mut server = Command::new(server_bin)
        .args(server_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn MCP server '{}': {}", server_bin, e))?;

    let mut server_stdin = server.stdin.take()
        .ok_or_else(|| anyhow::anyhow!("Failed to open server stdin"))?;
    let mut server_stdout = server.stdout.take()
        .ok_or_else(|| anyhow::anyhow!("Failed to open server stdout"))?;

    // Session tracking for Onus evaluation.
    let session_id = format!("mcp-{}", uuid::Uuid::new_v4());

    // Audit trail for approvals (must be Send for background thread safety)
    let resolved_path = db_path.unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let audit = Arc::new(Mutex::new(
        AuditTrail::open(&resolved_path).expect("Failed to open audit database")
    ));

    // Start the approval web server on a background thread if port is specified
    if let Some(port) = approval_port {
        let state = crate::approval::ApprovalState::new(audit.clone());
        std::thread::spawn(move || {
            if let Err(e) = crate::approval::serve(state, port) {
                log::error!("Approval server exited: {}", e);
            }
        });
        log::info!("Approval UI available at http://127.0.0.1:{}", port);
    }

    loop {
        let Some(msg) = read_json_message(&mut std::io::stdin().lock()) else {
            break;
        };

        let parsed: Value = match serde_json::from_str(&msg) {
            Ok(v) => v,
            Err(_) => {
                // Parse error: forward anyway, protocol-level messages
                if let Some(resp) = forward_message_to_server(&msg, &mut server_stdin, &mut server_stdout) {
                    write_json_message(&mut std::io::stdout(), &resp);
                }
                continue;
            }
        };

        // Check if this is a tools/call
        let is_tools_call = parsed.get("method").and_then(|m| m.as_str()) == Some("tools/call");

        if !is_tools_call {
            // Non-tools/call message — forward directly
            if let Some(resp) = forward_message_to_server(&msg, &mut server_stdin, &mut server_stdout) {
                write_json_message(&mut std::io::stdout(), &resp);
            }
            continue;
        }

        // tools/call: evaluate through Onus
        let tool_name = parsed["params"]["name"].as_str().unwrap_or("unknown").to_string();
        let arguments = parsed["params"]["arguments"].clone();

        // Build ActionRequest
        let request = ActionRequest {
            version: 1,
            session_id: session_id.clone(),
            sequence: 0,
            action: crate::ipc::Action {
                action_type: crate::ActionType::MCP,
                tool: tool_name.clone(),
                payload: arguments.clone(),
            },
        };

        // Evaluate
        let (verdict, rule_id, rule_name, correction) = crate::evaluate_request(&request);

        match verdict {
            Verdict::Allow | Verdict::Warn => {
                // Forward the tool call to the real server
                if let Some(resp) = forward_message_to_server(&msg, &mut server_stdin, &mut server_stdout) {
                    write_json_message(&mut std::io::stdout(), &resp);
                }
            }
            Verdict::Escalate => {
                let rid = rule_id.clone().unwrap_or_else(|| "unknown".to_string());
                let rname = rule_name.clone().unwrap_or_else(|| "unknown".to_string());
                let corr = correction.clone().unwrap_or_else(|| "".to_string());

                // Check if there's already a pending approval for this session+tool
                let existing = {
                    let audit_lock = audit.lock().unwrap();
                    audit_lock.get_pending_approvals().ok()
                        .and_then(|approvals| {
                            approvals.into_iter().find(|a| {
                                a.session_id == session_id && a.tool_name.as_deref() == Some(&tool_name)
                            })
                        })
                };

                match existing {
                    Some(approval) if approval.status == "approved" => {
                        // Already approved — forward the tool call to the real server
                        if let Some(resp) = forward_message_to_server(&msg, &mut server_stdin, &mut server_stdout) {
                            write_json_message(&mut std::io::stdout(), &resp);
                        }
                    }
                    Some(approval) if approval.status == "rejected" => {
                        // Was rejected — return a permanent rejection error
                        let error_response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": parsed["id"],
                            "error": {
                                "code": -32002,
                                "message": format!("Rejected by user: {} — {}", rname, corr)
                            }
                        });
                        write_json_message(&mut std::io::stdout(), &error_response.to_string().as_str());
                    }
                    _ => {
                        // No existing approval or still pending — create one if new
                        let action_id = uuid::Uuid::new_v4().to_string();

                        {
                            let mut audit_lock = audit.lock().unwrap();
                            let _ = audit_lock.create_pending_approval(
                                &action_id,
                                &session_id,
                                "mcp",
                                Some(&tool_name),
                                &serde_json::to_string(&arguments).unwrap_or_default(),
                                &rid,
                                &rname,
                                &corr,
                            );
                        }

                        // Return MCP error: pending human approval
                        let error_response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": parsed["id"],
                            "error": {
                                "code": -32000,
                                "message": format!("Pending human approval: {} — {}", rname, corr)
                            }
                        });
                        write_json_message(&mut std::io::stdout(), &error_response.to_string().as_str());
                    }
                }
            }
            Verdict::Block => {
                let rid = rule_id.clone().unwrap_or_else(|| "unknown".to_string());
                let rname = rule_name.clone().unwrap_or_else(|| "unknown".to_string());
                let corr = correction.clone().unwrap_or_else(|| "".to_string());

                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": parsed["id"],
                    "error": {
                        "code": -32001,
                        "message": format!("Blocked by Onus: {} — {} — {}", rname, rid, corr)
                    }
                });
                write_json_message(&mut std::io::stdout(), &error_response.to_string().as_str());
            }
        }
    }

    // Cleanup
    let _ = server.stdin.take();
    let _ = server.wait();
    Ok(())
}
