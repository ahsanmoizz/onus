use crate::audit::db::{ActionRecord, PendingApproval};
use crate::ipc::Action;
use crate::task_contract::{self, CompletionEvidence, TaskContract};
use crate::ActionType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityFinding {
    pub id: String,
    pub severity: FindingSeverity,
    pub message: String,
    pub required_evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FindingSeverity {
    Info,
    Warning,
    Blocking,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionVerification {
    pub missing_evidence: Vec<String>,
    pub findings: Vec<QualityFinding>,
}

const BASE_REQUIRED_EVIDENCE: &[(&str, &str)] = &[
    (
        "targeted_tests",
        "Run targeted tests for the touched module.",
    ),
    ("lint", "Run the configured linter."),
    (
        "typecheck",
        "Run the configured type checker or compiler check.",
    ),
    ("coverage", "Prove coverage did not decrease."),
    ("secret_scan", "Run a secret scan after changes."),
    (
        "architecture_review",
        "Verify architecture rules remain satisfied.",
    ),
    (
        "module_boundary_review",
        "Verify module boundaries remain satisfied.",
    ),
    (
        "final_scope",
        "Verify the final diff remains inside the task contract.",
    ),
    (
        "independent_verification",
        "Provide independent verification beyond the agent completion statement.",
    ),
    (
        "no_test_weakening",
        "Verify tests were not deleted, skipped, disabled, or weakened.",
    ),
];

const DEPENDENCY_EVIDENCE: &str = "dependency_review";
const CONFIG_EVIDENCE: &str = "configuration_review";
const SNAPSHOT_EVIDENCE: &str = "snapshot_review";

pub fn verify_completion(
    contract: &TaskContract,
    workspace_root: &Path,
    actions: &[ActionRecord],
    pending_approvals: &[PendingApproval],
    evidence: &[CompletionEvidence],
) -> CompletionVerification {
    let mut required = required_evidence_ids(contract, actions);
    let mut findings = Vec::new();

    for action in actions {
        findings.extend(quality_findings_for_action(
            contract,
            workspace_root,
            action,
        ));
    }

    findings.extend(coverage_findings(evidence));
    findings.extend(unresolved_approval_findings(pending_approvals));

    if evidence.iter().any(is_agent_statement) {
        findings.push(QualityFinding {
            id: "AGENT_STATEMENT_NOT_EVIDENCE".to_string(),
            severity: FindingSeverity::Blocking,
            message: "The agent's statement that work is complete is not evidence.".to_string(),
            required_evidence: vec!["independent_verification".to_string()],
        });
    }

    for finding in &findings {
        for id in &finding.required_evidence {
            required.insert(id.clone());
        }
    }

    let passed = evidence
        .iter()
        .filter(|item| item.passed && !is_agent_statement(item))
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    let missing_evidence = required
        .into_iter()
        .filter(|id| !passed.contains(id.as_str()))
        .collect::<Vec<_>>();

    CompletionVerification {
        missing_evidence,
        findings,
    }
}

fn required_evidence_ids(contract: &TaskContract, actions: &[ActionRecord]) -> BTreeSet<String> {
    let mut required = contract
        .required_evidence
        .iter()
        .map(|item| item.id.clone())
        .collect::<BTreeSet<_>>();
    for (id, _description) in BASE_REQUIRED_EVIDENCE {
        required.insert((*id).to_string());
    }
    if actions.iter().any(touches_dependency_manifest) {
        required.insert(DEPENDENCY_EVIDENCE.to_string());
    }
    if actions.iter().any(touches_configuration) {
        required.insert(CONFIG_EVIDENCE.to_string());
    }
    if actions.iter().any(touches_snapshot) {
        required.insert(SNAPSHOT_EVIDENCE.to_string());
    }
    required
}

pub fn base_required_evidence() -> Vec<crate::task_contract::RequiredEvidence> {
    BASE_REQUIRED_EVIDENCE
        .iter()
        .map(|(id, description)| crate::task_contract::RequiredEvidence {
            id: (*id).to_string(),
            description: (*description).to_string(),
            kind: "quality".to_string(),
        })
        .collect()
}

fn quality_findings_for_action(
    contract: &TaskContract,
    workspace_root: &Path,
    action: &ActionRecord,
) -> Vec<QualityFinding> {
    let mut findings = Vec::new();
    let payload = parse_payload(action);
    let path = action_path(&payload);
    let before = payload_string(&payload, "before_content");
    let after =
        payload_string(&payload, "after_content").or_else(|| payload_string(&payload, "content"));
    let action_type = action_type(action);

    if action.rule_id.as_deref() == Some("SECRET_001")
        || (action
            .payload_classification
            .contains("\"contains_sensitive\":true")
            && action.verdict != "block")
    {
        findings.push(QualityFinding {
            id: "HARDCODED_SECRET_DETECTED".to_string(),
            severity: FindingSeverity::Critical,
            message: "A hardcoded secret was detected in a governed action.".to_string(),
            required_evidence: vec!["secret_scan".to_string()],
        });
    }

    if is_test_path(path.as_deref()) {
        if action.action_type == "file_delete" {
            findings.push(QualityFinding {
                id: "TEST_DELETION_DETECTED".to_string(),
                severity: FindingSeverity::Critical,
                message: "A test file deletion was attempted or recorded.".to_string(),
                required_evidence: vec!["tests".to_string(), "no_test_weakening".to_string()],
            });
        }
        if skip_count(after.as_deref()) > skip_count(before.as_deref()) {
            findings.push(QualityFinding {
                id: "SKIPPED_TEST_DETECTED".to_string(),
                severity: FindingSeverity::Blocking,
                message: "Skipped, disabled, focused, or ignored tests increased.".to_string(),
                required_evidence: vec!["tests".to_string(), "no_test_weakening".to_string()],
            });
        }
        if assertion_count(after.as_deref()) < assertion_count(before.as_deref()) {
            findings.push(QualityFinding {
                id: "ASSERTION_WEAKENING_DETECTED".to_string(),
                severity: FindingSeverity::Blocking,
                message: "Test assertions decreased in a modified test file.".to_string(),
                required_evidence: vec!["tests".to_string(), "no_test_weakening".to_string()],
            });
        }
    }

    if is_snapshot_path(path.as_deref()) && matches!(action_type, Some(ActionType::FileWrite)) {
        findings.push(QualityFinding {
            id: "SUSPICIOUS_SNAPSHOT_CHANGE".to_string(),
            severity: FindingSeverity::Warning,
            message: "Snapshot output changed and requires explicit review evidence.".to_string(),
            required_evidence: vec![SNAPSHOT_EVIDENCE.to_string()],
        });
    }

    if let Some(path) = path.as_deref() {
        let synthetic = Action {
            action_type: action_type.unwrap_or(ActionType::Shell),
            tool: action.tool_name.clone().unwrap_or_default(),
            payload: serde_json::json!({"path": path, "file_path": path}),
        };
        if let Some(violation) =
            task_contract::evaluate_action(contract, &synthetic, workspace_root, &[])
        {
            if violation.rule_id != "ONUS_CONTRACT_APPROVAL_REQUIRED" {
                findings.push(QualityFinding {
                    id: "FINAL_SCOPE_VIOLATION".to_string(),
                    severity: FindingSeverity::Blocking,
                    message: format!(
                        "Final action trace violates task scope: {}",
                        violation.correction
                    ),
                    required_evidence: vec!["final_scope".to_string()],
                });
            }
        }
    }

    findings
}

fn coverage_findings(evidence: &[CompletionEvidence]) -> Vec<QualityFinding> {
    evidence
        .iter()
        .filter(|item| item.id == "coverage")
        .filter_map(|item| {
            let (before, after) = parse_coverage_pair(&item.value)?;
            if after + f64::EPSILON < before {
                Some(QualityFinding {
                    id: "COVERAGE_REDUCTION_DETECTED".to_string(),
                    severity: FindingSeverity::Blocking,
                    message: format!("Coverage decreased from {} to {}.", before, after),
                    required_evidence: vec!["coverage".to_string()],
                })
            } else {
                None
            }
        })
        .collect()
}

fn unresolved_approval_findings(pending_approvals: &[PendingApproval]) -> Vec<QualityFinding> {
    pending_approvals
        .iter()
        .filter(|approval| approval.status == "pending")
        .map(|approval| QualityFinding {
            id: "UNRESOLVED_APPROVAL".to_string(),
            severity: FindingSeverity::Blocking,
            message: format!(
                "Pending approval {} remains unresolved.",
                approval.action_id
            ),
            required_evidence: vec!["independent_verification".to_string()],
        })
        .collect()
}

fn parse_coverage_pair(input: &str) -> Option<(f64, f64)> {
    if let Ok(value) = serde_json::from_str::<Value>(input) {
        let before = value.get("before").and_then(|v| v.as_f64())?;
        let after = value.get("after").and_then(|v| v.as_f64())?;
        return Some((before, after));
    }
    let mut before = None;
    let mut after = None;
    for part in input.split([',', ';', ' ']) {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        let parsed = value.trim_end_matches('%').parse::<f64>().ok()?;
        match key.trim().to_ascii_lowercase().as_str() {
            "before" => before = Some(parsed),
            "after" => after = Some(parsed),
            _ => {}
        }
    }
    Some((before?, after?))
}

fn is_agent_statement(evidence: &CompletionEvidence) -> bool {
    evidence.kind.eq_ignore_ascii_case("agent_statement")
        || evidence.id.eq_ignore_ascii_case("agent_statement")
        || evidence.id.eq_ignore_ascii_case("agent_complete")
}

fn parse_payload(action: &ActionRecord) -> Value {
    serde_json::from_str(&action.payload).unwrap_or(Value::Null)
}

fn payload_string(payload: &Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
}

fn action_path(payload: &Value) -> Option<String> {
    payload
        .get("path")
        .and_then(|value| value.as_str())
        .or_else(|| payload.get("file_path").and_then(|value| value.as_str()))
        .or_else(|| payload.get("db_path").and_then(|value| value.as_str()))
        .map(ToString::to_string)
}

fn action_type(action: &ActionRecord) -> Option<ActionType> {
    match action.action_type.as_str() {
        "shell" => Some(ActionType::Shell),
        "file_write" => Some(ActionType::FileWrite),
        "file_delete" => Some(ActionType::FileDelete),
        "file_read" => Some(ActionType::FileRead),
        "git" => Some(ActionType::Git),
        "api_call" => Some(ActionType::ApiCall),
        "db_mutation" => Some(ActionType::DbMutation),
        "network" => Some(ActionType::Network),
        "mcp" => Some(ActionType::MCP),
        _ => None,
    }
}

fn is_test_path(path: Option<&str>) -> bool {
    let Some(path) = path else {
        return false;
    };
    let path = normalize(path);
    path.contains("/test")
        || path.starts_with("test")
        || path.contains("_test.")
        || path.contains(".spec.")
        || path.contains(".test.")
}

fn is_snapshot_path(path: Option<&str>) -> bool {
    let Some(path) = path else {
        return false;
    };
    let path = normalize(path);
    path.ends_with(".snap")
        || path.ends_with(".snapshot")
        || path.contains("__snapshots__")
        || path.contains("/snapshots/")
}

fn skip_count(input: Option<&str>) -> usize {
    let Some(input) = input else {
        return 0;
    };
    let lower = input.to_ascii_lowercase();
    [
        "pytest.mark.skip",
        "@unittest.skip",
        "unittest.skip",
        "describe.skip",
        "it.skip",
        "test.skip",
        "context.skip",
        "#[ignore]",
        "xfail",
        ".only(",
        "todo(",
        "skip(",
        "disabled",
    ]
    .iter()
    .map(|needle| lower.matches(needle).count())
    .sum()
}

fn assertion_count(input: Option<&str>) -> usize {
    let Some(input) = input else {
        return 0;
    };
    [
        "assert ",
        "assert(",
        "assert_eq!",
        "assert_ne!",
        "expect(",
        "should",
        "to_equal",
        "to_be",
        "toHave",
    ]
    .iter()
    .map(|needle| input.matches(needle).count())
    .sum()
}

fn touches_dependency_manifest(action: &ActionRecord) -> bool {
    path_from_action(action).is_some_and(|path| {
        matches!(
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(""),
            "Cargo.toml"
                | "Cargo.lock"
                | "package.json"
                | "package-lock.json"
                | "pnpm-lock.yaml"
                | "yarn.lock"
                | "requirements.txt"
                | "pyproject.toml"
                | "poetry.lock"
                | "Pipfile"
                | "Pipfile.lock"
                | "go.mod"
                | "go.sum"
        )
    })
}

fn touches_configuration(action: &ActionRecord) -> bool {
    path_from_action(action).is_some_and(|path| {
        let path_s = normalize(&path.to_string_lossy());
        path_s.ends_with(".env")
            || path_s.contains("/config/")
            || path_s.contains("/.github/workflows/")
            || path_s.ends_with(".yml")
            || path_s.ends_with(".yaml")
            || path_s.ends_with(".toml")
            || path_s.ends_with(".ini")
    })
}

fn touches_snapshot(action: &ActionRecord) -> bool {
    path_from_action(action).is_some_and(|path| is_snapshot_path(Some(&path.to_string_lossy())))
}

fn path_from_action(action: &ActionRecord) -> Option<PathBuf> {
    let payload = parse_payload(action);
    action_path(&payload).map(PathBuf::from)
}

fn normalize(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(
        action_type: &str,
        path: &str,
        before: Option<&str>,
        after: Option<&str>,
    ) -> ActionRecord {
        ActionRecord {
            id: "a1".to_string(),
            session_id: "s1".to_string(),
            sequence: 1,
            action_type: action_type.to_string(),
            tool_name: Some("Write".to_string()),
            payload: serde_json::json!({
                "path": path,
                "file_path": path,
                "before_content": before,
                "after_content": after,
                "content": after,
            })
            .to_string(),
            payload_hash: "hash".to_string(),
            payload_classification: "{}".to_string(),
            verdict: "allow".to_string(),
            rule_id: None,
            correction: None,
            approval_decision: None,
            guardian_mode: None,
            obligations_json: None,
            approval_reason: None,
            prev_hash: String::new(),
            hash: "hash".to_string(),
            created_at: "now".to_string(),
        }
    }

    fn contract() -> TaskContract {
        TaskContract {
            schema_version: 1,
            session_id: "s1".to_string(),
            original_prompt: "Fix tests".to_string(),
            normalized_objective: "Fix tests".to_string(),
            allowed_paths: vec!["tests/**".to_string(), "src/**".to_string()],
            allowed_resources: vec![],
            protected_paths: vec![],
            protected_resources: vec![],
            required_evidence: vec![crate::task_contract::RequiredEvidence {
                id: "tests".to_string(),
                description: "Tests pass".to_string(),
                kind: "test".to_string(),
            }],
            forbidden_actions: vec![],
            approval_required_actions: vec![],
            change_budget: Default::default(),
            environment_identity: "test".to_string(),
            policy_version: "test".to_string(),
            canonical_hash: String::new(),
        }
        .finalized()
    }

    #[test]
    fn detects_skipped_tests_and_assertion_weakening() {
        let actions = vec![action(
            "file_write",
            "tests/test_auth.py",
            Some("def test_auth():\n    assert login()\n    assert token()\n"),
            Some("@pytest.mark.skip\ndef test_auth():\n    assert login()\n"),
        )];
        let result = verify_completion(&contract(), Path::new("."), &actions, &[], &[]);
        assert!(result
            .findings
            .iter()
            .any(|finding| finding.id == "SKIPPED_TEST_DETECTED"));
        assert!(result
            .findings
            .iter()
            .any(|finding| finding.id == "ASSERTION_WEAKENING_DETECTED"));
    }

    #[test]
    fn agent_statement_is_not_evidence() {
        let result = verify_completion(
            &contract(),
            Path::new("."),
            &[],
            &[],
            &[CompletionEvidence {
                id: "agent_complete".to_string(),
                passed: true,
                value: "done".to_string(),
                kind: "agent_statement".to_string(),
            }],
        );
        assert!(result
            .findings
            .iter()
            .any(|finding| finding.id == "AGENT_STATEMENT_NOT_EVIDENCE"));
        assert!(result.missing_evidence.contains(&"tests".to_string()));
    }
}
