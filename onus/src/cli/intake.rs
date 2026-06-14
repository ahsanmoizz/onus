use crate::audit::AuditTrail;
use crate::prompt_intake::{analyze_prompt, IntakeStatus, PromptIntakeRequest, ProviderMode};
use crate::semantic::{PrivacyMode, SemanticFallbackPolicy, SemanticReviewerConfig};
use clap::{Args, ValueEnum};
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Args)]
pub struct IntakeArgs {
    /// Original user prompt. If omitted, stdin is used.
    #[arg(long)]
    pub prompt: Option<String>,

    /// Path to a file containing the original user prompt.
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Path to audit database.
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Workspace root for the proposed contract/session.
    #[arg(long)]
    pub workspace_root: Option<PathBuf>,

    /// Session ID. If omitted, one is generated.
    #[arg(long)]
    pub session_id: Option<String>,

    /// Agent name for persisted session rows.
    #[arg(long, default_value = "prompt-intake")]
    pub agent_name: String,

    /// Persist the proposed contract and create the session when intake is ready.
    #[arg(long)]
    pub start_session: bool,

    /// Semantic provider mode. Disabled means no LLM is invoked.
    #[arg(long, value_enum, default_value_t = IntakeProviderArg::Disabled)]
    pub provider: IntakeProviderArg,

    /// Semantic provider model name. Does not enable a provider by itself.
    #[arg(long)]
    pub semantic_model: Option<String>,

    /// Cloud provider endpoint or local model server endpoint.
    #[arg(long)]
    pub semantic_endpoint: Option<String>,

    /// Environment variable name holding the cloud provider API key.
    #[arg(long)]
    pub semantic_api_key_env: Option<String>,

    /// Local model adapter command. The command receives a JSON provider request on stdin.
    #[arg(long)]
    pub semantic_local_command: Option<String>,

    /// Semantic provider timeout in milliseconds.
    #[arg(long)]
    pub semantic_timeout_ms: Option<u64>,

    /// Privacy mode for semantic review payloads.
    #[arg(long, value_enum, default_value_t = IntakePrivacyArg::Strict)]
    pub semantic_privacy: IntakePrivacyArg,

    /// Disable semantic redaction before provider calls.
    #[arg(long)]
    pub no_semantic_redaction: bool,

    /// Maximum estimated provider tokens for semantic review.
    #[arg(long)]
    pub semantic_token_budget: Option<u32>,

    /// Maximum estimated semantic cost in micro-USD.
    #[arg(long)]
    pub semantic_cost_budget_micro_usd: Option<u64>,

    /// Estimated semantic cost per 1k tokens in micro-USD.
    #[arg(long)]
    pub semantic_cost_per_1k_tokens_micro_usd: Option<u64>,

    /// Fail closed instead of falling back when a critical semantic review fails.
    #[arg(long)]
    pub semantic_fail_closed: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum IntakeProviderArg {
    Deterministic,
    Disabled,
    Local,
    Cloud,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum IntakePrivacyArg {
    Strict,
    Balanced,
    Off,
}

pub fn run(args: IntakeArgs) -> anyhow::Result<()> {
    let prompt = read_prompt(args.prompt.as_deref(), args.file.as_ref())?;
    let workspace = args
        .workspace_root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let session_id = args
        .session_id
        .clone()
        .unwrap_or_else(|| format!("intake-{}", uuid::Uuid::new_v4()));
    let db_path = args
        .db
        .clone()
        .unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let memory_context = crate::memory::MemoryStore::open(&db_path)
        .and_then(|store| {
            store.retrieve_relevant(
                &crate::memory::MemoryScope::for_workspace(
                    &workspace.display().to_string(),
                    Some(session_id.clone()),
                ),
                &prompt,
                8,
            )
        })
        .map(|items| items.into_iter().map(|item| item.to_string()).collect())
        .unwrap_or_else(|e| {
            log::warn!("Failed to retrieve scoped memory for prompt intake: {}", e);
            Vec::new()
        });

    let semantic_config = semantic_config_from_args(&args);

    let mut result = analyze_prompt(PromptIntakeRequest {
        original_prompt: prompt,
        session_id: session_id.clone(),
        workspace_root: workspace.display().to_string(),
        memory_context,
        provider_mode: Some(match args.provider {
            IntakeProviderArg::Deterministic => ProviderMode::Deterministic,
            IntakeProviderArg::Disabled => ProviderMode::Disabled,
            IntakeProviderArg::Local => ProviderMode::Local,
            IntakeProviderArg::Cloud => ProviderMode::Cloud,
        }),
        semantic_config: Some(semantic_config),
    });

    let mut session_started = false;
    let mut contract_hash = None;

    if args.start_session {
        match result.status {
            IntakeStatus::Ready | IntakeStatus::ReadyWithSafeContract => {
                let Some(contract) = result.proposed_contract.as_ref() else {
                    anyhow::bail!("Prompt intake returned no contract for a ready prompt");
                };
                let mut audit = AuditTrail::open(&db_path)?;
                audit.start_session(
                    &contract.session_id,
                    &args.agent_name,
                    None,
                    &contract.normalized_objective,
                    &workspace.display().to_string(),
                )?;
                let saved = audit.save_task_contract(contract)?;
                audit.remember_session_intake(
                    &saved.session_id,
                    &workspace.display().to_string(),
                    &saved.original_prompt,
                    &saved.normalized_objective,
                )?;
                contract_hash = Some(saved.canonical_hash.clone());
                result.proposed_contract = Some(saved);
                session_started = true;
            }
            IntakeStatus::ClarificationRequired | IntakeStatus::RejectedAsUnsafe => {}
        }
    }

    println!(
        "{}",
        serde_json::json!({
            "status": result.status,
            "provider_mode": result.provider_mode,
            "semantic_review": result.semantic_review,
            "semantic_roles": result.semantic_roles,
            "reasons": result.reasons,
            "questions": result.questions,
            "session_started": session_started,
            "session_id": session_id,
            "contract_hash": contract_hash,
            "proposed_contract": result.proposed_contract,
        })
    );

    Ok(())
}

fn semantic_config_from_args(args: &IntakeArgs) -> SemanticReviewerConfig {
    let mut config = SemanticReviewerConfig::from_env();
    config.model = args.semantic_model.clone().or(config.model);
    config.endpoint = args.semantic_endpoint.clone().or(config.endpoint);
    config.api_key_env = args.semantic_api_key_env.clone().or(config.api_key_env);
    config.local_command = args.semantic_local_command.clone().or(config.local_command);
    config.timeout_ms = args.semantic_timeout_ms.unwrap_or(config.timeout_ms);
    config.privacy_mode = match args.semantic_privacy {
        IntakePrivacyArg::Strict => PrivacyMode::Strict,
        IntakePrivacyArg::Balanced => PrivacyMode::Balanced,
        IntakePrivacyArg::Off => PrivacyMode::Off,
    };
    config.redact = !args.no_semantic_redaction;
    config.token_budget = args.semantic_token_budget.or(config.token_budget);
    config.cost_budget_micro_usd = args
        .semantic_cost_budget_micro_usd
        .or(config.cost_budget_micro_usd);
    config.estimated_cost_per_1k_tokens_micro_usd = args
        .semantic_cost_per_1k_tokens_micro_usd
        .unwrap_or(config.estimated_cost_per_1k_tokens_micro_usd);
    if args.semantic_fail_closed {
        config.fallback_policy = SemanticFallbackPolicy::FailClosed;
    }
    config
}

fn read_prompt(prompt: Option<&str>, file: Option<&PathBuf>) -> anyhow::Result<String> {
    if let Some(prompt) = prompt {
        return Ok(prompt.to_string());
    }
    if let Some(file) = file {
        return Ok(std::fs::read_to_string(file)?);
    }
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw)?;
    Ok(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt_intake::IntakeStatus;

    #[test]
    fn scenario_a_returns_safe_contract_without_starting_agent() {
        let result = analyze_prompt(crate::prompt_intake::PromptIntakeRequest {
            original_prompt: "Fix everything and delete anything causing errors.".to_string(),
            session_id: "scenario-a".to_string(),
            workspace_root: "/repo".to_string(),
            memory_context: Vec::new(),
            provider_mode: Some(ProviderMode::Disabled),
            semantic_config: None,
        });

        assert_eq!(result.status, IntakeStatus::ReadyWithSafeContract);
        assert!(result
            .reasons
            .contains(&"dangerously_broad_prompt".to_string()));
        let contract = result.proposed_contract.unwrap();
        assert_eq!(
            contract.original_prompt,
            "Fix everything and delete anything causing errors."
        );
        assert!(contract
            .forbidden_actions
            .contains(&"file_delete".to_string()));
    }
}
