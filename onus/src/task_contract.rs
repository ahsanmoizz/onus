use crate::ipc::Action;
use crate::security;
use crate::{ActionType, Verdict};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeBudget {
    #[serde(default = "default_max_files_changed")]
    pub max_files_changed: u32,
    #[serde(default = "default_max_actions")]
    pub max_actions: u32,
}

fn default_max_files_changed() -> u32 {
    25
}

fn default_max_actions() -> u32 {
    500
}

impl Default for ChangeBudget {
    fn default() -> Self {
        Self {
            max_files_changed: default_max_files_changed(),
            max_actions: default_max_actions(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredEvidence {
    pub id: String,
    pub description: String,
    #[serde(default = "default_evidence_kind")]
    pub kind: String,
}

fn default_evidence_kind() -> String {
    "manual".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionEvidence {
    pub id: String,
    #[serde(default)]
    pub passed: bool,
    #[serde(default)]
    pub value: String,
    #[serde(default = "default_evidence_kind")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskContract {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub session_id: String,
    pub original_prompt: String,
    pub normalized_objective: String,
    #[serde(default)]
    pub allowed_paths: Vec<String>,
    #[serde(default)]
    pub allowed_resources: Vec<String>,
    #[serde(default)]
    pub protected_paths: Vec<String>,
    #[serde(default)]
    pub protected_resources: Vec<String>,
    #[serde(default)]
    pub required_evidence: Vec<RequiredEvidence>,
    #[serde(default)]
    pub forbidden_actions: Vec<String>,
    #[serde(default)]
    pub approval_required_actions: Vec<String>,
    #[serde(default)]
    pub change_budget: ChangeBudget,
    #[serde(default)]
    pub environment_identity: String,
    #[serde(default)]
    pub policy_version: String,
    #[serde(default)]
    pub canonical_hash: String,
}

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractViolation {
    pub verdict: Verdict,
    pub rule_id: String,
    pub rule_name: String,
    pub correction: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionStatus {
    CompletedVerified,
    CompletedWithExceptions {
        exceptions: Vec<String>,
    },
    HumanReviewRequired {
        missing_evidence: Vec<String>,
        findings: Vec<crate::quality::QualityFinding>,
    },
    FailedSafely {
        findings: Vec<crate::quality::QualityFinding>,
    },
    RolledBack {
        findings: Vec<crate::quality::QualityFinding>,
    },
    Terminated {
        findings: Vec<crate::quality::QualityFinding>,
    },
}

impl CompletionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CompletedVerified => "COMPLETED_VERIFIED",
            Self::CompletedWithExceptions { .. } => "COMPLETED_WITH_EXCEPTIONS",
            Self::HumanReviewRequired { .. } => "HUMAN_REVIEW_REQUIRED",
            Self::FailedSafely { .. } => "FAILED_SAFELY",
            Self::RolledBack { .. } => "ROLLED_BACK",
            Self::Terminated { .. } => "TERMINATED",
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::CompletedVerified | Self::CompletedWithExceptions { .. } => 0,
            Self::HumanReviewRequired { .. } => 4,
            Self::FailedSafely { .. } => 5,
            Self::RolledBack { .. } => 6,
            Self::Terminated { .. } => 7,
        }
    }
}

impl TaskContract {
    pub fn finalized(mut self) -> Self {
        if self.environment_identity.trim().is_empty() {
            self.environment_identity = security::environment_identity();
        }
        if self.policy_version.trim().is_empty() {
            self.policy_version = security::policy_version();
        }
        self.canonical_hash = self.compute_hash();
        self
    }

    pub fn compute_hash(&self) -> String {
        let mut clone = self.clone();
        clone.canonical_hash.clear();
        let value = serde_json::to_value(&clone).unwrap_or(Value::Null);
        security::sha256_hex(&security::canonical_json(&value))
    }

    pub fn verify_hash(&self) -> bool {
        self.canonical_hash == self.compute_hash()
    }
}

pub fn missing_contract_behavior() -> String {
    std::env::var("ONUS_MISSING_CONTRACT").unwrap_or_else(|_| "block_mutating".to_string())
}

pub fn evaluate_missing_contract(action: &Action) -> Option<ContractViolation> {
    match missing_contract_behavior().as_str() {
        "allow_legacy" => None,
        "block_all" => Some(contract_required_violation()),
        _ if is_mutating_action(&action.action_type) => Some(contract_required_violation()),
        _ => None,
    }
}

fn contract_required_violation() -> ContractViolation {
    ContractViolation {
        verdict: Verdict::Block,
        rule_id: "ONUS_CONTRACT_REQUIRED".to_string(),
        rule_name: "task-contract-required".to_string(),
        correction: "This governed action has no persisted task contract. Provide a contract or explicitly configure ONUS_MISSING_CONTRACT=allow_legacy for legacy behavior.".to_string(),
    }
}

pub fn evaluate_action(
    contract: &TaskContract,
    action: &Action,
    workspace_root: &Path,
    touched_paths: &[String],
) -> Option<ContractViolation> {
    let action_name = action_type_name(&action.action_type);
    let tool = action.tool.to_ascii_lowercase();
    if matches_contract_item(&contract.forbidden_actions, action_name)
        || matches_contract_item(&contract.forbidden_actions, &tool)
    {
        return Some(ContractViolation {
            verdict: Verdict::Block,
            rule_id: "ONUS_CONTRACT_FORBIDDEN_ACTION".to_string(),
            rule_name: "forbidden-action".to_string(),
            correction: format!("The task contract forbids action '{}'.", action_name),
        });
    }

    if requires_approval(contract, action) {
        return Some(ContractViolation {
            verdict: Verdict::Escalate,
            rule_id: "ONUS_CONTRACT_APPROVAL_REQUIRED".to_string(),
            rule_name: "contract-approval-required".to_string(),
            correction: format!(
                "The task contract requires human approval for '{}'.",
                action_name
            ),
        });
    }

    if let Some(path) = extract_action_path(action) {
        let normalized = normalize_path(workspace_root, &path);
        if matches_any_path(&contract.protected_paths, workspace_root, &normalized) {
            return Some(ContractViolation {
                verdict: Verdict::Block,
                rule_id: "ONUS_CONTRACT_PROTECTED_PATH".to_string(),
                rule_name: "protected-path".to_string(),
                correction: format!("The task contract protects '{}'.", path),
            });
        }

        if !contract.allowed_paths.is_empty()
            && !matches_any_path(&contract.allowed_paths, workspace_root, &normalized)
        {
            return Some(ContractViolation {
                verdict: Verdict::Block,
                rule_id: "ONUS_CONTRACT_OUT_OF_SCOPE".to_string(),
                rule_name: "out-of-scope-path".to_string(),
                correction: format!("The path '{}' is outside the task contract.", path),
            });
        }

        let mut unique: BTreeSet<String> = touched_paths.iter().cloned().collect();
        unique.insert(normalized.to_string_lossy().to_string());
        if unique.len() as u32 > contract.change_budget.max_files_changed {
            return Some(ContractViolation {
                verdict: Verdict::Block,
                rule_id: "ONUS_CONTRACT_CHANGE_BUDGET".to_string(),
                rule_name: "change-budget-exceeded".to_string(),
                correction: format!(
                    "The action would exceed the contract file-change budget of {}.",
                    contract.change_budget.max_files_changed
                ),
            });
        }
    }

    None
}

pub fn verify_completion(
    contract: &TaskContract,
    evidence: &[CompletionEvidence],
) -> CompletionStatus {
    let passed: BTreeSet<&str> = evidence
        .iter()
        .filter(|e| e.passed)
        .map(|e| e.id.as_str())
        .collect();
    let missing = contract
        .required_evidence
        .iter()
        .filter(|req| !passed.contains(req.id.as_str()))
        .map(|req| req.id.clone())
        .collect::<Vec<_>>();

    if missing.is_empty() {
        CompletionStatus::CompletedVerified
    } else {
        CompletionStatus::HumanReviewRequired {
            missing_evidence: missing,
            findings: Vec::new(),
        }
    }
}

pub fn extract_action_path(action: &Action) -> Option<String> {
    match &action.payload {
        Value::Object(map) => map
            .get("path")
            .and_then(|v| v.as_str())
            .or_else(|| map.get("file_path").and_then(|v| v.as_str()))
            .or_else(|| map.get("db_path").and_then(|v| v.as_str()))
            .map(|s| s.to_string()),
        _ => None,
    }
}

fn requires_approval(contract: &TaskContract, action: &Action) -> bool {
    let action_name = action_type_name(&action.action_type);
    if matches_contract_item(&contract.approval_required_actions, action_name) {
        return true;
    }
    if matches_contract_item(
        &contract.approval_required_actions,
        &action.tool.to_ascii_lowercase(),
    ) {
        return true;
    }
    if action.action_type == ActionType::DbMutation && is_schema_migration(&action.payload) {
        return matches_contract_item(&contract.approval_required_actions, "db_migration")
            || matches_contract_item(&contract.approval_required_actions, "database_migration");
    }
    false
}

fn is_schema_migration(payload: &Value) -> bool {
    let sql = payload
        .get("sql")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    [
        "alter table",
        "create table",
        "drop table",
        "create index",
        "drop index",
    ]
    .iter()
    .any(|needle| sql.contains(needle))
}

fn is_mutating_action(action_type: &ActionType) -> bool {
    matches!(
        action_type,
        ActionType::FileWrite
            | ActionType::FileDelete
            | ActionType::Git
            | ActionType::ApiCall
            | ActionType::DbMutation
            | ActionType::Network
            | ActionType::MCP
    )
}

fn action_type_name(action_type: &ActionType) -> &'static str {
    match action_type {
        ActionType::Shell => "shell",
        ActionType::FileWrite => "file_write",
        ActionType::FileDelete => "file_delete",
        ActionType::FileRead => "file_read",
        ActionType::Git => "git",
        ActionType::ApiCall => "api_call",
        ActionType::DbMutation => "db_mutation",
        ActionType::Network => "network",
        ActionType::MCP => "mcp",
    }
}

fn matches_contract_item(items: &[String], needle: &str) -> bool {
    let needle = needle.to_ascii_lowercase();
    items.iter().any(|item| item.to_ascii_lowercase() == needle)
}

fn matches_any_path(patterns: &[String], workspace_root: &Path, candidate: &Path) -> bool {
    patterns
        .iter()
        .any(|pattern| path_matches(pattern, workspace_root, candidate))
}

fn path_matches(pattern: &str, workspace_root: &Path, candidate: &Path) -> bool {
    let normalized_pattern = pattern.replace('\\', "/");
    let candidate_s = candidate.to_string_lossy().replace('\\', "/");
    let root_s = workspace_root.to_string_lossy().replace('\\', "/");
    let absolute_pattern = normalize_path(workspace_root, pattern)
        .to_string_lossy()
        .replace('\\', "/");

    if normalized_pattern.ends_with("/**") {
        let prefix = normalized_pattern.trim_end_matches("/**");
        let abs_prefix = normalize_path(workspace_root, prefix)
            .to_string_lossy()
            .replace('\\', "/");
        return candidate_s == abs_prefix || candidate_s.starts_with(&(abs_prefix + "/"));
    }

    candidate_s == absolute_pattern
        || candidate_s.starts_with(&(absolute_pattern.clone() + "/"))
        || candidate_s.strip_prefix(&(root_s + "/")) == Some(normalized_pattern.as_str())
}

fn normalize_path(workspace_root: &Path, path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        workspace_root.join(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::Action;

    fn contract() -> TaskContract {
        TaskContract {
            schema_version: 1,
            session_id: "s1".to_string(),
            original_prompt: "Fix auth".to_string(),
            normalized_objective: "Fix auth expiry".to_string(),
            allowed_paths: vec!["src/auth/**".to_string()],
            allowed_resources: vec![],
            protected_paths: vec![".env".to_string()],
            protected_resources: vec![],
            required_evidence: vec![RequiredEvidence {
                id: "tests".to_string(),
                description: "Tests pass".to_string(),
                kind: "test".to_string(),
            }],
            forbidden_actions: vec!["file_delete".to_string()],
            approval_required_actions: vec!["db_migration".to_string()],
            change_budget: ChangeBudget {
                max_files_changed: 1,
                max_actions: 10,
            },
            environment_identity: "env".to_string(),
            policy_version: "policy".to_string(),
            canonical_hash: String::new(),
        }
        .finalized()
    }

    #[test]
    fn canonical_hash_is_stable() {
        let a = contract();
        let b = contract();
        assert_eq!(a.canonical_hash, b.canonical_hash);
        assert!(a.verify_hash());
    }

    #[test]
    fn protected_path_blocks() {
        let action = Action {
            action_type: ActionType::FileWrite,
            tool: "Write".to_string(),
            payload: serde_json::json!({"path": ".env", "content": "x"}),
        };
        let violation = evaluate_action(&contract(), &action, Path::new("/repo"), &[]).unwrap();
        assert_eq!(violation.rule_id, "ONUS_CONTRACT_PROTECTED_PATH");
    }

    #[test]
    fn missing_evidence_rejects_completion() {
        let status = verify_completion(&contract(), &[]);
        assert!(matches!(
            status,
            CompletionStatus::HumanReviewRequired { .. }
        ));
    }
}
