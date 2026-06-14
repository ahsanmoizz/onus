use crate::ipc::{Action, ActionRequest};
use crate::security;
use crate::task_contract::{RequiredEvidence, TaskContract};
use crate::Verdict;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT_MS: u64 = 5_000;
const DEFAULT_MAX_INPUT_BYTES: usize = 16 * 1024;
const DEFAULT_TOKEN_BUDGET: u32 = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticProviderKind {
    Disabled,
    Deterministic,
    Local,
    Cloud,
    #[serde(skip)]
    Fixture,
}

impl std::fmt::Display for SemanticProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => write!(f, "disabled"),
            Self::Deterministic => write!(f, "deterministic"),
            Self::Local => write!(f, "local"),
            Self::Cloud => write!(f, "cloud"),
            Self::Fixture => write!(f, "fixture"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyMode {
    Strict,
    Balanced,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticFallbackPolicy {
    Deterministic,
    FailClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticReviewerConfig {
    pub provider: SemanticProviderKind,
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub api_key_env: Option<String>,
    pub local_command: Option<String>,
    pub timeout_ms: u64,
    pub privacy_mode: PrivacyMode,
    pub redact: bool,
    pub max_input_bytes: usize,
    pub token_budget: Option<u32>,
    pub cost_budget_micro_usd: Option<u64>,
    pub estimated_cost_per_1k_tokens_micro_usd: u64,
    pub fallback_policy: SemanticFallbackPolicy,
    pub fail_closed_on_critical: bool,
}

impl Default for SemanticReviewerConfig {
    fn default() -> Self {
        Self {
            provider: SemanticProviderKind::Disabled,
            model: None,
            endpoint: None,
            api_key_env: Some("ONUS_SEMANTIC_API_KEY".to_string()),
            local_command: None,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            privacy_mode: PrivacyMode::Strict,
            redact: true,
            max_input_bytes: DEFAULT_MAX_INPUT_BYTES,
            token_budget: Some(DEFAULT_TOKEN_BUDGET),
            cost_budget_micro_usd: None,
            estimated_cost_per_1k_tokens_micro_usd: 0,
            fallback_policy: SemanticFallbackPolicy::Deterministic,
            fail_closed_on_critical: true,
        }
    }
}

impl SemanticReviewerConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        if let Ok(provider) = std::env::var("ONUS_SEMANTIC_PROVIDER") {
            config.provider = parse_provider(&provider);
        }
        config.model = env_nonempty("ONUS_SEMANTIC_MODEL").or(config.model);
        config.endpoint = env_nonempty("ONUS_SEMANTIC_ENDPOINT").or(config.endpoint);
        config.local_command = env_nonempty("ONUS_SEMANTIC_LOCAL_COMMAND").or(config.local_command);
        config.api_key_env = env_nonempty("ONUS_SEMANTIC_API_KEY_ENV").or(config.api_key_env);
        config.timeout_ms = env_u64("ONUS_SEMANTIC_TIMEOUT_MS").unwrap_or(config.timeout_ms);
        config.privacy_mode = std::env::var("ONUS_SEMANTIC_PRIVACY_MODE")
            .ok()
            .map(|v| parse_privacy_mode(&v))
            .unwrap_or(config.privacy_mode);
        config.redact = std::env::var("ONUS_SEMANTIC_REDACT")
            .ok()
            .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false")))
            .unwrap_or(config.redact);
        config.max_input_bytes = env_u64("ONUS_SEMANTIC_MAX_INPUT_BYTES")
            .unwrap_or(config.max_input_bytes as u64) as usize;
        config.token_budget = env_u64("ONUS_SEMANTIC_TOKEN_BUDGET")
            .map(|v| v as u32)
            .or(config.token_budget);
        config.cost_budget_micro_usd =
            env_u64("ONUS_SEMANTIC_COST_BUDGET_MICRO_USD").or(config.cost_budget_micro_usd);
        config.estimated_cost_per_1k_tokens_micro_usd =
            env_u64("ONUS_SEMANTIC_COST_PER_1K_TOKENS_MICRO_USD")
                .unwrap_or(config.estimated_cost_per_1k_tokens_micro_usd);
        config.fallback_policy = std::env::var("ONUS_SEMANTIC_FALLBACK")
            .ok()
            .map(|v| parse_fallback(&v))
            .unwrap_or(config.fallback_policy);
        config.fail_closed_on_critical = std::env::var("ONUS_SEMANTIC_FAIL_CLOSED_CRITICAL")
            .ok()
            .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false")))
            .unwrap_or(config.fail_closed_on_critical);
        config
    }

    pub fn provider_enabled(&self) -> bool {
        matches!(
            self.provider,
            SemanticProviderKind::Local
                | SemanticProviderKind::Cloud
                | SemanticProviderKind::Fixture
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SemanticRole {
    IntentInterpreter,
    SemanticRiskCritic,
    StructuredCorrectionGenerator,
    IndependentCompletionVerifier,
    UserGuidanceAssistant,
}

impl SemanticRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IntentInterpreter => "intent_interpreter",
            Self::SemanticRiskCritic => "semantic_risk_critic",
            Self::StructuredCorrectionGenerator => "structured_correction_generator",
            Self::IndependentCompletionVerifier => "independent_completion_verifier",
            Self::UserGuidanceAssistant => "user_guidance_assistant",
        }
    }

    fn schema_name(&self) -> &'static str {
        match self {
            Self::IntentInterpreter => "TaskInterpretation",
            Self::SemanticRiskCritic => "SemanticAssessment",
            Self::StructuredCorrectionGenerator => "StructuredCorrection",
            Self::IndependentCompletionVerifier => "CompletionAssessment",
            Self::UserGuidanceAssistant => "UserGuidance",
        }
    }

    fn instruction(&self) -> &'static str {
        match self {
            Self::IntentInterpreter => {
                "Interpret the user's request. Return only valid JSON matching the schema. Do not approve actions."
            }
            Self::SemanticRiskCritic => {
                "Review semantic risk after deterministic policy. Return only valid JSON. Never lower a deterministic denial."
            }
            Self::StructuredCorrectionGenerator => {
                "Generate a structured correction. Label hypotheses; never invent verified root causes."
            }
            Self::IndependentCompletionVerifier => {
                "Independently verify completion evidence. The executor success statement is not proof."
            }
            Self::UserGuidanceAssistant => {
                "Explain risk and safer alternatives for the user. Do not execute or approve actions."
            }
        }
    }
}

impl std::fmt::Display for SemanticRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRoleTrace {
    pub role: SemanticRole,
    pub provider: SemanticProviderKind,
    pub provider_invoked: bool,
    pub fallback_used: bool,
    pub accepted: bool,
    pub error: Option<String>,
    pub estimated_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct SemanticCall<T> {
    pub output: T,
    pub trace: SemanticRoleTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInterpretationRequest {
    pub original_prompt: String,
    pub repository_metadata: Vec<String>,
    #[serde(default)]
    pub memory_context: Vec<String>,
    pub existing_policy: Vec<String>,
    pub current_environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskInterpretation {
    pub schema_version: u32,
    pub normalized_objective: String,
    pub allowed_scope: Vec<String>,
    pub protected_scope: Vec<String>,
    pub completion_evidence: Vec<String>,
    pub ambiguities: Vec<String>,
    pub risk_assumptions: Vec<String>,
    pub questions: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionReviewRequest {
    pub task_contract_hash: Option<String>,
    pub action: Action,
    pub relevant_diff: Option<String>,
    pub previous_actions: Vec<String>,
    pub repository_architecture: Vec<String>,
    pub deterministic_verdict: Verdict,
    pub policy_findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticAssessment {
    pub schema_version: u32,
    pub aligned_with_task: bool,
    pub proportionate: bool,
    pub quality_problems: Vec<String>,
    pub hidden_side_effects: Vec<String>,
    pub confidence: f32,
    pub recommended_decision: Verdict,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionRequest {
    pub deterministic_verdict: Verdict,
    pub rule_id: Option<String>,
    pub correction: Option<String>,
    #[serde(default)]
    pub memory_context: Vec<String>,
    pub task_contract: Option<TaskContract>,
    pub action: Option<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredCorrection {
    pub schema_version: u32,
    pub violation: String,
    pub reason: String,
    pub required_action: String,
    pub constraints: Vec<String>,
    pub required_evidence: Vec<String>,
    pub hypotheses: Vec<String>,
    pub retry_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionReviewRequest {
    pub original_task: String,
    pub task_contract: TaskContract,
    pub final_diff: String,
    pub action_trace: Vec<String>,
    pub denied_and_corrected_actions: Vec<String>,
    pub evidence: Vec<String>,
    pub policy_exceptions: Vec<String>,
    #[serde(default)]
    pub memory_context: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionAssessment {
    pub schema_version: u32,
    pub achieved_requested_outcome: bool,
    pub tests_weakened: bool,
    pub implementation_drifted: bool,
    pub security_reduced: bool,
    pub evidence_sufficient: bool,
    pub accept: bool,
    pub confidence: f32,
    pub findings: Vec<String>,
    pub required_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGuidanceRequest {
    pub action_summary: String,
    pub deterministic_verdict: Verdict,
    pub risk_context: Vec<String>,
    pub reversibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserGuidance {
    pub schema_version: u32,
    pub explanation: String,
    pub risks: Vec<String>,
    pub safer_alternative: String,
    pub reversibility: String,
    pub recommendation: String,
}

pub trait SemanticReviewer {
    fn interpret_task(
        &self,
        input: TaskInterpretationRequest,
    ) -> Result<SemanticCall<TaskInterpretation>, SemanticRoleTrace>;

    fn review_action(
        &self,
        input: ActionReviewRequest,
        critical: bool,
    ) -> Result<SemanticCall<SemanticAssessment>, SemanticRoleTrace>;

    fn generate_correction(
        &self,
        input: CorrectionRequest,
        critical: bool,
    ) -> Result<SemanticCall<StructuredCorrection>, SemanticRoleTrace>;

    fn verify_completion(
        &self,
        input: CompletionReviewRequest,
        critical: bool,
    ) -> Result<SemanticCall<CompletionAssessment>, SemanticRoleTrace>;

    fn guide_user(
        &self,
        input: UserGuidanceRequest,
    ) -> Result<SemanticCall<UserGuidance>, SemanticRoleTrace>;
}

#[derive(Debug, Clone)]
pub struct ConfiguredSemanticReviewer {
    config: SemanticReviewerConfig,
}

impl ConfiguredSemanticReviewer {
    pub fn new(config: SemanticReviewerConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &SemanticReviewerConfig {
        &self.config
    }

    fn request_role<T>(
        &self,
        role: SemanticRole,
        input: Value,
        fallback: T,
        critical: bool,
    ) -> Result<SemanticCall<T>, SemanticRoleTrace>
    where
        T: DeserializeOwned + TrustedSchema,
    {
        if !self.config.provider_enabled() {
            return Ok(SemanticCall {
                output: fallback,
                trace: SemanticRoleTrace {
                    role,
                    provider: self.config.provider.clone(),
                    provider_invoked: false,
                    fallback_used: true,
                    accepted: true,
                    error: None,
                    estimated_tokens: 0,
                },
            });
        }

        let prepared = match prepare_provider_request(role, &input, &self.config) {
            Ok(value) => value,
            Err(err) => return self.fallback_or_fail(role, fallback, critical, false, 0, err),
        };
        let estimated_tokens = estimate_tokens(&security::canonical_json(&prepared));

        let raw = match self.config.provider {
            SemanticProviderKind::Local => call_local_adapter(&prepared, &self.config),
            SemanticProviderKind::Cloud => call_cloud_adapter(&prepared, &self.config),
            SemanticProviderKind::Fixture => Err(SemanticError::Provider(
                "fixture provider must be supplied through FixtureSemanticReviewer".to_string(),
            )),
            SemanticProviderKind::Disabled | SemanticProviderKind::Deterministic => unreachable!(),
        };

        match raw.and_then(|text| parse_trusted_output::<T>(&text)) {
            Ok(output) => Ok(SemanticCall {
                output,
                trace: SemanticRoleTrace {
                    role,
                    provider: self.config.provider.clone(),
                    provider_invoked: true,
                    fallback_used: false,
                    accepted: true,
                    error: None,
                    estimated_tokens,
                },
            }),
            Err(err) => {
                self.fallback_or_fail(role, fallback, critical, true, estimated_tokens, err)
            }
        }
    }

    fn fallback_or_fail<T>(
        &self,
        role: SemanticRole,
        fallback: T,
        critical: bool,
        provider_invoked: bool,
        estimated_tokens: u32,
        err: SemanticError,
    ) -> Result<SemanticCall<T>, SemanticRoleTrace> {
        let trace = SemanticRoleTrace {
            role,
            provider: self.config.provider.clone(),
            provider_invoked,
            fallback_used: false,
            accepted: false,
            error: Some(err.to_string()),
            estimated_tokens,
        };

        if critical
            && self.config.fail_closed_on_critical
            && matches!(
                self.config.fallback_policy,
                SemanticFallbackPolicy::FailClosed
            )
        {
            return Err(trace);
        }

        Ok(SemanticCall {
            output: fallback,
            trace: SemanticRoleTrace {
                fallback_used: true,
                ..trace
            },
        })
    }
}

impl SemanticReviewer for ConfiguredSemanticReviewer {
    fn interpret_task(
        &self,
        input: TaskInterpretationRequest,
    ) -> Result<SemanticCall<TaskInterpretation>, SemanticRoleTrace> {
        let fallback = deterministic_task_interpretation(&input);
        self.request_role(
            SemanticRole::IntentInterpreter,
            serde_json::to_value(&input).unwrap_or(Value::Null),
            fallback,
            false,
        )
    }

    fn review_action(
        &self,
        input: ActionReviewRequest,
        critical: bool,
    ) -> Result<SemanticCall<SemanticAssessment>, SemanticRoleTrace> {
        let fallback = deterministic_action_assessment(&input);
        self.request_role(
            SemanticRole::SemanticRiskCritic,
            serde_json::to_value(&input).unwrap_or(Value::Null),
            fallback,
            critical,
        )
    }

    fn generate_correction(
        &self,
        input: CorrectionRequest,
        critical: bool,
    ) -> Result<SemanticCall<StructuredCorrection>, SemanticRoleTrace> {
        let fallback = deterministic_correction(&input);
        self.request_role(
            SemanticRole::StructuredCorrectionGenerator,
            serde_json::to_value(&input).unwrap_or(Value::Null),
            fallback,
            critical,
        )
    }

    fn verify_completion(
        &self,
        input: CompletionReviewRequest,
        critical: bool,
    ) -> Result<SemanticCall<CompletionAssessment>, SemanticRoleTrace> {
        let fallback = deterministic_completion_assessment(&input);
        self.request_role(
            SemanticRole::IndependentCompletionVerifier,
            serde_json::to_value(&input).unwrap_or(Value::Null),
            fallback,
            critical,
        )
    }

    fn guide_user(
        &self,
        input: UserGuidanceRequest,
    ) -> Result<SemanticCall<UserGuidance>, SemanticRoleTrace> {
        let fallback = deterministic_guidance(&input);
        self.request_role(
            SemanticRole::UserGuidanceAssistant,
            serde_json::to_value(&input).unwrap_or(Value::Null),
            fallback,
            false,
        )
    }
}

#[derive(Debug, Clone)]
pub struct FixtureSemanticReviewer {
    responses: BTreeMap<SemanticRole, String>,
}

impl FixtureSemanticReviewer {
    pub fn new(responses: BTreeMap<SemanticRole, String>) -> Self {
        Self { responses }
    }

    fn response<T>(
        &self,
        role: SemanticRole,
        fallback: T,
    ) -> Result<SemanticCall<T>, SemanticRoleTrace>
    where
        T: DeserializeOwned + TrustedSchema,
    {
        let Some(raw) = self.responses.get(&role) else {
            return Ok(SemanticCall {
                output: fallback,
                trace: SemanticRoleTrace {
                    role,
                    provider: SemanticProviderKind::Fixture,
                    provider_invoked: true,
                    fallback_used: true,
                    accepted: false,
                    error: Some("fixture response missing".to_string()),
                    estimated_tokens: 0,
                },
            });
        };
        match parse_trusted_output::<T>(raw) {
            Ok(output) => Ok(SemanticCall {
                output,
                trace: SemanticRoleTrace {
                    role,
                    provider: SemanticProviderKind::Fixture,
                    provider_invoked: true,
                    fallback_used: false,
                    accepted: true,
                    error: None,
                    estimated_tokens: estimate_tokens(raw),
                },
            }),
            Err(err) => Ok(SemanticCall {
                output: fallback,
                trace: SemanticRoleTrace {
                    role,
                    provider: SemanticProviderKind::Fixture,
                    provider_invoked: true,
                    fallback_used: true,
                    accepted: false,
                    error: Some(err.to_string()),
                    estimated_tokens: estimate_tokens(raw),
                },
            }),
        }
    }
}

impl SemanticReviewer for FixtureSemanticReviewer {
    fn interpret_task(
        &self,
        input: TaskInterpretationRequest,
    ) -> Result<SemanticCall<TaskInterpretation>, SemanticRoleTrace> {
        self.response(
            SemanticRole::IntentInterpreter,
            deterministic_task_interpretation(&input),
        )
    }

    fn review_action(
        &self,
        input: ActionReviewRequest,
        _critical: bool,
    ) -> Result<SemanticCall<SemanticAssessment>, SemanticRoleTrace> {
        self.response(
            SemanticRole::SemanticRiskCritic,
            deterministic_action_assessment(&input),
        )
    }

    fn generate_correction(
        &self,
        input: CorrectionRequest,
        _critical: bool,
    ) -> Result<SemanticCall<StructuredCorrection>, SemanticRoleTrace> {
        self.response(
            SemanticRole::StructuredCorrectionGenerator,
            deterministic_correction(&input),
        )
    }

    fn verify_completion(
        &self,
        input: CompletionReviewRequest,
        _critical: bool,
    ) -> Result<SemanticCall<CompletionAssessment>, SemanticRoleTrace> {
        self.response(
            SemanticRole::IndependentCompletionVerifier,
            deterministic_completion_assessment(&input),
        )
    }

    fn guide_user(
        &self,
        input: UserGuidanceRequest,
    ) -> Result<SemanticCall<UserGuidance>, SemanticRoleTrace> {
        self.response(
            SemanticRole::UserGuidanceAssistant,
            deterministic_guidance(&input),
        )
    }
}

#[derive(Debug, Clone)]
pub enum SemanticError {
    Budget(String),
    Provider(String),
    Malformed(String),
    Untrusted(String),
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Budget(msg) => write!(f, "semantic budget exceeded: {}", msg),
            Self::Provider(msg) => write!(f, "semantic provider error: {}", msg),
            Self::Malformed(msg) => write!(f, "malformed semantic output: {}", msg),
            Self::Untrusted(msg) => write!(f, "untrusted semantic output: {}", msg),
        }
    }
}

impl std::error::Error for SemanticError {}

pub trait TrustedSchema {
    fn validate_trusted(&self) -> Result<(), SemanticError>;
}

impl TrustedSchema for TaskInterpretation {
    fn validate_trusted(&self) -> Result<(), SemanticError> {
        validate_schema_version(self.schema_version)?;
        validate_confidence(self.confidence)?;
        validate_nonempty("normalized_objective", &self.normalized_objective)?;
        Ok(())
    }
}

impl TrustedSchema for SemanticAssessment {
    fn validate_trusted(&self) -> Result<(), SemanticError> {
        validate_schema_version(self.schema_version)?;
        validate_confidence(self.confidence)?;
        validate_nonempty("rationale", &self.rationale)?;
        Ok(())
    }
}

impl TrustedSchema for StructuredCorrection {
    fn validate_trusted(&self) -> Result<(), SemanticError> {
        validate_schema_version(self.schema_version)?;
        validate_nonempty("violation", &self.violation)?;
        validate_nonempty("reason", &self.reason)?;
        validate_nonempty("required_action", &self.required_action)?;
        Ok(())
    }
}

impl TrustedSchema for CompletionAssessment {
    fn validate_trusted(&self) -> Result<(), SemanticError> {
        validate_schema_version(self.schema_version)?;
        validate_confidence(self.confidence)?;
        if self.accept
            && (self.tests_weakened
                || self.implementation_drifted
                || self.security_reduced
                || !self.evidence_sufficient)
        {
            return Err(SemanticError::Untrusted(
                "completion accepted despite failed safety fields".to_string(),
            ));
        }
        Ok(())
    }
}

impl TrustedSchema for UserGuidance {
    fn validate_trusted(&self) -> Result<(), SemanticError> {
        validate_schema_version(self.schema_version)?;
        validate_nonempty("explanation", &self.explanation)?;
        validate_nonempty("recommendation", &self.recommendation)?;
        Ok(())
    }
}

fn validate_schema_version(version: u32) -> Result<(), SemanticError> {
    if version == 1 {
        Ok(())
    } else {
        Err(SemanticError::Untrusted(format!(
            "unsupported schema_version {}",
            version
        )))
    }
}

fn validate_confidence(confidence: f32) -> Result<(), SemanticError> {
    if (0.0..=1.0).contains(&confidence) {
        Ok(())
    } else {
        Err(SemanticError::Untrusted(format!(
            "confidence {} outside 0..1",
            confidence
        )))
    }
}

fn validate_nonempty(field: &str, value: &str) -> Result<(), SemanticError> {
    if value.trim().is_empty() {
        Err(SemanticError::Untrusted(format!("{} is empty", field)))
    } else {
        Ok(())
    }
}

fn parse_trusted_output<T>(raw: &str) -> Result<T, SemanticError>
where
    T: DeserializeOwned + TrustedSchema,
{
    let parsed: T = serde_json::from_str(raw)
        .map_err(|e| SemanticError::Malformed(format!("JSON schema parse failed: {}", e)))?;
    parsed.validate_trusted()?;
    Ok(parsed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRequest {
    pub schema_version: u32,
    pub role: SemanticRole,
    pub schema_name: String,
    pub instructions: String,
    pub input: Value,
    pub privacy_mode: PrivacyMode,
    pub model: Option<String>,
    pub response_schema: Value,
}

fn prepare_provider_request(
    role: SemanticRole,
    input: &Value,
    config: &SemanticReviewerConfig,
) -> Result<Value, SemanticError> {
    let mut safe_input = input.clone();
    if config.redact {
        safe_input = serde_json::from_str(&security::classify_payload(&safe_input).redacted)
            .map_err(|e| SemanticError::Malformed(format!("redaction parse failed: {}", e)))?;
    }
    safe_input = apply_privacy_mode(safe_input, &config.privacy_mode);

    let request = ProviderRequest {
        schema_version: 1,
        role,
        schema_name: role.schema_name().to_string(),
        instructions: role.instruction().to_string(),
        input: safe_input,
        privacy_mode: config.privacy_mode.clone(),
        model: config.model.clone(),
        response_schema: schema_for_role(role),
    };
    let value = serde_json::to_value(request).unwrap_or(Value::Null);
    let canonical = security::canonical_json(&value);
    if canonical.len() > config.max_input_bytes {
        return Err(SemanticError::Budget(format!(
            "prepared input {} bytes exceeds max {}",
            canonical.len(),
            config.max_input_bytes
        )));
    }
    enforce_token_budget(&canonical, config)?;
    Ok(value)
}

fn enforce_token_budget(input: &str, config: &SemanticReviewerConfig) -> Result<(), SemanticError> {
    let tokens = estimate_tokens(input);
    if let Some(limit) = config.token_budget {
        if tokens > limit {
            return Err(SemanticError::Budget(format!(
                "{} estimated tokens exceeds limit {}",
                tokens, limit
            )));
        }
    }
    if let Some(limit) = config.cost_budget_micro_usd {
        let estimated_cost = (tokens as u64 * config.estimated_cost_per_1k_tokens_micro_usd) / 1000;
        if estimated_cost > limit {
            return Err(SemanticError::Budget(format!(
                "{} micro-USD estimated cost exceeds limit {}",
                estimated_cost, limit
            )));
        }
    }
    Ok(())
}

fn estimate_tokens(input: &str) -> u32 {
    ((input.len() as u32).saturating_add(3) / 4).max(1)
}

fn apply_privacy_mode(value: Value, mode: &PrivacyMode) -> Value {
    match mode {
        PrivacyMode::Off | PrivacyMode::Balanced => value,
        PrivacyMode::Strict => privacy_filter_value(value),
    }
}

fn privacy_filter_value(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, child)| {
                    let lower = key.to_ascii_lowercase();
                    if matches!(
                        lower.as_str(),
                        "content"
                            | "file_contents"
                            | "before"
                            | "after"
                            | "diff"
                            | "final_diff"
                            | "payload"
                    ) {
                        (key, Value::String("[OMITTED_BY_PRIVACY_MODE]".to_string()))
                    } else {
                        (key, privacy_filter_value(child))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(privacy_filter_value).collect()),
        other => other,
    }
}

fn schema_for_role(role: SemanticRole) -> Value {
    match role {
        SemanticRole::IntentInterpreter => serde_json::json!({
            "schema_version": 1,
            "normalized_objective": "string",
            "allowed_scope": ["string"],
            "protected_scope": ["string"],
            "completion_evidence": ["string"],
            "ambiguities": ["string"],
            "risk_assumptions": ["string"],
            "questions": ["string"],
            "confidence": "0.0..1.0"
        }),
        SemanticRole::SemanticRiskCritic => serde_json::json!({
            "schema_version": 1,
            "aligned_with_task": true,
            "proportionate": true,
            "quality_problems": ["string"],
            "hidden_side_effects": ["string"],
            "confidence": "0.0..1.0",
            "recommended_decision": "allow|warn|block|escalate",
            "rationale": "string"
        }),
        SemanticRole::StructuredCorrectionGenerator => serde_json::json!({
            "schema_version": 1,
            "violation": "string",
            "reason": "string",
            "required_action": "string",
            "constraints": ["string"],
            "required_evidence": ["string"],
            "hypotheses": ["string"],
            "retry_allowed": true
        }),
        SemanticRole::IndependentCompletionVerifier => serde_json::json!({
            "schema_version": 1,
            "achieved_requested_outcome": false,
            "tests_weakened": false,
            "implementation_drifted": false,
            "security_reduced": false,
            "evidence_sufficient": false,
            "accept": false,
            "confidence": "0.0..1.0",
            "findings": ["string"],
            "required_actions": ["string"]
        }),
        SemanticRole::UserGuidanceAssistant => serde_json::json!({
            "schema_version": 1,
            "explanation": "string",
            "risks": ["string"],
            "safer_alternative": "string",
            "reversibility": "string",
            "recommendation": "string"
        }),
    }
}

fn call_local_adapter(
    request: &Value,
    config: &SemanticReviewerConfig,
) -> Result<String, SemanticError> {
    let command = config.local_command.as_ref().ok_or_else(|| {
        SemanticError::Provider("local model adapter command is not configured".to_string())
    })?;
    let mut parts = split_command_line(command);
    let program = parts
        .first()
        .cloned()
        .ok_or_else(|| SemanticError::Provider("local command is empty".to_string()))?;
    let args = parts.drain(1..).collect::<Vec<_>>();

    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| SemanticError::Provider(format!("failed to start local adapter: {}", e)))?;

    if let Some(stdin) = child.stdin.as_mut() {
        let body = serde_json::to_vec(request)
            .map_err(|e| SemanticError::Malformed(format!("request encode failed: {}", e)))?;
        stdin
            .write_all(&body)
            .map_err(|e| SemanticError::Provider(format!("local adapter stdin failed: {}", e)))?;
    }
    drop(child.stdin.take());

    let started = Instant::now();
    loop {
        if child
            .try_wait()
            .map_err(|e| SemanticError::Provider(format!("local adapter wait failed: {}", e)))?
            .is_some()
        {
            let output = child.wait_with_output().map_err(|e| {
                SemanticError::Provider(format!("local adapter output failed: {}", e))
            })?;
            if !output.status.success() {
                return Err(SemanticError::Provider(format!(
                    "local adapter exited with {}; stderr={}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
            return String::from_utf8(output.stdout).map_err(|e| {
                SemanticError::Malformed(format!("local adapter stdout was not UTF-8: {}", e))
            });
        }
        if started.elapsed() > Duration::from_millis(config.timeout_ms) {
            let _ = child.kill();
            return Err(SemanticError::Provider(format!(
                "local adapter timed out after {} ms",
                config.timeout_ms
            )));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn split_command_line(command: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for ch in command.chars() {
        match (quote, ch) {
            (Some(q), c) if c == q => quote = None,
            (None, '"' | '\'') => quote = Some(ch),
            (None, c) if c.is_whitespace() => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

fn call_cloud_adapter(
    request: &Value,
    config: &SemanticReviewerConfig,
) -> Result<String, SemanticError> {
    let endpoint = config
        .endpoint
        .as_ref()
        .ok_or_else(|| SemanticError::Provider("cloud endpoint is not configured".to_string()))?;
    let model = config
        .model
        .as_ref()
        .ok_or_else(|| SemanticError::Provider("cloud model is not configured".to_string()))?;

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(config.timeout_ms))
        .timeout_read(Duration::from_millis(config.timeout_ms))
        .build();

    let mut req = agent.post(endpoint).set("Content-Type", "application/json");
    if !is_local_endpoint(endpoint) {
        let key_env = config
            .api_key_env
            .as_deref()
            .unwrap_or("ONUS_SEMANTIC_API_KEY");
        let key = std::env::var(key_env).map_err(|_| {
            SemanticError::Provider(format!(
                "cloud API key env var {} is not configured",
                key_env
            ))
        })?;
        req = req.set("Authorization", &format!("Bearer {}", key));
    }

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are an Onus semantic reviewer. Return only strict JSON matching the supplied schema."
            },
            {
                "role": "user",
                "content": security::canonical_json(request)
            }
        ],
        "response_format": {"type": "json_object"},
        "max_tokens": config.token_budget.unwrap_or(DEFAULT_TOKEN_BUDGET)
    });

    let response = req
        .send_json(body)
        .map_err(|e| SemanticError::Provider(format!("cloud request failed: {}", e)))?;
    let value: Value = response
        .into_json()
        .map_err(|e| SemanticError::Malformed(format!("cloud JSON response failed: {}", e)))?;
    if let Some(content) = value
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
    {
        return Ok(content.to_string());
    }
    if value.get("schema_version").is_some() {
        return Ok(security::canonical_json(&value));
    }
    Err(SemanticError::Malformed(
        "cloud response did not contain choices[0].message.content or a direct schema object"
            .to_string(),
    ))
}

fn is_local_endpoint(endpoint: &str) -> bool {
    endpoint.contains("localhost") || endpoint.contains("127.0.0.1") || endpoint.contains("[::1]")
}

pub fn is_critical_action(request: &ActionRequest) -> bool {
    let payload = security::canonical_json(&request.action.payload).to_ascii_lowercase();
    match request.action.action_type {
        crate::ActionType::FileDelete
        | crate::ActionType::DbMutation
        | crate::ActionType::ApiCall
        | crate::ActionType::Network
        | crate::ActionType::MCP => true,
        crate::ActionType::Shell => contains_any(
            &payload,
            &[
                "sudo",
                "rm -rf",
                "drop table",
                "delete from",
                "curl",
                "wget",
                "chmod",
                "chown",
            ],
        ),
        crate::ActionType::FileWrite => contains_any(
            &payload,
            &[".env", "secret", "token", "password", "credential"],
        ),
        crate::ActionType::Git | crate::ActionType::FileRead => false,
    }
}

fn deterministic_task_interpretation(input: &TaskInterpretationRequest) -> TaskInterpretation {
    TaskInterpretation {
        schema_version: 1,
        normalized_objective: input
            .original_prompt
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
        allowed_scope: Vec::new(),
        protected_scope: Vec::new(),
        completion_evidence: Vec::new(),
        ambiguities: Vec::new(),
        risk_assumptions: vec!["deterministic fallback; no model was used".to_string()],
        questions: Vec::new(),
        confidence: 0.0,
    }
}

fn deterministic_action_assessment(input: &ActionReviewRequest) -> SemanticAssessment {
    SemanticAssessment {
        schema_version: 1,
        aligned_with_task: true,
        proportionate: true,
        quality_problems: Vec::new(),
        hidden_side_effects: Vec::new(),
        confidence: 0.0,
        recommended_decision: input.deterministic_verdict.clone(),
        rationale: "deterministic fallback; no semantic approval granted".to_string(),
    }
}

fn deterministic_correction(input: &CorrectionRequest) -> StructuredCorrection {
    StructuredCorrection {
        schema_version: 1,
        violation: input
            .rule_id
            .clone()
            .unwrap_or_else(|| "deterministic_policy".to_string()),
        reason: input.correction.clone().unwrap_or_else(|| {
            "The deterministic policy did not provide an additional correction.".to_string()
        }),
        required_action:
            "Revise the action so it satisfies deterministic policy and the task contract."
                .to_string(),
        constraints: {
            let mut constraints = vec!["Do not weaken security requirements.".to_string()];
            constraints.extend(
                input
                    .memory_context
                    .iter()
                    .take(3)
                    .map(|item| format!("Respect scoped memory: {}", item)),
            );
            constraints
        },
        required_evidence: vec!["Re-run the relevant Onus evaluation.".to_string()],
        hypotheses: Vec::new(),
        retry_allowed: true,
    }
}

fn deterministic_completion_assessment(input: &CompletionReviewRequest) -> CompletionAssessment {
    let evidence_sufficient = !input.evidence.is_empty();
    CompletionAssessment {
        schema_version: 1,
        achieved_requested_outcome: false,
        tests_weakened: false,
        implementation_drifted: false,
        security_reduced: false,
        evidence_sufficient,
        accept: false,
        confidence: 0.0,
        findings: {
            let mut findings = vec!["deterministic fallback requires human review".to_string()];
            findings.extend(
                input
                    .memory_context
                    .iter()
                    .take(3)
                    .map(|item| format!("Relevant memory considered: {}", item)),
            );
            findings
        },
        required_actions: vec!["Provide independent verification evidence.".to_string()],
    }
}

fn deterministic_guidance(input: &UserGuidanceRequest) -> UserGuidance {
    UserGuidance {
        schema_version: 1,
        explanation: format!(
            "Onus returned a deterministic {:?} verdict for this action.",
            input.deterministic_verdict
        ),
        risks: input.risk_context.clone(),
        safer_alternative: "Narrow the action and rerun deterministic evaluation.".to_string(),
        reversibility: input
            .reversibility
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        recommendation: "Follow deterministic policy and request human approval when required."
            .to_string(),
    }
}

pub fn semantic_review_summary(trace: &SemanticRoleTrace) -> String {
    if !trace.provider_invoked && matches!(trace.provider, SemanticProviderKind::Disabled) {
        return "provider_disabled; no LLM provider was invoked".to_string();
    }
    if !trace.provider_invoked && matches!(trace.provider, SemanticProviderKind::Deterministic) {
        return "deterministic_rules_only; no LLM provider was invoked".to_string();
    }
    if trace.accepted {
        format!(
            "semantic_provider:{}; role:{}; schema_accepted",
            trace.provider, trace.role
        )
    } else if trace.fallback_used {
        format!(
            "semantic_provider:{}; role:{}; deterministic_fallback_used; error={}",
            trace.provider,
            trace.role,
            trace.error.as_deref().unwrap_or("unknown")
        )
    } else {
        format!(
            "semantic_provider:{}; role:{}; failed_closed; error={}",
            trace.provider,
            trace.role,
            trace.error.as_deref().unwrap_or("unknown")
        )
    }
}

fn parse_provider(value: &str) -> SemanticProviderKind {
    match value.to_ascii_lowercase().replace('-', "_").as_str() {
        "local" | "local_model" => SemanticProviderKind::Local,
        "cloud" | "remote" | "openai" | "openai_compatible" => SemanticProviderKind::Cloud,
        "deterministic" => SemanticProviderKind::Deterministic,
        _ => SemanticProviderKind::Disabled,
    }
}

fn parse_privacy_mode(value: &str) -> PrivacyMode {
    match value.to_ascii_lowercase().replace('-', "_").as_str() {
        "off" | "none" => PrivacyMode::Off,
        "balanced" => PrivacyMode::Balanced,
        _ => PrivacyMode::Strict,
    }
}

fn parse_fallback(value: &str) -> SemanticFallbackPolicy {
    match value.to_ascii_lowercase().replace('-', "_").as_str() {
        "fail_closed" | "failclosed" => SemanticFallbackPolicy::FailClosed,
        _ => SemanticFallbackPolicy::Deterministic,
    }
}

fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn env_u64(name: &str) -> Option<u64> {
    std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok())
}

fn contains_any(input: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| input.contains(needle))
}

pub fn evidence_from_semantic(label: &str) -> RequiredEvidence {
    RequiredEvidence {
        id: format!(
            "semantic_{}",
            label
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect::<String>()
                .to_ascii_lowercase()
        ),
        description: label.to_string(),
        kind: "semantic".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role_responses() -> BTreeMap<SemanticRole, String> {
        BTreeMap::from([
            (
                SemanticRole::IntentInterpreter,
                serde_json::json!({
                    "schema_version": 1,
                    "normalized_objective": "Fix the login bug locally.",
                    "allowed_scope": ["src/auth/**"],
                    "protected_scope": [".env"],
                    "completion_evidence": ["pytest auth tests"],
                    "ambiguities": [],
                    "risk_assumptions": ["local only"],
                    "questions": [],
                    "confidence": 0.86
                })
                .to_string(),
            ),
            (
                SemanticRole::SemanticRiskCritic,
                serde_json::json!({
                    "schema_version": 1,
                    "aligned_with_task": false,
                    "proportionate": false,
                    "quality_problems": ["Deletes unrelated files"],
                    "hidden_side_effects": ["May remove evidence"],
                    "confidence": 0.9,
                    "recommended_decision": "block",
                    "rationale": "Action is outside the task scope."
                })
                .to_string(),
            ),
            (
                SemanticRole::StructuredCorrectionGenerator,
                serde_json::json!({
                    "schema_version": 1,
                    "violation": "acceptance_test_deleted",
                    "reason": "The deleted test is required evidence.",
                    "required_action": "Restore the test.",
                    "constraints": ["Do not skip tests"],
                    "required_evidence": ["Original test passes"],
                    "hypotheses": ["The failing test may indicate a real bug"],
                    "retry_allowed": true
                })
                .to_string(),
            ),
            (
                SemanticRole::IndependentCompletionVerifier,
                serde_json::json!({
                    "schema_version": 1,
                    "achieved_requested_outcome": false,
                    "tests_weakened": false,
                    "implementation_drifted": true,
                    "security_reduced": false,
                    "evidence_sufficient": false,
                    "accept": false,
                    "confidence": 0.75,
                    "findings": ["Diff touched unrelated module"],
                    "required_actions": ["Revert unrelated changes"]
                })
                .to_string(),
            ),
            (
                SemanticRole::UserGuidanceAssistant,
                serde_json::json!({
                    "schema_version": 1,
                    "explanation": "The action is risky.",
                    "risks": ["Deletes files"],
                    "safer_alternative": "Limit the path.",
                    "reversibility": "partial",
                    "recommendation": "Block until narrowed."
                })
                .to_string(),
            ),
        ])
    }

    #[test]
    fn fixture_provider_supports_all_roles() {
        let fixture = FixtureSemanticReviewer::new(role_responses());
        let task = fixture
            .interpret_task(TaskInterpretationRequest {
                original_prompt: "Fix auth".to_string(),
                repository_metadata: vec![],
                memory_context: vec!["project:auth architecture".to_string()],
                existing_policy: vec![],
                current_environment: "local".to_string(),
            })
            .unwrap();
        assert!(task.trace.accepted);
        assert_eq!(
            task.output.normalized_objective,
            "Fix the login bug locally."
        );

        let action = Action {
            action_type: crate::ActionType::FileDelete,
            tool: "Delete".to_string(),
            payload: serde_json::json!({"path": "tests/auth_test.py"}),
        };
        let review = fixture
            .review_action(
                ActionReviewRequest {
                    task_contract_hash: None,
                    action: action.clone(),
                    relevant_diff: None,
                    previous_actions: vec![],
                    repository_architecture: vec![],
                    deterministic_verdict: Verdict::Allow,
                    policy_findings: vec![],
                },
                true,
            )
            .unwrap();
        assert_eq!(review.output.recommended_decision, Verdict::Block);

        let correction = fixture
            .generate_correction(
                CorrectionRequest {
                    deterministic_verdict: Verdict::Block,
                    rule_id: Some("TEST".to_string()),
                    correction: Some("blocked".to_string()),
                    memory_context: vec!["incident:test deletion blocked before".to_string()],
                    task_contract: None,
                    action: Some(action),
                },
                true,
            )
            .unwrap();
        assert_eq!(correction.output.violation, "acceptance_test_deleted");

        let contract = TaskContract {
            schema_version: 1,
            session_id: "s".to_string(),
            original_prompt: "Fix auth".to_string(),
            normalized_objective: "Fix auth".to_string(),
            allowed_paths: vec!["src/**".to_string()],
            allowed_resources: vec![],
            protected_paths: vec![],
            protected_resources: vec![],
            required_evidence: vec![],
            forbidden_actions: vec![],
            approval_required_actions: vec![],
            change_budget: Default::default(),
            environment_identity: "test".to_string(),
            policy_version: "test".to_string(),
            canonical_hash: String::new(),
        }
        .finalized();
        let completion = fixture
            .verify_completion(
                CompletionReviewRequest {
                    original_task: "Fix auth".to_string(),
                    task_contract: contract,
                    final_diff: String::new(),
                    action_trace: vec![],
                    denied_and_corrected_actions: vec![],
                    evidence: vec![],
                    policy_exceptions: vec![],
                    memory_context: vec!["project:auth tests required".to_string()],
                },
                true,
            )
            .unwrap();
        assert!(!completion.output.accept);

        let guidance = fixture
            .guide_user(UserGuidanceRequest {
                action_summary: "delete tests".to_string(),
                deterministic_verdict: Verdict::Block,
                risk_context: vec!["Deletes evidence".to_string()],
                reversibility: Some("partial".to_string()),
            })
            .unwrap();
        assert_eq!(guidance.output.recommendation, "Block until narrowed.");
    }

    #[test]
    fn malformed_fixture_output_is_rejected_and_falls_back() {
        let fixture = FixtureSemanticReviewer::new(BTreeMap::from([(
            SemanticRole::IntentInterpreter,
            "{\"schema_version\":1,\"unexpected\":true}".to_string(),
        )]));
        let result = fixture
            .interpret_task(TaskInterpretationRequest {
                original_prompt: "Fix auth".to_string(),
                repository_metadata: vec![],
                memory_context: vec![],
                existing_policy: vec![],
                current_environment: "local".to_string(),
            })
            .unwrap();
        assert!(!result.trace.accepted);
        assert!(result.trace.fallback_used);
        assert!(result.trace.error.unwrap().contains("malformed"));
        assert_eq!(result.output.normalized_objective, "Fix auth");
    }

    #[test]
    fn privacy_redaction_happens_before_provider_request() {
        let config = SemanticReviewerConfig {
            provider: SemanticProviderKind::Local,
            privacy_mode: PrivacyMode::Strict,
            ..SemanticReviewerConfig::default()
        };
        let prepared = prepare_provider_request(
            SemanticRole::SemanticRiskCritic,
            &serde_json::json!({
                "action": {
                    "payload": {
                        "content": "AWS_SECRET_ACCESS_KEY=\"abc123\""
                    }
                }
            }),
            &config,
        )
        .unwrap();
        let canonical = security::canonical_json(&prepared);
        assert!(!canonical.contains("abc123"));
        assert!(canonical.contains("[OMITTED_BY_PRIVACY_MODE]"));
    }

    #[test]
    fn token_budget_prevents_provider_call() {
        let reviewer = ConfiguredSemanticReviewer::new(SemanticReviewerConfig {
            provider: SemanticProviderKind::Local,
            local_command: Some("should-not-run".to_string()),
            token_budget: Some(1),
            fallback_policy: SemanticFallbackPolicy::Deterministic,
            ..SemanticReviewerConfig::default()
        });
        let result = reviewer
            .interpret_task(TaskInterpretationRequest {
                original_prompt: "Fix ".repeat(100),
                repository_metadata: vec![],
                memory_context: vec![],
                existing_policy: vec![],
                current_environment: "local".to_string(),
            })
            .unwrap();
        assert!(result.trace.fallback_used);
        assert!(!result.trace.provider_invoked);
        assert!(result.trace.error.unwrap().contains("budget"));
    }

    #[test]
    fn completion_verifier_uses_memory_context_in_fallback_findings() {
        let contract = TaskContract {
            schema_version: 1,
            session_id: "s".to_string(),
            original_prompt: "Fix auth".to_string(),
            normalized_objective: "Fix auth".to_string(),
            allowed_paths: vec!["src/**".to_string()],
            allowed_resources: vec![],
            protected_paths: vec![],
            protected_resources: vec![],
            required_evidence: vec![],
            forbidden_actions: vec![],
            approval_required_actions: vec![],
            change_budget: Default::default(),
            environment_identity: "test".to_string(),
            policy_version: "test".to_string(),
            canonical_hash: String::new(),
        }
        .finalized();
        let reviewer = ConfiguredSemanticReviewer::new(SemanticReviewerConfig::default());
        let result = reviewer
            .verify_completion(
                CompletionReviewRequest {
                    original_task: "Fix auth".to_string(),
                    task_contract: contract,
                    final_diff: String::new(),
                    action_trace: vec![],
                    denied_and_corrected_actions: vec![],
                    evidence: vec![],
                    policy_exceptions: vec![],
                    memory_context: vec!["incident: previous auth test deletion".to_string()],
                },
                true,
            )
            .unwrap();
        assert!(result
            .output
            .findings
            .iter()
            .any(|item| item.contains("previous auth test deletion")));
    }
}
