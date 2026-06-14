//! Policy rule definition and TOML parsing.

use crate::{ActionType, Reversibility, Verdict};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A single policy rule.
#[derive(Debug, Clone, Deserialize)]
pub struct RuleConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub tier: u8,
    pub action_type: String,
    pub pattern: String,
    pub decision: String,
    #[serde(default)]
    pub correction: String,
    #[serde(default)]
    pub allowlist: Vec<String>,
    /// Reversibility classification (defaults to irreversible for safety)
    #[serde(default = "default_reversibility")]
    pub reversibility: String,
}

fn default_reversibility() -> String {
    "irreversible".to_string()
}

/// A compiled rule, ready for evaluation.
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub tier: u8,
    pub action_type: ActionType,
    pub pattern: Regex,
    pub decision: Verdict,
    pub correction: String,
    pub allowlist: Vec<Regex>,
    pub reversibility: Reversibility,
}

impl Rule {
    /// Compile a RuleConfig into an executable Rule.
    pub fn compile(config: &RuleConfig) -> Result<Self, String> {
        let pattern = Regex::new(&config.pattern)
            .map_err(|e| format!("Rule {}: invalid pattern '{}': {}", config.id, config.pattern, e))?;

        let action_type = match config.action_type.as_str() {
            "shell" => ActionType::Shell,
            "file_write" => ActionType::FileWrite,
            "file_delete" => ActionType::FileDelete,
            "file_read" => ActionType::FileRead,
            "git" => ActionType::Git,
            "api_call" => ActionType::ApiCall,
            "db_mutation" => ActionType::DbMutation,
            "network" => ActionType::Network,
            "mcp" => ActionType::MCP,
            other => return Err(format!("Rule {}: unknown action_type '{}'", config.id, other)),
        };

        let decision = match config.decision.as_str() {
            "allow" => Verdict::Allow,
            "warn" => Verdict::Warn,
            "block" => Verdict::Block,
            "escalate" => Verdict::Escalate,
            other => return Err(format!("Rule {}: unknown decision '{}'", config.id, other)),
        };

        let reversibility = match config.reversibility.as_str() {
            "reversible" => Reversibility::Reversible,
            "compensable" => Reversibility::Compensable,
            "irreversible" => Reversibility::Irreversible,
            other => return Err(format!("Rule {}: unknown reversibility '{}'", config.id, other)),
        };

        let allowlist: Vec<Regex> = config
            .allowlist
            .iter()
            .filter_map(|a| {
                // Allowlist entries are glob-like — convert to regex
                let re_str = glob_to_regex(a);
                Regex::new(&re_str).ok()
            })
            .collect();

        Ok(Rule {
            id: config.id.clone(),
            name: config.name.clone(),
            tier: config.tier,
            action_type,
            pattern,
            decision,
            correction: config.correction.clone(),
            allowlist,
            reversibility,
        })
    }

    /// Check if a matched action is allowlisted.
    pub fn is_allowlisted(&self, payload: &str) -> bool {
        self.allowlist.iter().any(|re| re.is_match(payload))
    }
}

/// A collection of compiled rules.
#[derive(Debug, Clone, Deserialize)]
pub struct RuleSetConfig {
    pub rule: Vec<RuleConfig>,
}

/// Top-level TOML structure.
#[derive(Debug, Clone, Deserialize)]
struct RulesFile {
    rule: Vec<RuleConfig>,
}

/// Load and compile rules from a TOML file.
pub fn load_rules(path: &Path) -> Result<Vec<Rule>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read rules file {}: {}", path.display(), e))?;

    let rules_file: RulesFile = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse rules TOML: {}", e))?;

    let mut rules = Vec::new();
    for config in &rules_file.rule {
        match Rule::compile(config) {
            Ok(rule) => rules.push(rule),
            Err(e) => {
                log::warn!("Skipping rule {}: {}", config.id, e);
            }
        }
    }

    if rules.is_empty() {
        return Err("No valid rules loaded".into());
    }

    log::info!("Loaded {} rules from {}", rules.len(), path.display());
    Ok(rules)
}

/// Load and compile rules from a TOML string (no file path).
pub fn load_rules_from_str(toml_content: &str) -> Result<Vec<Rule>, String> {
    let rules_file: RulesFile = toml::from_str(toml_content)
        .map_err(|e| format!("Failed to parse rules TOML: {}", e))?;

    let mut rules = Vec::new();
    for config in &rules_file.rule {
        match Rule::compile(config) {
            Ok(rule) => rules.push(rule),
            Err(e) => {
                log::warn!("Skipping rule {}: {}", config.id, e);
            }
        }
    }

    if rules.is_empty() {
        return Err("No valid rules loaded".into());
    }

    log::info!("Loaded {} rules from string", rules.len());
    Ok(rules)
}

/// Convert a simple glob pattern to a regex.
/// Supports: * as wildcard, paths.
fn glob_to_regex(glob: &str) -> String {
    let escaped = regex::escape(glob);
    // Replace escaped \* with .* (reluctant match)
    let re = escaped.replace(r"\*", ".*?");
    format!(".*{}.*", re)
}

/// Summary of a rule for IPC transport (no compiled regex).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub verdict: String,
    pub tier: u8,
    pub enabled: bool,
    pub reversibility: String,
}

impl From<&Rule> for RuleSummary {
    fn from(r: &Rule) -> Self {
        RuleSummary {
            id: r.id.clone(),
            name: r.name.clone(),
            description: r.correction.clone(),
            verdict: format!("{:?}", r.decision),
            tier: r.tier,
            enabled: true,
            reversibility: format!("{:?}", r.reversibility),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_safety_001() {
        let config = RuleConfig {
            id: "SAFETY_001".into(),
            name: "destructive-filesystem".into(),
            description: "Blocks rm -rf".into(),
            tier: 1,
            action_type: "shell".into(),
            pattern: r"rm\s+-rf".into(),
            decision: "block".into(),
            correction: "Destructive command blocked".into(),
            allowlist: vec!["node_modules".into()],
            reversibility: "irreversible".into(),
        };

        let rule = Rule::compile(&config).unwrap();
        assert_eq!(rule.id, "SAFETY_001");
        assert_eq!(rule.decision, Verdict::Block);
        assert!(rule.pattern.is_match("rm -rf /tmp/test"));
        assert!(!rule.pattern.is_match("ls -la"));

        // Allowlist check
        assert!(rule.is_allowlisted("rm -rf node_modules"));
        assert!(!rule.is_allowlisted("rm -rf src"));
    }

    #[test]
    fn test_glob_to_regex() {
        let re = glob_to_regex("node_modules");
        assert!(regex::Regex::new(&re).unwrap().is_match("/home/user/node_modules/package"));
    }
}
