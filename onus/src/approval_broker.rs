use crate::ipc::{Action, ActionRequest};
use crate::security;
use crate::task_contract::{self, TaskContract};
use crate::{ActionType, Reversibility, Verdict};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuardianMode {
    Beginner,
    Professional,
    EnterpriseStrict,
}

impl GuardianMode {
    pub fn from_env() -> Self {
        std::env::var("ONUS_GUARDIAN_MODE")
            .ok()
            .as_deref()
            .and_then(Self::parse)
            .unwrap_or_else(|| {
                if security::strict_mode_enabled() {
                    Self::EnterpriseStrict
                } else {
                    Self::Professional
                }
            })
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().replace('-', "_").as_str() {
            "beginner" | "beginner_guardian" | "beginner_guardian_mode" => Some(Self::Beginner),
            "professional" | "professional_reviewer" | "professional_reviewer_mode" => {
                Some(Self::Professional)
            }
            "enterprise" | "strict" | "enterprise_strict" | "enterprise_strict_mode" => {
                Some(Self::EnterpriseStrict)
            }
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Beginner => "beginner",
            Self::Professional => "professional",
            Self::EnterpriseStrict => "enterprise_strict",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApprovalDecision {
    AllowAutomatically,
    AllowWithObligations,
    RequireHumanApproval,
    DenyWithCorrection,
    TerminateSession,
}

impl ApprovalDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AllowAutomatically => "ALLOW_AUTOMATICALLY",
            Self::AllowWithObligations => "ALLOW_WITH_OBLIGATIONS",
            Self::RequireHumanApproval => "REQUIRE_HUMAN_APPROVAL",
            Self::DenyWithCorrection => "DENY_WITH_CORRECTION",
            Self::TerminateSession => "TERMINATE_SESSION",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrokerOutcome {
    pub decision: ApprovalDecision,
    pub guardian_mode: GuardianMode,
    pub reason: String,
    #[serde(default)]
    pub obligations: Vec<String>,
    pub risk_summary: RiskSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskSummary {
    pub inside_task_contract: bool,
    pub low_risk: bool,
    pub protected_resource_touched: bool,
    pub production_environment: bool,
    pub blast_radius_within_limits: bool,
    pub reversible_or_read_only: bool,
    pub deterministic_rule_triggered: bool,
    pub human_approval_required: bool,
    pub schema_change: bool,
    pub credential_use: bool,
    pub infrastructure_change: bool,
    pub external_communication: bool,
    pub irreversible: bool,
    pub test_weakening: bool,
    pub low_confidence: bool,
}

pub struct BrokerInput<'a> {
    pub request: &'a ActionRequest,
    pub deterministic_verdict: &'a Verdict,
    pub rule_id: Option<&'a str>,
    pub reversibility: Option<&'a Reversibility>,
    pub contract: Option<&'a TaskContract>,
    pub contract_violation_present: bool,
}

pub fn decide(input: BrokerInput<'_>) -> BrokerOutcome {
    let guardian_mode = GuardianMode::from_env();
    decide_with_mode(input, guardian_mode)
}

pub fn decide_with_mode(input: BrokerInput<'_>, guardian_mode: GuardianMode) -> BrokerOutcome {
    let risk = summarize_risk(&input);

    if matches!(input.deterministic_verdict, Verdict::Block) {
        return BrokerOutcome {
            decision: ApprovalDecision::DenyWithCorrection,
            guardian_mode,
            reason: "Deterministic policy denied this action; broker cannot override denial."
                .to_string(),
            obligations: Vec::new(),
            risk_summary: risk,
        };
    }

    if critical_termination_required(&guardian_mode, &risk) {
        return BrokerOutcome {
            decision: ApprovalDecision::TerminateSession,
            guardian_mode,
            reason: "Enterprise Strict Mode terminates sessions on critical production credential or infrastructure risk without a verified boundary.".to_string(),
            obligations: vec!["Restart in an Onus-controlled L3/L4 environment before retrying.".to_string()],
            risk_summary: risk,
        };
    }

    if requires_human(input.deterministic_verdict, &risk) {
        return BrokerOutcome {
            decision: ApprovalDecision::RequireHumanApproval,
            guardian_mode,
            reason: human_reason(&risk),
            obligations: human_obligations(&risk),
            risk_summary: risk,
        };
    }

    if test_weakening_should_be_denied(&guardian_mode, &risk) {
        return BrokerOutcome {
            decision: ApprovalDecision::DenyWithCorrection,
            guardian_mode,
            reason: "The action appears to delete, skip, or weaken tests.".to_string(),
            obligations: vec![
                "Restore or preserve the affected test.".to_string(),
                "Repair implementation without reducing test coverage or assertions.".to_string(),
            ],
            risk_summary: risk,
        };
    }

    if risk.low_risk
        && risk.inside_task_contract
        && !risk.protected_resource_touched
        && !risk.production_environment
        && risk.blast_radius_within_limits
        && risk.reversible_or_read_only
        && !risk.deterministic_rule_triggered
        && !risk.human_approval_required
    {
        return BrokerOutcome {
            decision: ApprovalDecision::AllowAutomatically,
            guardian_mode,
            reason: "All automatic-approval conditions are satisfied.".to_string(),
            obligations: Vec::new(),
            risk_summary: risk,
        };
    }

    BrokerOutcome {
        decision: ApprovalDecision::AllowWithObligations,
        guardian_mode,
        reason:
            "The action can proceed, but completion must satisfy quality and evidence obligations."
                .to_string(),
        obligations: quality_obligations(&input),
        risk_summary: risk,
    }
}

fn summarize_risk(input: &BrokerInput<'_>) -> RiskSummary {
    let action = &input.request.action;
    let payload_text = security::canonical_json(&action.payload).to_ascii_lowercase();
    let classified = security::classify_payload(&action.payload);
    let contains_sensitive = classified
        .classification
        .contains("\"contains_sensitive\":true");
    let contract = input.contract;
    let production_environment = is_production_environment(contract, &payload_text);
    let schema_change = is_schema_change(action, &payload_text);
    let credential_use = contains_sensitive || mentions_credentials(action, &payload_text);
    let infrastructure_change = is_infrastructure_change(action, &payload_text);
    let external_communication = is_external_communication(action, &payload_text);
    let test_weakening = is_test_weakening(action, &payload_text);
    let irreversible = is_irreversible(action, input.reversibility, &payload_text);
    let protected_resource_touched = input.contract_violation_present
        || touches_protected_resource(contract, action, &payload_text)
        || credential_use;
    let deterministic_rule_triggered =
        !matches!(input.deterministic_verdict, Verdict::Allow) || input.rule_id.is_some();
    let low_confidence = semantic_confidence(action)
        .map(|confidence| confidence < 0.70)
        .unwrap_or(false);
    let human_approval_required = matches!(input.deterministic_verdict, Verdict::Escalate)
        || production_environment
        || schema_change
        || credential_use
        || infrastructure_change
        || external_communication
        || irreversible
        || test_weakening
        || low_confidence;

    RiskSummary {
        inside_task_contract: contract.is_some() && !input.contract_violation_present,
        low_risk: is_low_risk(action, &payload_text)
            && !schema_change
            && !credential_use
            && !infrastructure_change
            && !external_communication
            && !irreversible
            && !test_weakening
            && !low_confidence,
        protected_resource_touched,
        production_environment,
        blast_radius_within_limits: blast_radius_within_limits(contract, action, &payload_text),
        reversible_or_read_only: is_read_only(action) || is_reversible(action, input.reversibility),
        deterministic_rule_triggered,
        human_approval_required,
        schema_change,
        credential_use,
        infrastructure_change,
        external_communication,
        irreversible,
        test_weakening,
        low_confidence,
    }
}

fn requires_human(verdict: &Verdict, risk: &RiskSummary) -> bool {
    matches!(verdict, Verdict::Escalate)
        || risk.production_environment
        || risk.schema_change
        || risk.credential_use
        || risk.infrastructure_change
        || risk.external_communication
        || risk.irreversible
        || risk.test_weakening
        || risk.low_confidence
}

fn critical_termination_required(mode: &GuardianMode, risk: &RiskSummary) -> bool {
    matches!(mode, GuardianMode::EnterpriseStrict)
        && risk.production_environment
        && (risk.credential_use || risk.infrastructure_change)
}

fn test_weakening_should_be_denied(mode: &GuardianMode, risk: &RiskSummary) -> bool {
    matches!(mode, GuardianMode::Beginner | GuardianMode::Professional) && risk.test_weakening
}

fn human_reason(risk: &RiskSummary) -> String {
    let mut reasons = Vec::new();
    if risk.production_environment {
        reasons.push("production environment");
    }
    if risk.schema_change {
        reasons.push("database schema change");
    }
    if risk.credential_use {
        reasons.push("credential or secret use");
    }
    if risk.infrastructure_change {
        reasons.push("infrastructure change");
    }
    if risk.external_communication {
        reasons.push("external communication");
    }
    if risk.irreversible {
        reasons.push("irreversible action");
    }
    if risk.test_weakening {
        reasons.push("test deletion or weakening");
    }
    if risk.low_confidence {
        reasons.push("low-confidence semantic assessment");
    }
    if reasons.is_empty() {
        "Human approval is required by policy.".to_string()
    } else {
        format!("Human approval is required for {}.", reasons.join(", "))
    }
}

fn human_obligations(risk: &RiskSummary) -> Vec<String> {
    let mut obligations = Vec::new();
    if risk.production_environment {
        obligations.push("Show verified environment identity before approval.".to_string());
    }
    if risk.schema_change {
        obligations.push("Provide migration diff and rollback plan.".to_string());
    }
    if risk.credential_use {
        obligations.push("Use credential references only; do not expose raw secrets.".to_string());
    }
    if risk.infrastructure_change {
        obligations.push("Require infrastructure review before execution.".to_string());
    }
    if risk.external_communication {
        obligations.push("Confirm destination, payload, and data exposure.".to_string());
    }
    if risk.irreversible {
        obligations.push("Explain what cannot be reverted.".to_string());
    }
    if risk.test_weakening {
        obligations.push("Restore or preserve tests; do not reduce assertions.".to_string());
    }
    if risk.low_confidence {
        obligations.push("Resolve low-confidence assessment before proceeding.".to_string());
    }
    obligations
}

fn quality_obligations(input: &BrokerInput<'_>) -> Vec<String> {
    let mut obligations = Vec::new();
    if matches!(input.request.action.action_type, ActionType::FileWrite) {
        obligations
            .push("Run targeted tests for the touched module before completion.".to_string());
        obligations.push("Verify no secrets were introduced.".to_string());
    }
    if matches!(input.request.action.action_type, ActionType::Shell) {
        obligations.push("Record command output as completion evidence when relevant.".to_string());
    }
    if obligations.is_empty() {
        obligations
            .push("Record evidence required by the task contract before completion.".to_string());
    }
    obligations
}

fn is_low_risk(action: &Action, payload_text: &str) -> bool {
    match action.action_type {
        ActionType::FileRead => true,
        ActionType::FileWrite => payload_text.len() <= 12_000 && !path_looks_sensitive(action),
        ActionType::Shell => is_safe_local_command(payload_text),
        _ => false,
    }
}

fn is_safe_local_command(payload_text: &str) -> bool {
    let trimmed = payload_text.trim();
    [
        "pytest",
        "python -m pytest",
        "cargo test",
        "cargo fmt",
        "cargo clippy",
        "npm test",
        "npm run test",
        "ruff",
        "mypy",
    ]
    .iter()
    .any(|prefix| trimmed.contains(prefix))
}

fn is_read_only(action: &Action) -> bool {
    matches!(action.action_type, ActionType::FileRead)
}

fn is_reversible(action: &Action, reversibility: Option<&Reversibility>) -> bool {
    if matches!(reversibility, Some(Reversibility::Irreversible)) {
        return false;
    }
    matches!(
        action.action_type,
        ActionType::FileRead | ActionType::FileWrite | ActionType::Shell
    )
}

fn is_irreversible(
    action: &Action,
    reversibility: Option<&Reversibility>,
    payload_text: &str,
) -> bool {
    matches!(reversibility, Some(Reversibility::Irreversible))
        || matches!(action.action_type, ActionType::FileDelete)
        || payload_text.contains("rm -rf")
        || payload_text.contains("drop table")
        || payload_text.contains("delete from")
}

fn is_schema_change(action: &Action, payload_text: &str) -> bool {
    matches!(action.action_type, ActionType::DbMutation)
        && [
            "create table",
            "alter table",
            "drop table",
            "create index",
            "drop index",
            "migration",
            "schema",
        ]
        .iter()
        .any(|needle| payload_text.contains(needle))
}

fn is_production_environment(contract: Option<&TaskContract>, payload_text: &str) -> bool {
    contract
        .map(|contract| contract.environment_identity.to_ascii_lowercase())
        .filter(|env| env.contains("prod") || env.contains("production"))
        .is_some()
        || payload_text.contains("production")
        || payload_text.contains("\"env\":\"prod\"")
        || payload_text.contains("\"environment\":\"prod\"")
        || std::env::var("ONUS_ENVIRONMENT_IDENTITY")
            .map(|env| {
                let env = env.to_ascii_lowercase();
                env.contains("prod") || env.contains("production")
            })
            .unwrap_or(false)
}

fn mentions_credentials(action: &Action, payload_text: &str) -> bool {
    matches!(
        action.action_type,
        ActionType::ApiCall | ActionType::Network
    ) && [
        "authorization",
        "bearer ",
        "api_key",
        "apikey",
        "token",
        "secret",
    ]
    .iter()
    .any(|needle| payload_text.contains(needle))
}

fn is_infrastructure_change(action: &Action, payload_text: &str) -> bool {
    matches!(
        action.action_type,
        ActionType::Shell | ActionType::FileWrite | ActionType::MCP
    ) && [
        "terraform apply",
        "terraform destroy",
        "kubectl apply",
        "kubectl delete",
        "helm upgrade",
        "cloudformation",
        "pulumi up",
        ".github/workflows",
        "docker-compose",
    ]
    .iter()
    .any(|needle| payload_text.contains(needle))
}

fn is_external_communication(action: &Action, payload_text: &str) -> bool {
    matches!(
        action.action_type,
        ActionType::ApiCall | ActionType::Network
    ) || [
        "curl http://",
        "curl https://",
        "wget http://",
        "wget https://",
        "ssh ",
        "scp ",
        "smtp",
        "webhook",
    ]
    .iter()
    .any(|needle| payload_text.contains(needle))
}

fn is_test_weakening(action: &Action, payload_text: &str) -> bool {
    let path = task_contract::extract_action_path(action)
        .unwrap_or_default()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let touches_tests = path.contains("/test")
        || path.starts_with("test")
        || path.contains("_test.")
        || path.contains(".spec.")
        || path.contains(".test.");
    let weakening_text = [
        "skip(",
        "xfail",
        ".only(",
        "todo(",
        "assert true",
        "delete test",
        "remove test",
        "disable test",
    ]
    .iter()
    .any(|needle| payload_text.contains(needle));
    matches!(action.action_type, ActionType::FileDelete) && touches_tests
        || (matches!(action.action_type, ActionType::FileWrite) && touches_tests && weakening_text)
        || payload_text.contains("rm ") && payload_text.contains("test")
}

fn touches_protected_resource(
    contract: Option<&TaskContract>,
    action: &Action,
    payload_text: &str,
) -> bool {
    path_looks_sensitive(action)
        || contract
            .map(|contract| {
                contract.protected_resources.iter().any(|resource| {
                    let resource = resource.to_ascii_lowercase();
                    !resource.is_empty() && payload_text.contains(&resource)
                })
            })
            .unwrap_or(false)
}

fn path_looks_sensitive(action: &Action) -> bool {
    task_contract::extract_action_path(action)
        .map(|path| {
            let path = path.replace('\\', "/").to_ascii_lowercase();
            path.ends_with(".env")
                || path.contains("/.env")
                || path.contains("secrets")
                || path.contains("credentials")
        })
        .unwrap_or(false)
}

fn blast_radius_within_limits(
    contract: Option<&TaskContract>,
    action: &Action,
    payload_text: &str,
) -> bool {
    if payload_text.len() > 50_000 {
        return false;
    }
    if matches!(action.action_type, ActionType::FileDelete) {
        return false;
    }
    contract
        .map(|contract| contract.change_budget.max_files_changed > 0)
        .unwrap_or(false)
}

fn semantic_confidence(action: &Action) -> Option<f64> {
    action
        .payload
        .get("semantic_confidence")
        .and_then(|v| v.as_f64())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::ActionRequest;
    use crate::task_contract::{ChangeBudget, TaskContract};

    fn contract(env: &str) -> TaskContract {
        TaskContract {
            schema_version: 1,
            session_id: "session-1".to_string(),
            original_prompt: "Update allowed file".to_string(),
            normalized_objective: "Update allowed file".to_string(),
            allowed_paths: vec!["src/**".to_string()],
            allowed_resources: vec![],
            protected_paths: vec![".env".to_string()],
            protected_resources: vec!["production-db".to_string()],
            required_evidence: vec![],
            forbidden_actions: vec![],
            approval_required_actions: vec![],
            change_budget: ChangeBudget {
                max_files_changed: 5,
                max_actions: 20,
            },
            environment_identity: env.to_string(),
            policy_version: "test-policy".to_string(),
            canonical_hash: String::new(),
        }
        .finalized()
    }

    fn request(action_type: ActionType, payload: serde_json::Value) -> ActionRequest {
        ActionRequest {
            version: 1,
            session_id: "session-1".to_string(),
            sequence: 1,
            action: Action {
                action_type,
                tool: "test-tool".to_string(),
                payload,
            },
        }
    }

    #[test]
    fn low_risk_read_can_be_automatic() {
        let contract = contract("local-dev");
        let request = request(
            ActionType::FileRead,
            serde_json::json!({"path": "src/lib.rs"}),
        );
        let outcome = decide_with_mode(
            BrokerInput {
                request: &request,
                deterministic_verdict: &Verdict::Allow,
                rule_id: None,
                reversibility: None,
                contract: Some(&contract),
                contract_violation_present: false,
            },
            GuardianMode::Professional,
        );
        assert_eq!(outcome.decision, ApprovalDecision::AllowAutomatically);
    }

    #[test]
    fn production_schema_change_requires_human() {
        let contract = contract("production-us-east-1");
        let request = request(
            ActionType::DbMutation,
            serde_json::json!({"sql": "ALTER TABLE users ADD COLUMN name TEXT"}),
        );
        let outcome = decide_with_mode(
            BrokerInput {
                request: &request,
                deterministic_verdict: &Verdict::Allow,
                rule_id: None,
                reversibility: None,
                contract: Some(&contract),
                contract_violation_present: false,
            },
            GuardianMode::Professional,
        );
        assert_eq!(outcome.decision, ApprovalDecision::RequireHumanApproval);
        assert!(outcome.risk_summary.production_environment);
        assert!(outcome.risk_summary.schema_change);
    }

    #[test]
    fn enterprise_strict_terminates_production_credential_infrastructure() {
        let contract = contract("production");
        let request = request(
            ActionType::Shell,
            serde_json::json!({"command": "terraform apply", "token": "secret-value"}),
        );
        let outcome = decide_with_mode(
            BrokerInput {
                request: &request,
                deterministic_verdict: &Verdict::Allow,
                rule_id: None,
                reversibility: None,
                contract: Some(&contract),
                contract_violation_present: false,
            },
            GuardianMode::EnterpriseStrict,
        );
        assert_eq!(outcome.decision, ApprovalDecision::TerminateSession);
    }
}
