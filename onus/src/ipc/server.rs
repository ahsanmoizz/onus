//! IPC server — listens on Unix domain socket or Windows named pipe,
//! accepts length-prefixed JSON messages, evaluates actions, returns verdicts.

use crate::audit::AuditTrail;
use crate::ipc::{ActionRequest, ActionResponse, DaemonMessage, DaemonResponse, ServerCommand, SessionCommand};
use crate::policy::rule::RuleSummary;
use crate::policy::PolicyEngine;
use crate::scope::ScopeTracker;
use crate::Verdict;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::ipc::protocol;

/// Shared application state accessible by the IPC server.
pub struct ServerState {
    pub policy_engine: PolicyEngine,
    pub scope_tracker: Arc<Mutex<ScopeTracker>>,
    pub audit_trail: Arc<Mutex<AuditTrail>>,
    pub shutting_down: Arc<std::sync::atomic::AtomicBool>,
}

impl ServerState {
    pub fn new(
        policy_engine: PolicyEngine,
        audit_trail: AuditTrail,
    ) -> Self {
        Self {
            policy_engine,
            scope_tracker: Arc::new(Mutex::new(ScopeTracker::new())),
            audit_trail: Arc::new(Mutex::new(audit_trail)),
            shutting_down: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

/// Handle a single action request and return the response.
pub fn handle_action(state: &ServerState, request: ActionRequest) -> ActionResponse {
    let start = Instant::now();

    // Check scope rules first (need scope tracker state)
    let scope_verdict = {
        let scope = state.scope_tracker.lock().unwrap();
        scope.evaluate(&request)
    };

    // Check policy rules
    let policy_verdict = state.policy_engine.evaluate(&request);

    // Retry escalation: if agent has been blocked 3+ times, escalate to force human review
    let retry_count = {
        let scope = state.scope_tracker.lock().unwrap();
        scope.retry_count(&request.session_id)
    };
    let final_verdict = if retry_count >= 3 && policy_verdict != Verdict::Allow && scope_verdict != Verdict::Allow {
        Verdict::Escalate
    } else {
        // Most restrictive wins: Escalate > Block > Warn > Allow
        most_restrictive(scope_verdict, policy_verdict)
    };

    let (rule_id, rule_name, correction, reversibility) = match &final_verdict {
        Verdict::Block | Verdict::Warn | Verdict::Escalate => {
            let result = state.policy_engine.last_match();
            (
                result.as_ref().map(|r| r.id.clone()),
                result.as_ref().map(|r| r.name.clone()),
                result.as_ref().map(|r| r.correction.clone()),
                result.as_ref().map(|r| r.reversibility.clone()),
            )
        }
        Verdict::Allow => (None, None, None, None),
    };

    let latency_us = start.elapsed().as_micros() as u64;

    // Record in audit trail
    {
        let mut audit = state.audit_trail.lock().unwrap();
        if let Err(e) = audit.record_action(
            &request.session_id,
            request.sequence as u64,
            &request.action.action_type.to_string(),
            &request.action.tool,
            &serde_json::to_string(&request.action.payload).unwrap_or_default(),
            &final_verdict,
            rule_id.as_deref(),
            correction.as_deref(),
            latency_us,
        ) {
            log::error!("Audit record failed: {}", e);
        }
    }

    // Update scope tracker with allowed actions
    if final_verdict == Verdict::Allow || final_verdict == Verdict::Warn {
        if let Ok(mut scope) = state.scope_tracker.lock() {
            scope.record_action(&request);
        }
    }

    // Increment retry count on blocked actions for escalation tracking
    if final_verdict == Verdict::Block || final_verdict == Verdict::Escalate {
        if let Ok(mut scope) = state.scope_tracker.lock() {
            scope.increment_retry(&request.session_id);
        }
    }

    // Transition session status for terminal verdicts
    if final_verdict == Verdict::Escalate {
        if let Ok(mut audit) = state.audit_trail.lock() {
            let _ = audit.update_session_status(&request.session_id, "escalated");
        }
    } else if final_verdict == Verdict::Block {
        if let Ok(mut audit) = state.audit_trail.lock() {
            let _ = audit.update_session_status(&request.session_id, "aborted");
        }
    }

    ActionResponse {
        version: 1,
        session_id: request.session_id,
        sequence: request.sequence,
        decision: final_verdict,
        rule_id,
        rule_name,
        correction,
        latency_us,
        reversibility,
    }
}

/// Handle a session management command.
pub fn handle_session(state: &ServerState, command: SessionCommand) -> crate::ipc::SessionResponse {
    match command {
        SessionCommand::Start {
            session_id,
            agent_name,
            agent_version,
            task_description,
            workspace_root,
            declared_files,
            allowed_paths,
        } => {
            // Start audit session
            {
                let mut audit = state.audit_trail.lock().unwrap();
                if let Err(e) = audit.start_session(
                    &session_id,
                    &agent_name,
                    agent_version.as_deref(),
                    &task_description,
                    &workspace_root,
                ) {
                    return crate::ipc::SessionResponse {
                        status: "error".into(),
                        error: Some(format!("Failed to start audit session: {}", e)),
                    };
                }
            }

            // Initialize scope
            {
                let mut scope = state.scope_tracker.lock().unwrap();
                scope.start_session(
                    session_id.clone(),
                    task_description,
                    workspace_root,
                    declared_files,
                    allowed_paths,
                );
            }

            crate::ipc::SessionResponse {
                status: "ok".into(),
                error: None,
            }
        }
        SessionCommand::End { session_id } => {
            // End scope session
            {
                let mut scope = state.scope_tracker.lock().unwrap();
                scope.end_session(&session_id);
            }

            // End audit session
            {
                let mut audit = state.audit_trail.lock().unwrap();
                if let Err(e) = audit.end_session(&session_id) {
                    return crate::ipc::SessionResponse {
                        status: "error".into(),
                        error: Some(format!("Failed to end audit session: {}", e)),
                    };
                }
            }

            crate::ipc::SessionResponse {
                status: "ok".into(),
                error: None,
            }
        }
    }
}

/// Handle a single client connection — reads messages in a loop and writes responses.
pub fn handle_client(
    state: &ServerState,
    mut stream: impl Read + Write,
) -> std::io::Result<()> {
    loop {
        // Attempt to read a message. On EOF or error, disconnect.
        let buf = match protocol::read_message_raw(&mut stream) {
            Ok(buf) => buf,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                log::warn!("IPC read error: {}", e);
                break;
            }
        };

        // Try to parse as DaemonMessage first (new clients),
        // fall back to ActionRequest (legacy clients).
        let response = if let Ok(dm) = serde_json::from_slice::<DaemonMessage>(&buf) {
            let resp = handle_daemon_message(state, dm);
            serde_json::to_vec(&resp).unwrap_or_default()
        } else if let Ok(action_req) = serde_json::from_slice::<ActionRequest>(&buf) {
            let action_resp = handle_action(state, action_req);
            serde_json::to_vec(&action_resp).unwrap_or_default()
        } else {
            log::warn!("IPC: unknown message format, ignoring");
            continue;
        };

        protocol::write_message_raw(&mut stream, &response)?;
    }
    Ok(())
}

/// Dispatch a DaemonMessage to the appropriate handler.
fn handle_daemon_message(state: &ServerState, msg: DaemonMessage) -> DaemonResponse {
    // Handle shutdown command before building response.
    if let Some(ServerCommand::Shutdown) = &msg.server_command {
        request_shutdown(state);
        log::info!("Daemon shutdown requested via IPC");
    }

    DaemonResponse {
        action_response: msg.action_request.map(|req| handle_action(state, req)),
        session_response: msg.session_command.map(|cmd| handle_session(state, cmd)),
        status_response: msg.server_command.as_ref().and_then(|cmd| {
            if matches!(cmd, ServerCommand::Status) {
                let session_count = state.scope_tracker.lock().unwrap().session_count();
                let audit = state.audit_trail.lock().unwrap();
                let summary = audit.get_status().ok();
                Some(crate::ipc::StatusResponse {
                    daemon_running: true,
                    active_sessions: session_count,
                    total_actions: summary.as_ref().map(|s| s.total_actions).unwrap_or(0),
                    total_blocks: summary.as_ref().map(|s| s.blocked_actions).unwrap_or(0),
                    version: crate::VERSION.to_string(),
                })
            } else {
                None
            }
        }),
        rules_response: msg.server_command.as_ref().and_then(|cmd| {
            if matches!(cmd, ServerCommand::Rules) {
                let rules = state.policy_engine.rules();
                let summaries: Vec<RuleSummary> = rules.iter().map(|r| RuleSummary::from(r)).collect();
                Some(crate::ipc::RulesResponse { rules: summaries })
            } else {
                None
            }
        }),
    }
}

/// Check if the server has been asked to shut down.
pub fn is_shutting_down(state: &ServerState) -> bool {
    state.shutting_down.load(std::sync::atomic::Ordering::Relaxed)
}

/// Mark the server for graceful shutdown.
pub fn request_shutdown(state: &ServerState) {
    state.shutting_down.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn most_restrictive(a: Verdict, b: Verdict) -> Verdict {
    use Verdict::*;
    match (&a, &b) {
        (Escalate, _) | (_, Escalate) => Escalate,
        (Block, _) | (_, Block) => Block,
        (Warn, _) | (_, Warn) => Warn,
        _ => Allow,
    }
}
