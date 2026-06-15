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
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const DEFAULT_RESPONSE_TIMEOUT_MS: u64 = 5_000;
const DEFAULT_MAX_RESPONSE_BYTES: usize = 1024 * 1024;

#[derive(Debug)]
enum McpReadError {
    MissingContentLength,
    MalformedHeader(String),
    InvalidContentLength(String),
    ResponseTooLarge { len: usize, max: usize },
    Io(String),
    Utf8(String),
}

impl std::fmt::Display for McpReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingContentLength => write!(f, "missing Content-Length header"),
            Self::MalformedHeader(line) => write!(f, "malformed header: {}", line),
            Self::InvalidContentLength(value) => write!(f, "invalid Content-Length: {}", value),
            Self::ResponseTooLarge { len, max } => {
                write!(f, "message size {} exceeds limit {}", len, max)
            }
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::Utf8(err) => write!(f, "UTF-8 error: {}", err),
        }
    }
}

fn read_json_message(
    reader: &mut dyn BufRead,
    max_response_bytes: usize,
) -> Result<Option<String>, McpReadError> {
    let mut content_length: Option<usize> = None;

    loop {
        let mut raw_line = String::new();
        if reader
            .read_line(&mut raw_line)
            .map_err(|e| McpReadError::Io(e.to_string()))?
            == 0
        {
            return Ok(None);
        }
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
            let len = len_str
                .parse::<usize>()
                .map_err(|_| McpReadError::InvalidContentLength(len_str.to_string()))?;
            if len > max_response_bytes {
                return Err(McpReadError::ResponseTooLarge {
                    len,
                    max: max_response_bytes,
                });
            }
            content_length = Some(len);
        } else if !trimmed.contains(':') {
            return Err(McpReadError::MalformedHeader(trimmed.to_string()));
        }
    }

    let len = content_length.ok_or(McpReadError::MissingContentLength)?;
    let mut body = vec![0u8; len];
    reader
        .read_exact(&mut body)
        .map_err(|e| McpReadError::Io(e.to_string()))?;
    String::from_utf8(body)
        .map(Some)
        .map_err(|e| McpReadError::Utf8(e.to_string()))
}

fn write_json_message(writer: &mut dyn Write, json: &str) {
    let bytes = json.as_bytes();
    let header = format!("Content-Length: {}\r\n\r\n", bytes.len());
    let _ = writer.write_all(header.as_bytes());
    let _ = writer.write_all(bytes);
    let _ = writer.flush();
}

#[derive(Clone)]
struct McpServerIdentity {
    command: String,
    args: Vec<String>,
    identity_hash: String,
}

impl McpServerIdentity {
    fn new(server_bin: &str, server_args: &[String]) -> Self {
        let identity_hash = security::sha256_hex(&format!(
            "{}\n{}",
            server_bin,
            serde_json::to_string(server_args).unwrap_or_default()
        ));
        Self {
            command: server_bin.to_string(),
            args: server_args.to_vec(),
            identity_hash,
        }
    }

    fn value(&self) -> Value {
        serde_json::json!({
            "command": self.command,
            "args_hash": security::sha256_hex(&serde_json::to_string(&self.args).unwrap_or_default()),
            "identity_hash": self.identity_hash,
        })
    }
}

#[derive(Debug)]
enum ForwardError {
    Timeout,
    ServerClosed,
    Read(McpReadError),
    Write(String),
}

fn forward_message_to_server(
    msg: &str,
    server_stdin: &mut dyn Write,
    server_rx: &mpsc::Receiver<Result<Option<String>, McpReadError>>,
    timeout: Duration,
) -> Result<String, ForwardError> {
    write_json_message(server_stdin, msg);
    server_stdin
        .flush()
        .map_err(|e| ForwardError::Write(e.to_string()))?;
    match server_rx.recv_timeout(timeout) {
        Ok(Ok(Some(resp))) => Ok(resp),
        Ok(Ok(None)) => Err(ForwardError::ServerClosed),
        Ok(Err(err)) => Err(ForwardError::Read(err)),
        Err(mpsc::RecvTimeoutError::Timeout) => Err(ForwardError::Timeout),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(ForwardError::ServerClosed),
    }
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

fn normalized_tool_payload(
    server_identity: &McpServerIdentity,
    method: &str,
    tool_name: &str,
    arguments: Value,
) -> Value {
    serde_json::json!({
        "protocol": "mcp",
        "method": method,
        "server_identity": server_identity.value(),
        "tool": {
            "name": tool_name,
            "arguments": arguments,
        }
    })
}

fn receipt_from_response(action_response: &ActionResponse) -> Value {
    serde_json::json!({
        "session_id": action_response.session_id,
        "sequence": action_response.sequence,
        "decision": action_response.decision.to_string(),
        "action_id": action_response.action_id,
        "canonical_payload_hash": action_response.canonical_payload_hash,
        "rule_id": action_response.rule_id,
        "approval_decision": action_response.approval_decision.as_ref().map(|d| d.as_str()),
        "guardian_mode": action_response.guardian_mode.as_ref().map(|m| m.as_str()),
    })
}

fn json_rpc_error(id: Value, code: i64, message: impl Into<String>, data: Option<Value>) -> String {
    let mut error = serde_json::json!({
        "code": code,
        "message": message.into(),
    });
    if let Some(data) = data {
        error["data"] = data;
    }
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": error,
    })
    .to_string()
}

fn gateway_metadata(
    server_identity: &McpServerIdentity,
    max_response_bytes: usize,
    timeout_ms: u64,
) -> Value {
    serde_json::json!({
        "gateway": "onus-mcp-proxy",
        "server_identity": server_identity.value(),
        "limits": {
            "max_response_bytes": max_response_bytes,
            "response_timeout_ms": timeout_ms,
        },
        "direct_server_bypass": "Onus protects only MCP traffic routed through this proxy. Direct client connections to the upstream server bypass Onus."
    })
}

fn annotate_gateway_response(
    response: &str,
    server_identity: &McpServerIdentity,
    max_response_bytes: usize,
    timeout_ms: u64,
) -> String {
    let Ok(mut value) = serde_json::from_str::<Value>(response) else {
        return response.to_string();
    };
    if let Some(result) = value.get_mut("result").and_then(|v| v.as_object_mut()) {
        result.insert(
            "_onus_gateway".to_string(),
            gateway_metadata(server_identity, max_response_bytes, timeout_ms),
        );
    }
    value.to_string()
}

fn annotate_tool_response(response: &str, receipt: Value) -> String {
    let Ok(mut value) = serde_json::from_str::<Value>(response) else {
        return response.to_string();
    };
    if let Some(result) = value.get_mut("result").and_then(|v| v.as_object_mut()) {
        result.insert("_onus_receipt".to_string(), receipt);
    }
    value.to_string()
}

fn enforce_response_size(response: &str, max_response_bytes: usize) -> Result<(), McpReadError> {
    let len = response.len();
    if len > max_response_bytes {
        Err(McpReadError::ResponseTooLarge {
            len,
            max: max_response_bytes,
        })
    } else {
        Ok(())
    }
}

fn forward_error_response(parsed: &Value, err: ForwardError) -> String {
    let message = match err {
        ForwardError::Timeout => {
            "Blocked by Onus: upstream MCP server response timed out".to_string()
        }
        ForwardError::ServerClosed => "Blocked by Onus: upstream MCP server closed".to_string(),
        ForwardError::Read(err) => {
            format!("Blocked by Onus: malformed upstream MCP response: {}", err)
        }
        ForwardError::Write(err) => format!(
            "Blocked by Onus: failed to write to upstream MCP server: {}",
            err
        ),
    };
    json_rpc_error(
        parsed.get("id").cloned().unwrap_or(Value::Null),
        -32098,
        message,
        None,
    )
}

/// Run the MCP proxy. Spawns the real MCP server as a subprocess and bridges
/// stdio between the agent and the server.
pub fn run_proxy(
    server_bin: &str,
    server_args: &[String],
    db_path: Option<std::path::PathBuf>,
    approval_port: Option<u16>,
    session_id: Option<String>,
    response_timeout_ms: u64,
    max_response_bytes: usize,
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
    let server_stdout = server
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to open server stdout"))?;
    let server_identity = McpServerIdentity::new(server_bin, server_args);
    let response_timeout_ms = if response_timeout_ms == 0 {
        DEFAULT_RESPONSE_TIMEOUT_MS
    } else {
        response_timeout_ms
    };
    let max_response_bytes = if max_response_bytes == 0 {
        DEFAULT_MAX_RESPONSE_BYTES
    } else {
        max_response_bytes
    };
    let response_timeout = Duration::from_millis(response_timeout_ms);
    let (server_tx, server_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(server_stdout);
        loop {
            let message = read_json_message(&mut reader, max_response_bytes);
            let should_stop = matches!(message, Ok(None) | Err(_));
            if server_tx.send(message).is_err() || should_stop {
                break;
            }
        }
    });

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

    let stdin = std::io::stdin();
    let mut client_reader = BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout();

    loop {
        let msg = match read_json_message(&mut client_reader, max_response_bytes) {
            Ok(Some(msg)) => msg,
            Ok(None) => break,
            Err(err) => {
                let error = json_rpc_error(
                    Value::Null,
                    -32700,
                    format!("Blocked by Onus: malformed MCP client message: {}", err),
                    None,
                );
                write_json_message(&mut stdout, &error);
                continue;
            }
        };

        let parsed: Value = match serde_json::from_str(&msg) {
            Ok(v) => v,
            Err(_) => {
                let error_response = json_rpc_error(
                    Value::Null,
                    -32700,
                    "Blocked by Onus: invalid MCP JSON could not be evaluated",
                    None,
                );
                write_json_message(&mut stdout, &error_response);
                continue;
            }
        };

        if parsed.get("method").and_then(|m| m.as_str()) != Some("tools/call") {
            match forward_message_to_server(&msg, &mut server_stdin, &server_rx, response_timeout) {
                Ok(resp) => {
                    let resp = if matches!(
                        parsed.get("method").and_then(|m| m.as_str()),
                        Some("initialize") | Some("tools/list")
                    ) {
                        annotate_gateway_response(
                            &resp,
                            &server_identity,
                            max_response_bytes,
                            response_timeout_ms,
                        )
                    } else {
                        resp
                    };
                    if let Err(err) = enforce_response_size(&resp, max_response_bytes) {
                        let error_response = json_rpc_error(
                            parsed.get("id").cloned().unwrap_or(Value::Null),
                            -32097,
                            format!(
                                "Blocked by Onus: upstream response exceeded size limit: {}",
                                err
                            ),
                            None,
                        );
                        write_json_message(&mut stdout, &error_response);
                    } else {
                        write_json_message(&mut stdout, &resp);
                    }
                }
                Err(err) => {
                    let error_response = forward_error_response(&parsed, err);
                    write_json_message(&mut stdout, &error_response);
                }
            }
            continue;
        }

        let tool_name = parsed["params"]["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let arguments = parsed["params"]["arguments"].clone();
        let normalized_payload =
            normalized_tool_payload(&server_identity, "tools/call", &tool_name, arguments);
        let classified = security::classify_payload(&normalized_payload);
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
                payload: normalized_payload,
            },
        };

        let action_response = evaluate_mcp_action(&state, request);
        let verdict = action_response.decision.clone();
        let rule_id = action_response.rule_id.clone();
        let rule_name = action_response.rule_name.clone();
        let correction = action_response.correction.clone();

        match verdict {
            Verdict::Allow | Verdict::Warn => {
                match forward_message_to_server(
                    &msg,
                    &mut server_stdin,
                    &server_rx,
                    response_timeout,
                ) {
                    Ok(resp) => {
                        let resp =
                            annotate_tool_response(&resp, receipt_from_response(&action_response));
                        if let Err(err) = enforce_response_size(&resp, max_response_bytes) {
                            let error_response = json_rpc_error(
                                parsed.get("id").cloned().unwrap_or(Value::Null),
                                -32097,
                                format!(
                                    "Blocked by Onus: upstream response exceeded size limit: {}",
                                    err
                                ),
                                Some(receipt_from_response(&action_response)),
                            );
                            write_json_message(&mut stdout, &error_response);
                        } else {
                            write_json_message(&mut stdout, &resp);
                        }
                    }
                    Err(err) => {
                        let error_response = forward_error_response(&parsed, err);
                        write_json_message(&mut stdout, &error_response);
                    }
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
                    match forward_message_to_server(
                        &msg,
                        &mut server_stdin,
                        &server_rx,
                        response_timeout,
                    ) {
                        Ok(resp) => {
                            let resp = annotate_tool_response(
                                &resp,
                                receipt_from_response(&action_response),
                            );
                            if let Err(err) = enforce_response_size(&resp, max_response_bytes) {
                                let error_response = json_rpc_error(
                                    parsed.get("id").cloned().unwrap_or(Value::Null),
                                    -32097,
                                    format!(
                                        "Blocked by Onus: upstream response exceeded size limit: {}",
                                        err
                                    ),
                                    Some(receipt_from_response(&action_response)),
                                );
                                write_json_message(&mut stdout, &error_response);
                            } else {
                                write_json_message(&mut stdout, &resp);
                            }
                        }
                        Err(err) => {
                            let error_response = forward_error_response(&parsed, err);
                            write_json_message(&mut stdout, &error_response);
                        }
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
                            "message": format!("Rejected by user: {} - {}", rname, corr),
                            "data": receipt_from_response(&action_response)
                        }
                    });
                    write_json_message(&mut stdout, error_response.to_string().as_str());
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
                        "message": format!("Pending human approval: {} - {}", rname, corr),
                        "data": receipt_from_response(&action_response)
                    }
                });
                write_json_message(&mut stdout, error_response.to_string().as_str());
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
                        "message": format!("Blocked by Onus: {} - {} - {}", rname, rid, corr),
                        "data": receipt_from_response(&action_response)
                    }
                });
                write_json_message(&mut stdout, error_response.to_string().as_str());
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
