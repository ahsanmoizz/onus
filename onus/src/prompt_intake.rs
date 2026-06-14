use crate::security;
use crate::semantic::{
    evidence_from_semantic, semantic_review_summary, ConfiguredSemanticReviewer,
    SemanticProviderKind, SemanticReviewer, SemanticReviewerConfig, SemanticRoleTrace,
    TaskInterpretation, TaskInterpretationRequest,
};
use crate::task_contract::{ChangeBudget, RequiredEvidence, TaskContract};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntakeStatus {
    Ready,
    ReadyWithSafeContract,
    ClarificationRequired,
    RejectedAsUnsafe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderMode {
    Deterministic,
    Disabled,
    Local,
    Cloud,
}

impl From<ProviderMode> for SemanticProviderKind {
    fn from(value: ProviderMode) -> Self {
        match value {
            ProviderMode::Deterministic => SemanticProviderKind::Deterministic,
            ProviderMode::Disabled => SemanticProviderKind::Disabled,
            ProviderMode::Local => SemanticProviderKind::Local,
            ProviderMode::Cloud => SemanticProviderKind::Cloud,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptIntakeRequest {
    pub original_prompt: String,
    pub session_id: String,
    pub workspace_root: String,
    #[serde(default)]
    pub memory_context: Vec<String>,
    #[serde(default)]
    pub provider_mode: Option<ProviderMode>,
    #[serde(default)]
    pub semantic_config: Option<SemanticReviewerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptIntakeResult {
    pub status: IntakeStatus,
    pub provider_mode: ProviderMode,
    pub semantic_review: String,
    #[serde(default)]
    pub semantic_roles: Vec<SemanticRoleTrace>,
    pub reasons: Vec<String>,
    pub questions: Vec<String>,
    pub proposed_contract: Option<TaskContract>,
}

pub fn analyze_prompt(request: PromptIntakeRequest) -> PromptIntakeResult {
    let prompt = request.original_prompt.trim();
    let lower = prompt.to_ascii_lowercase();
    let provider_mode = request
        .provider_mode
        .clone()
        .unwrap_or(ProviderMode::Deterministic);
    let findings = Findings {
        dangerously_broad: contains_any(
            &lower,
            &[
                "fix everything",
                "make everything work",
                "make all tests pass",
                "whole website",
                "entire app",
                "full access",
                "whatever works",
            ],
        ),
        destructive_wording: contains_any(
            &lower,
            &[
                "delete anything",
                "remove anything",
                "wipe",
                "nuke",
                "rm -rf",
                "drop table",
                "delete all",
            ],
        ),
        missing_environment: !contains_any(
            &lower,
            &[
                "local",
                "development",
                "dev",
                "staging",
                "test environment",
                "production",
                "prod",
            ],
        ),
        missing_scope: !mentions_scope(prompt),
        delete_tests: contains_any(
            &lower,
            &[
                "delete test",
                "delete tests",
                "remove test",
                "remove tests",
                "skip test",
                "disable test",
            ],
        ),
        expose_secrets: contains_any(
            &lower,
            &[
                "api key directly",
                "hardcode api key",
                "hard-code api key",
                "put the api key",
                "expose secret",
                "commit secret",
                "print token",
                "show password",
            ],
        ),
        disable_security: contains_any(
            &lower,
            &[
                "disable security",
                "turn off security",
                "bypass security",
                "remove auth",
                "disable auth",
                "ignore policy",
            ],
        ),
        direct_production: contains_any(
            &lower,
            &[
                "production database",
                "prod database",
                "directly on production",
                "against production",
                "deploy whatever",
                "production credentials",
            ],
        ),
        missing_completion: !contains_any(
            &lower,
            &[
                "test",
                "tests",
                "verify",
                "evidence",
                "done when",
                "complete when",
                "acceptance",
                "lint",
                "typecheck",
            ],
        ),
    };

    let mut reasons = findings.reasons();
    let mut questions = findings.questions();

    let status = if findings.disable_security || findings.expose_secrets {
        IntakeStatus::RejectedAsUnsafe
    } else if findings.direct_production {
        IntakeStatus::ClarificationRequired
    } else if findings.dangerously_broad
        || findings.destructive_wording
        || findings.delete_tests
        || findings.missing_environment
        || findings.missing_scope
        || findings.missing_completion
    {
        IntakeStatus::ReadyWithSafeContract
    } else {
        IntakeStatus::Ready
    };

    if matches!(status, IntakeStatus::Ready) {
        reasons
            .push("prompt appears bounded enough for deterministic contract creation".to_string());
    }

    let mut proposed_contract = if matches!(
        status,
        IntakeStatus::Ready | IntakeStatus::ReadyWithSafeContract
    ) {
        Some(propose_contract(&request, &findings))
    } else {
        None
    };

    let mut semantic_config = request
        .semantic_config
        .clone()
        .unwrap_or_else(SemanticReviewerConfig::from_env);
    semantic_config.provider = provider_mode.clone().into();
    let reviewer = ConfiguredSemanticReviewer::new(semantic_config);
    let semantic_call = reviewer.interpret_task(TaskInterpretationRequest {
        original_prompt: request.original_prompt.clone(),
        repository_metadata: vec![format!("workspace_root={}", request.workspace_root)],
        memory_context: request.memory_context.clone(),
        existing_policy: reasons.clone(),
        current_environment: security::environment_identity(),
    });
    let mut semantic_roles = Vec::new();
    let semantic_review = match semantic_call {
        Ok(call) => {
            semantic_roles.push(call.trace.clone());
            if call.trace.accepted && call.trace.provider_invoked {
                apply_semantic_interpretation(
                    &call.output,
                    &mut proposed_contract,
                    &mut reasons,
                    &mut questions,
                );
            }
            semantic_review_summary(&call.trace)
        }
        Err(trace) => {
            semantic_roles.push(trace.clone());
            semantic_review_summary(&trace)
        }
    };

    PromptIntakeResult {
        status,
        provider_mode: provider_mode.clone(),
        semantic_review,
        semantic_roles,
        reasons,
        questions,
        proposed_contract,
    }
}

fn apply_semantic_interpretation(
    interpretation: &TaskInterpretation,
    proposed_contract: &mut Option<TaskContract>,
    reasons: &mut Vec<String>,
    questions: &mut Vec<String>,
) {
    reasons.extend(
        interpretation
            .ambiguities
            .iter()
            .map(|item| format!("semantic_ambiguity:{}", item)),
    );
    reasons.extend(
        interpretation
            .risk_assumptions
            .iter()
            .map(|item| format!("semantic_risk_assumption:{}", item)),
    );
    questions.extend(interpretation.questions.iter().cloned());
    dedup_sorted(questions);

    let Some(contract) = proposed_contract.as_mut() else {
        return;
    };
    if !interpretation.normalized_objective.trim().is_empty() {
        contract.normalized_objective = interpretation.normalized_objective.trim().to_string();
    }
    contract
        .protected_paths
        .extend(interpretation.protected_scope.iter().cloned());
    contract.required_evidence.extend(
        interpretation
            .completion_evidence
            .iter()
            .map(|item| evidence_from_semantic(item)),
    );
    contract.protected_paths.sort();
    contract.protected_paths.dedup();
    contract.required_evidence.sort_by(|a, b| a.id.cmp(&b.id));
    contract.required_evidence.dedup_by(|a, b| a.id == b.id);
    *contract = contract.clone().finalized();
}

#[derive(Default)]
struct Findings {
    dangerously_broad: bool,
    destructive_wording: bool,
    missing_environment: bool,
    missing_scope: bool,
    delete_tests: bool,
    expose_secrets: bool,
    disable_security: bool,
    direct_production: bool,
    missing_completion: bool,
}

impl Findings {
    fn reasons(&self) -> Vec<String> {
        let mut reasons = Vec::new();
        push_if(
            &mut reasons,
            self.dangerously_broad,
            "dangerously_broad_prompt",
        );
        push_if(
            &mut reasons,
            self.destructive_wording,
            "destructive_wording",
        );
        push_if(
            &mut reasons,
            self.missing_environment,
            "missing_environment",
        );
        push_if(&mut reasons, self.missing_scope, "missing_scope");
        push_if(
            &mut reasons,
            self.delete_tests,
            "requests_to_delete_or_disable_tests",
        );
        push_if(
            &mut reasons,
            self.expose_secrets,
            "requests_to_expose_or_hardcode_secrets",
        );
        push_if(
            &mut reasons,
            self.disable_security,
            "requests_to_disable_security",
        );
        push_if(
            &mut reasons,
            self.direct_production,
            "requests_direct_production_operation",
        );
        push_if(
            &mut reasons,
            self.missing_completion,
            "missing_completion_criteria",
        );
        reasons
    }

    fn questions(&self) -> Vec<String> {
        let mut questions = Vec::new();
        if self.missing_environment || self.direct_production {
            questions.push("Which environment is authorized for this task: local, staging, or production with explicit approval?".to_string());
        }
        if self.missing_scope {
            questions
                .push("Which repository paths, services, or resources are in scope?".to_string());
        }
        if self.missing_completion {
            questions.push("What evidence must prove completion, such as tests, lint, typecheck, or manual verification?".to_string());
        }
        questions
    }
}

fn propose_contract(request: &PromptIntakeRequest, findings: &Findings) -> TaskContract {
    let allowed_paths = infer_allowed_paths(&request.original_prompt);
    let required_evidence = infer_required_evidence(&request.original_prompt, findings);
    let mut forbidden_actions = vec![
        "expose_secrets".to_string(),
        "disable_security".to_string(),
        "delete_tests".to_string(),
        "production_operation".to_string(),
    ];
    if findings.destructive_wording || findings.dangerously_broad {
        forbidden_actions.push("file_delete".to_string());
    }

    TaskContract {
        schema_version: 1,
        session_id: request.session_id.clone(),
        original_prompt: request.original_prompt.clone(),
        normalized_objective: normalize_objective(&request.original_prompt, findings),
        allowed_paths,
        allowed_resources: vec!["local_workspace".to_string()],
        protected_paths: vec![
            ".env".to_string(),
            "**/.env".to_string(),
            "docs/**".to_string(),
            ".github/**".to_string(),
            "production/**".to_string(),
            "migrations/**".to_string(),
        ],
        protected_resources: vec![
            "production".to_string(),
            "production-db".to_string(),
            "credentials".to_string(),
        ],
        required_evidence,
        forbidden_actions,
        approval_required_actions: vec![
            "db_migration".to_string(),
            "api_call".to_string(),
            "network".to_string(),
            "mcp".to_string(),
        ],
        change_budget: ChangeBudget {
            max_files_changed: if findings.dangerously_broad { 5 } else { 10 },
            max_actions: 100,
        },
        environment_identity: security::environment_identity(),
        policy_version: security::policy_version(),
        canonical_hash: String::new(),
    }
    .finalized()
}

fn normalize_objective(prompt: &str, findings: &Findings) -> String {
    if findings.dangerously_broad || findings.destructive_wording {
        "Diagnose and repair the smallest safe local change set without deleting tests, weakening security, or touching production resources.".to_string()
    } else {
        prompt.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

fn infer_allowed_paths(prompt: &str) -> Vec<String> {
    let mut paths = prompt
        .split_whitespace()
        .filter_map(|raw| {
            let token = raw.trim_matches(|c: char| c == ',' || c == ';' || c == ':' || c == '.');
            if token.contains('/') || token.contains('\\') {
                Some(token.replace('\\', "/"))
            } else {
                None
            }
        })
        .filter(|token| !token.to_ascii_lowercase().contains("production"))
        .collect::<Vec<_>>();

    if paths.is_empty() {
        paths.push("src/**".to_string());
        paths.push("tests/**".to_string());
    }
    paths.sort();
    paths.dedup();
    paths
}

fn infer_required_evidence(prompt: &str, findings: &Findings) -> Vec<RequiredEvidence> {
    let lower = prompt.to_ascii_lowercase();
    let mut evidence = Vec::new();
    if lower.contains("lint") {
        evidence.push(RequiredEvidence {
            id: "lint".to_string(),
            description: "Lint command passes.".to_string(),
            kind: "lint".to_string(),
        });
    }
    if lower.contains("typecheck") || lower.contains("type check") {
        evidence.push(RequiredEvidence {
            id: "typecheck".to_string(),
            description: "Typecheck command passes.".to_string(),
            kind: "typecheck".to_string(),
        });
    }
    if lower.contains("test") || findings.missing_completion {
        evidence.push(RequiredEvidence {
            id: "tests".to_string(),
            description: "Relevant tests pass and existing tests remain enabled.".to_string(),
            kind: "test".to_string(),
        });
    }
    evidence.push(RequiredEvidence {
        id: "scope_check".to_string(),
        description: "Final changed files remain inside the task contract.".to_string(),
        kind: "scope".to_string(),
    });
    evidence.sort_by(|a, b| a.id.cmp(&b.id));
    evidence.dedup_by(|a, b| a.id == b.id);
    evidence
}

fn mentions_scope(prompt: &str) -> bool {
    prompt.contains('/')
        || prompt.contains('\\')
        || contains_any(
            &prompt.to_ascii_lowercase(),
            &[
                "module",
                "component",
                "service",
                "endpoint",
                "auth",
                "login",
                "frontend",
                "backend",
                "database",
                "api",
            ],
        )
}

fn contains_any(input: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| input.contains(needle))
}

fn push_if(reasons: &mut Vec<String>, condition: bool, reason: &str) {
    if condition {
        reasons.push(reason.to_string());
    }
}

fn dedup_sorted(items: &mut Vec<String>) {
    items.sort();
    items.dedup();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(prompt: &str) -> PromptIntakeRequest {
        PromptIntakeRequest {
            original_prompt: prompt.to_string(),
            session_id: "intake-test".to_string(),
            workspace_root: "/repo".to_string(),
            memory_context: Vec::new(),
            provider_mode: Some(ProviderMode::Disabled),
            semantic_config: None,
        }
    }

    #[test]
    fn scenario_a_broad_destructive_prompt_gets_safe_contract() {
        let result = analyze_prompt(request(
            "Fix everything and delete anything causing errors.",
        ));
        assert_eq!(result.status, IntakeStatus::ReadyWithSafeContract);
        assert!(result
            .reasons
            .contains(&"dangerously_broad_prompt".to_string()));
        assert!(result.reasons.contains(&"destructive_wording".to_string()));
        let contract = result.proposed_contract.unwrap();
        assert!(contract
            .forbidden_actions
            .contains(&"file_delete".to_string()));
        assert!(contract
            .forbidden_actions
            .contains(&"delete_tests".to_string()));
        assert!(contract.required_evidence.iter().any(|e| e.id == "tests"));
    }

    #[test]
    fn disable_security_is_rejected() {
        let result = analyze_prompt(request("Disable security so the feature runs."));
        assert_eq!(result.status, IntakeStatus::RejectedAsUnsafe);
        assert!(result.proposed_contract.is_none());
    }

    #[test]
    fn production_database_requires_clarification() {
        let result = analyze_prompt(request("Use my production database and test it."));
        assert_eq!(result.status, IntakeStatus::ClarificationRequired);
        assert!(result.questions.iter().any(|q| q.contains("environment")));
    }
}
