//! IPC server — listens on Unix domain socket or Windows named pipe,
//! accepts length-prefixed JSON messages, evaluates actions, returns verdicts.

use crate::approval_broker::{self, ApprovalDecision, BrokerInput};
use crate::audit::AuditTrail;
use crate::ipc::{
    ActionRequest, ActionResponse, DaemonMessage, DaemonResponse, ServerCommand, SessionCommand,
};
use crate::policy::rule::RuleSummary;
use crate::policy::PolicyEngine;
use crate::scope::ScopeTracker;
use crate::semantic::{
    self, ActionReviewRequest, ConfiguredSemanticReviewer, SemanticFallbackPolicy, SemanticReviewer,
};
use crate::task_contract::{self, ContractViolation};
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
    pub fn new(policy_engine: PolicyEngine, audit_trail: AuditTrail) -> Self {
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

    let (task_contract, contract_violation) = evaluate_contract(state, &request);

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
    let contract_verdict = contract_violation
        .as_ref()
        .map(|v| v.verdict.clone())
        .unwrap_or(Verdict::Allow);

    let mut final_verdict = if retry_count >= 3
        && policy_verdict != Verdict::Allow
        && scope_verdict != Verdict::Allow
    {
        Verdict::Escalate
    } else {
        // Most restrictive wins: Block > Escalate > Warn > Allow.
        most_restrictive(
            most_restrictive(scope_verdict, policy_verdict),
            contract_verdict,
        )
    };

    let (mut rule_id, mut rule_name, mut correction, mut reversibility) = if let Some(violation) =
        contract_violation
            .as_ref()
            .filter(|v| v.verdict == final_verdict)
    {
        (
            Some(violation.rule_id.clone()),
            Some(violation.rule_name.clone()),
            Some(violation.correction.clone()),
            None,
        )
    } else {
        match &final_verdict {
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
        }
    };

    apply_semantic_risk_review(
        &request,
        &mut final_verdict,
        &mut rule_id,
        &mut rule_name,
        &mut correction,
        &mut reversibility,
    );

    let broker_outcome = approval_broker::decide(BrokerInput {
        request: &request,
        deterministic_verdict: &final_verdict,
        rule_id: rule_id.as_deref(),
        reversibility: reversibility.as_ref(),
        contract: task_contract.as_ref(),
        contract_violation_present: contract_violation.is_some(),
    });

    apply_broker_outcome(
        &broker_outcome,
        &mut final_verdict,
        &mut rule_id,
        &mut rule_name,
        &mut correction,
        &mut reversibility,
    );

    let latency_us = start.elapsed().as_micros() as u64;

    // Record in audit trail. Persistence failure is fail-closed.
    let audit_result = {
        let mut audit = state.audit_trail.lock().unwrap();
        audit.record_action_with_broker_decision(
            &request.session_id,
            request.sequence as u64,
            &request.action.action_type.to_string(),
            &request.action.tool,
            &serde_json::to_string(&request.action.payload).unwrap_or_default(),
            &final_verdict,
            rule_id.as_deref(),
            correction.as_deref(),
            latency_us,
            Some(broker_outcome.decision.as_str()),
            Some(broker_outcome.guardian_mode.as_str()),
            &broker_outcome.obligations,
            Some(&broker_outcome.reason),
        )
    };

    let (action_id, canonical_payload_hash) = match audit_result {
        Ok((id, payload_hash)) => (Some(id), Some(payload_hash)),
        Err(e) => {
            log::error!("Audit record failed: {}", e);
            return ActionResponse {
                version: 1,
                session_id: request.session_id,
                sequence: request.sequence,
                decision: Verdict::Block,
                action_id: None,
                canonical_payload_hash: None,
                rule_id: Some("ONUS_AUDIT_FAILURE".into()),
                rule_name: Some("audit-record-failed".into()),
                correction: Some(
                    "Onus blocked this action because it could not persist an audit record.".into(),
                ),
                latency_us,
                reversibility: None,
                approval_decision: Some(ApprovalDecision::DenyWithCorrection),
                guardian_mode: Some(approval_broker::GuardianMode::from_env()),
                obligations: Vec::new(),
                approval_reason: Some(
                    "Audit persistence failed; Onus fails closed for critical evaluator persistence."
                        .into(),
                ),
            };
        }
    };

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
    if broker_outcome.decision == ApprovalDecision::TerminateSession {
        if let Ok(mut audit) = state.audit_trail.lock() {
            let _ = audit.update_session_status(&request.session_id, "terminated");
        }
    }

    if final_verdict == Verdict::Block || final_verdict == Verdict::Escalate {
        if let Ok(mut audit) = state.audit_trail.lock() {
            let workspace_root = audit
                .get_session(&request.session_id)
                .ok()
                .flatten()
                .map(|session| session.workspace_root)
                .unwrap_or_else(|| ".".to_string());
            let incident_key = format!(
                "{}:{}",
                request.action.action_type,
                rule_id.as_deref().unwrap_or("unknown")
            );
            let _ = audit.remember_incident(
                &request.session_id,
                &workspace_root,
                &incident_key,
                serde_json::json!({
                    "action_type": request.action.action_type.to_string(),
                    "tool": request.action.tool.clone(),
                    "verdict": final_verdict.to_string(),
                    "rule_id": rule_id.clone(),
                    "correction": correction.clone(),
                }),
            );
        }
    }

    ActionResponse {
        version: 1,
        session_id: request.session_id,
        sequence: request.sequence,
        decision: final_verdict,
        action_id,
        canonical_payload_hash,
        rule_id,
        rule_name,
        correction,
        latency_us,
        reversibility,
        approval_decision: Some(broker_outcome.decision),
        guardian_mode: Some(broker_outcome.guardian_mode),
        obligations: broker_outcome.obligations,
        approval_reason: Some(broker_outcome.reason),
    }
}

fn apply_broker_outcome(
    broker_outcome: &approval_broker::BrokerOutcome,
    final_verdict: &mut Verdict,
    rule_id: &mut Option<String>,
    rule_name: &mut Option<String>,
    correction: &mut Option<String>,
    reversibility: &mut Option<crate::Reversibility>,
) {
    match broker_outcome.decision {
        ApprovalDecision::AllowAutomatically => {}
        ApprovalDecision::AllowWithObligations => {
            if matches!(final_verdict, Verdict::Allow | Verdict::Warn)
                && correction.is_none()
                && !broker_outcome.obligations.is_empty()
            {
                *correction = Some(format!(
                    "ALLOW_WITH_OBLIGATIONS: {}",
                    broker_outcome.obligations.join("; ")
                ));
            }
        }
        ApprovalDecision::RequireHumanApproval => {
            if matches!(final_verdict, Verdict::Allow | Verdict::Warn) {
                *final_verdict = Verdict::Escalate;
                *rule_id = Some("ONUS_APPROVAL_BROKER_HUMAN_REQUIRED".to_string());
                *rule_name = Some("approval-decision-broker".to_string());
                *correction = Some(format!(
                    "{} Obligations: {}",
                    broker_outcome.reason,
                    broker_outcome.obligations.join("; ")
                ));
                *reversibility = None;
            }
        }
        ApprovalDecision::DenyWithCorrection => {
            if !matches!(final_verdict, Verdict::Block) {
                *final_verdict = Verdict::Block;
                *rule_id = Some("ONUS_APPROVAL_BROKER_DENIED".to_string());
                *rule_name = Some("approval-decision-broker".to_string());
                *correction = Some(format!(
                    "{} Obligations: {}",
                    broker_outcome.reason,
                    broker_outcome.obligations.join("; ")
                ));
                *reversibility = None;
            }
        }
        ApprovalDecision::TerminateSession => {
            *final_verdict = Verdict::Block;
            *rule_id = Some("ONUS_APPROVAL_BROKER_TERMINATED".to_string());
            *rule_name = Some("approval-decision-broker".to_string());
            *correction = Some(format!(
                "{} Obligations: {}",
                broker_outcome.reason,
                broker_outcome.obligations.join("; ")
            ));
            *reversibility = None;
        }
    }
}

fn apply_semantic_risk_review(
    request: &ActionRequest,
    final_verdict: &mut Verdict,
    rule_id: &mut Option<String>,
    rule_name: &mut Option<String>,
    correction: &mut Option<String>,
    reversibility: &mut Option<crate::Reversibility>,
) {
    if matches!(final_verdict, Verdict::Block) {
        return;
    }
    let config = semantic::SemanticReviewerConfig::from_env();
    apply_semantic_risk_review_with_config(
        request,
        final_verdict,
        rule_id,
        rule_name,
        correction,
        reversibility,
        config,
    );
}

fn apply_semantic_risk_review_with_config(
    request: &ActionRequest,
    final_verdict: &mut Verdict,
    rule_id: &mut Option<String>,
    rule_name: &mut Option<String>,
    correction: &mut Option<String>,
    reversibility: &mut Option<crate::Reversibility>,
    mut config: semantic::SemanticReviewerConfig,
) {
    if !config.provider_enabled() {
        return;
    }
    if semantic::is_critical_action(request) {
        config.fallback_policy = SemanticFallbackPolicy::FailClosed;
    }
    let reviewer = ConfiguredSemanticReviewer::new(config);
    let critical = semantic::is_critical_action(request);
    let findings = rule_id
        .iter()
        .chain(rule_name.iter())
        .chain(correction.iter())
        .cloned()
        .collect::<Vec<_>>();

    match reviewer.review_action(
        ActionReviewRequest {
            task_contract_hash: None,
            action: request.action.clone(),
            relevant_diff: None,
            previous_actions: Vec::new(),
            repository_architecture: Vec::new(),
            deterministic_verdict: final_verdict.clone(),
            policy_findings: findings,
        },
        critical,
    ) {
        Ok(call) => {
            if !call.trace.accepted {
                return;
            }
            let semantic_verdict = call.output.recommended_decision.clone();
            let tightened = most_restrictive(final_verdict.clone(), semantic_verdict.clone());
            if tightened != *final_verdict {
                *final_verdict = tightened;
                *rule_id = Some("ONUS_SEMANTIC_RISK_CRITIC".to_string());
                *rule_name = Some("semantic-risk-critic".to_string());
                *correction = Some(format!(
                    "Semantic Risk Critic recommended {}: {}",
                    semantic_verdict, call.output.rationale
                ));
                *reversibility = None;
            }
        }
        Err(trace) => {
            *final_verdict = Verdict::Block;
            *rule_id = Some("ONUS_SEMANTIC_REVIEW_FAILED".to_string());
            *rule_name = Some("semantic-review-failed-closed".to_string());
            *correction = Some(format!(
                "Onus blocked this critical action because semantic review failed closed: {}",
                trace.error.as_deref().unwrap_or("unknown error")
            ));
            *reversibility = None;
        }
    }
}

fn evaluate_contract(
    state: &ServerState,
    request: &ActionRequest,
) -> (
    Option<crate::task_contract::TaskContract>,
    Option<ContractViolation>,
) {
    let Ok(audit) = state.audit_trail.lock() else {
        return (
            None,
            Some(ContractViolation {
                verdict: Verdict::Block,
                rule_id: "ONUS_CONTRACT_LOOKUP_FAILED".to_string(),
                rule_name: "task-contract-lock-failed".to_string(),
                correction: "Onus could not lock the audit store to verify the task contract; action blocked.".to_string(),
            }),
        );
    };
    let contract = match audit.get_task_contract(&request.session_id) {
        Ok(value) => value,
        Err(e) => {
            log::error!("Task contract lookup failed: {}", e);
            return (
                None,
                Some(ContractViolation {
                    verdict: Verdict::Block,
                    rule_id: "ONUS_CONTRACT_LOOKUP_FAILED".to_string(),
                    rule_name: "task-contract-lookup-failed".to_string(),
                    correction:
                        "Onus could not verify the persisted task contract; action blocked."
                            .to_string(),
                }),
            );
        }
    };

    let Some(contract) = contract else {
        return (
            None,
            task_contract::evaluate_missing_contract(&request.action),
        );
    };

    let workspace = audit
        .get_session(&request.session_id)
        .ok()
        .flatten()
        .map(|s| std::path::PathBuf::from(s.workspace_root))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let touched = audit
        .session_touched_paths(&request.session_id)
        .unwrap_or_default();
    let violation =
        task_contract::evaluate_action(&contract, &request.action, &workspace, &touched);
    (Some(contract), violation)
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
            task_contract,
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
                if let Some(contract) = task_contract.as_deref() {
                    if let Err(e) = audit.save_task_contract(contract) {
                        return crate::ipc::SessionResponse {
                            status: "error".into(),
                            error: Some(format!("Failed to save task contract: {}", e)),
                        };
                    }
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
pub fn handle_client(state: &ServerState, mut stream: impl Read + Write) -> std::io::Result<()> {
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
                let summaries: Vec<RuleSummary> = rules.iter().map(RuleSummary::from).collect();
                Some(crate::ipc::RulesResponse { rules: summaries })
            } else {
                None
            }
        }),
    }
}

/// Check if the server has been asked to shut down.
pub fn is_shutting_down(state: &ServerState) -> bool {
    state
        .shutting_down
        .load(std::sync::atomic::Ordering::Relaxed)
}

/// Mark the server for graceful shutdown.
pub fn request_shutdown(state: &ServerState) {
    state
        .shutting_down
        .store(true, std::sync::atomic::Ordering::Relaxed);
}

fn most_restrictive(a: Verdict, b: Verdict) -> Verdict {
    use Verdict::*;
    match (&a, &b) {
        (Block, _) | (_, Block) => Block,
        (Escalate, _) | (_, Escalate) => Escalate,
        (Warn, _) | (_, Warn) => Warn,
        _ => Allow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::Action;
    use crate::policy::rule::load_rules_from_str;
    use crate::ActionType;
    use std::path::PathBuf;

    fn temp_db(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("onus-{}-{}.db", name, uuid::Uuid::new_v4()))
    }

    fn allow_state(db_path: &std::path::Path) -> ServerState {
        let rules = load_rules_from_str(
            r#"
            [[rule]]
            id = "TEST_NO_MATCH"
            name = "no match"
            tier = 1
            action_type = "shell"
            pattern = "NEVER_MATCH_THIS_PATTERN"
            decision = "block"
            correction = "no match"
            reversibility = "irreversible"
            "#,
        )
        .unwrap();
        ServerState::new(PolicyEngine::new(rules), AuditTrail::open(db_path).unwrap())
    }

    #[test]
    fn deterministic_block_dominates_approval_escalation() {
        assert_eq!(
            most_restrictive(Verdict::Block, Verdict::Escalate),
            Verdict::Block
        );
        assert_eq!(
            most_restrictive(Verdict::Escalate, Verdict::Block),
            Verdict::Block
        );
    }

    #[test]
    fn critical_semantic_provider_failure_fails_closed() {
        let request = ActionRequest {
            version: 1,
            session_id: "semantic-fail-closed".to_string(),
            sequence: 1,
            action: Action {
                action_type: ActionType::Shell,
                tool: "Bash".to_string(),
                payload: serde_json::json!({"command": "sudo true"}),
            },
        };
        let mut verdict = Verdict::Allow;
        let mut rule_id = None;
        let mut rule_name = None;
        let mut correction = None;
        let mut reversibility = None;

        apply_semantic_risk_review_with_config(
            &request,
            &mut verdict,
            &mut rule_id,
            &mut rule_name,
            &mut correction,
            &mut reversibility,
            semantic::SemanticReviewerConfig {
                provider: semantic::SemanticProviderKind::Local,
                local_command: Some("definitely-not-a-command".to_string()),
                fail_closed_on_critical: true,
                ..semantic::SemanticReviewerConfig::default()
            },
        );

        assert_eq!(verdict, Verdict::Block);
        assert_eq!(rule_id.as_deref(), Some("ONUS_SEMANTIC_REVIEW_FAILED"));
    }

    #[test]
    fn audit_persistence_failure_blocks_without_strict_mode() {
        let db_path = temp_db("strict-mode");
        let state = allow_state(&db_path);
        {
            let mut audit = state.audit_trail.lock().unwrap();
            audit.break_actions_table_for_test().unwrap();
        }

        std::env::remove_var("ONUS_STRICT");
        let response = handle_action(
            &state,
            ActionRequest {
                version: 1,
                session_id: "strict-session".to_string(),
                sequence: 1,
                action: Action {
                    action_type: ActionType::FileWrite,
                    tool: "Write".to_string(),
                    payload: serde_json::json!({"path": "demo.txt", "content": "ok"}),
                },
            },
        );

        assert_eq!(response.decision, Verdict::Block);
        assert_eq!(response.rule_id.as_deref(), Some("ONUS_AUDIT_FAILURE"));
        assert!(response.action_id.is_none());

        let _ = std::fs::remove_file(db_path);
    }
}
