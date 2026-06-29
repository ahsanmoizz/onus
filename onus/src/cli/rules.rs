//! `onus rules` — manage safety rules.

use clap::{Args, Subcommand};

#[derive(Args)]
pub struct RulesArgs {
    #[command(subcommand)]
    pub command: RulesCommand,
}

#[derive(Subcommand)]
pub enum RulesCommand {
    /// Initialize default rules in config directory
    Init,
    /// List all loaded rules
    List,
    /// Test a rule against a sample action payload
    Test {
        /// Rule ID to test
        rule_id: String,
        /// Sample payload string
        payload: String,
    },
    /// Open the rules file in your editor ($EDITOR)
    Edit,
    /// Fetch latest community rules from GitHub
    Pull,
    /// Install a signed policy file into the rules directory
    Install {
        /// Path to the signed policy file (JSON)
        path: String,
    },
    /// Verify the signature of a signed policy file
    Verify {
        /// Path to the signed policy file (JSON)
        path: String,
    },
    /// Sign a policy file with the local signing key
    Sign {
        /// Path to the policy file (JSON) to sign
        path: String,
    },
    /// Revoke a previously installed policy by removing it from the rules directory
    Revoke {
        /// Policy name (filename without extension) to revoke
        name: String,
    },
    /// Generate Ed25519 signing keys for policy signing
    GenerateKeys,
}

pub fn run(args: RulesArgs) -> anyhow::Result<()> {
    let rules_dir = crate::config_dir().join("rules");
    let rules_path = rules_dir.join("default.toml");

    match args.command {
        RulesCommand::Init => {
            std::fs::create_dir_all(&rules_dir)?;

            // Copy embedded default rules.
            let default_rules = include_str!("../../rules/default.toml");

            if rules_path.exists() {
                anyhow::bail!(
                    "Rules file already exists at {}. Use 'onus rules edit' to modify.",
                    rules_path.display()
                );
            }

            std::fs::write(&rules_path, default_rules)?;
            println!("Default rules written to {}", rules_path.display());
            let count = default_rules.matches("[[rule]]").count();
            println!(
                "{} safety rules installed. Use 'onus rules edit' to customize.",
                count
            );
        }

        RulesCommand::List => {
            if !rules_path.exists() {
                anyhow::bail!(
                    "No rules file found. Run 'onus rules init' to create default rules."
                );
            }

            let rules = crate::policy::rule::load_rules(&rules_path)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            println!(
                "{:14}  {:5}  {:12}  {:10}  NAME",
                "ID", "TIER", "TYPE", "DECISION"
            );
            println!("{}", "─".repeat(100));

            let mut tier1 = vec![];
            let mut tier2 = vec![];

            for rule in &rules {
                if rule.tier == 1 {
                    tier1.push(rule);
                } else {
                    tier2.push(rule);
                }
            }

            for rules in [&tier1, &tier2] {
                for rule in rules {
                    let decision_styled = match rule.decision {
                        crate::Verdict::Allow => "\x1b[32mallow\x1b[0m".to_string(),
                        crate::Verdict::Warn => "\x1b[33mwarn\x1b[0m".to_string(),
                        crate::Verdict::Block => "\x1b[31mblock\x1b[0m".to_string(),
                        crate::Verdict::Escalate => "\x1b[35mescalate\x1b[0m".to_string(),
                    };

                    println!(
                        "{:14}  {:>2}    {:12}  {}  {}",
                        rule.id,
                        rule.tier,
                        rule.action_type.to_string(),
                        decision_styled,
                        rule.name,
                    );
                }
                if !tier2.is_empty() && rules.is_empty() {
                    // tier separator already handled
                }
            }
        }

        RulesCommand::Test { rule_id, payload } => {
            if !rules_path.exists() {
                anyhow::bail!(
                    "No rules file found. Run 'onus rules init' to create default rules."
                );
            }

            let rules = crate::policy::rule::load_rules(&rules_path)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let rule = rules.iter().find(|r| r.id == rule_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "Rule '{}' not found. Use 'onus rules list' to see all rules.",
                    rule_id
                )
            })?;

            println!("Testing rule: {} ({})", rule.id, rule.name);
            println!("Pattern:      {}", rule.pattern);
            println!("Payload:      {}", payload);

            if rule.pattern.is_match(&payload) {
                let allowlisted = rule.is_allowlisted(&payload);
                if allowlisted {
                    println!("\nResult:       \x1b[32mMATCHED but ALLOWLISTED\x1b[0m — action would be ALLOWED");
                } else {
                    let decision = match rule.decision {
                        crate::Verdict::Block => "\x1b[31mBLOCK\x1b[0m",
                        crate::Verdict::Warn => "\x1b[33mWARN\x1b[0m",
                        crate::Verdict::Escalate => "\x1b[35mESCALATE\x1b[0m",
                        crate::Verdict::Allow => "ALLOW",
                    };
                    println!(
                        "\nResult:       \x1b[31mMATCHED\x1b[0m MATCHED — action would be {}",
                        decision
                    );
                    if !rule.correction.is_empty() {
                        println!("Correction:   {}", rule.correction);
                    }
                }
            } else {
                println!("\nResult:       No match — action would be ALLOWED");
            }
        }

        RulesCommand::Edit => {
            if !rules_path.exists() {
                // Initialize if needed.
                std::fs::create_dir_all(&rules_dir)?;
                let default_rules = include_str!("../../rules/default.toml");
                std::fs::write(&rules_path, default_rules)?;
            }

            let editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| {
                    #[cfg(windows)]
                    {
                        "notepad.exe".into()
                    }
                    #[cfg(not(windows))]
                    {
                        "vi".into()
                    }
                });

            let status = std::process::Command::new(&editor)
                .arg(&rules_path)
                .status()
                .map_err(|e| anyhow::anyhow!("Failed to open editor '{}': {}", editor, e))?;

            if !status.success() {
                anyhow::bail!("Editor exited with error");
            }

            // Validate the modified rules.
            match crate::policy::rule::load_rules(&rules_path) {
                Ok(rules) => {
                    println!("Rules updated successfully ({} rules loaded).", rules.len());
                }
                Err(e) => {
                    eprintln!("\x1b[31mWarning:\x1b[0m Rules file has errors: {}", e);
                    eprintln!("Fix the errors and re-save to restore Onus protection.");
                }
            }
        }

        RulesCommand::Pull => {
            let url = "https://raw.githubusercontent.com/ahsanmoizz/onus/main/onus/rules/default.toml";
            println!("Fetching latest community rules from {}", url);

            let body =
                download_url(url).map_err(|e| anyhow::anyhow!("Failed to fetch rules: {}", e))?;

            // Validate that the fetched content is valid TOML with rules.
            crate::policy::rule::load_rules_from_str(&body)
                .map_err(|e| anyhow::anyhow!("Fetched rules are invalid: {}", e))?;

            // Backup existing rules.
            if rules_path.exists() {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let backup_path = rules_dir.join(format!("default.toml.{}.bak", timestamp));
                std::fs::copy(&rules_path, &backup_path)?;
                println!("Backed up previous rules to {}", backup_path.display());
            }

            std::fs::create_dir_all(&rules_dir)?;
            std::fs::write(&rules_path, &body)?;

            let count = body.matches("[[rule]]").count();
            println!("Fetched and installed {} rules from community.", count);
            println!("File: {}", rules_path.display());
        }

        RulesCommand::Install { ref path } => {
            let data = std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read policy file: {}", e))?;
            let signed: crate::policy::SignedPolicy = serde_json::from_str(&data)
                .map_err(|e| anyhow::anyhow!("Invalid signed policy JSON: {}", e))?;

            // Verify signature if present
            if let Some(ref sig) = signed.signature {
                if !sig.is_empty() {
                    signed.verify()
                        .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))?;
                    println!("Signature verified (signer: {})", &sig.signer[..16.min(sig.signer.len())]);
                } else {
                    println!("Warning: policy has empty signature field, installing without verification");
                }
            } else {
                println!("Warning: policy is not signed, installing without verification");
            }

            let filename = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("installed");
            let dest = rules_dir.join(format!("{}.json", filename));
            std::fs::create_dir_all(&rules_dir)?;

            // Backup existing if present
            if dest.exists() {
                let backup = rules_dir.join(format!("{}.json.bak", filename));
                std::fs::copy(&dest, &backup)?;
                println!("Backed up existing policy to {}", backup.display());
            }

            std::fs::write(&dest, &data)?;
            println!("Installed policy to {}", dest.display());
        }

        RulesCommand::Verify { ref path } => {
            let data = std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read policy file: {}", e))?;
            let signed: crate::policy::SignedPolicy = serde_json::from_str(&data)
                .map_err(|e| anyhow::anyhow!("Invalid signed policy JSON: {}", e))?;

            match signed.signature {
                Some(ref sig) if !sig.is_empty() => {
                    println!("Algorithm: {}", sig.algorithm);
                    println!("Signer:    {}", sig.signer);
                    match signed.verify() {
                        Ok(_) => println!("Verdict:   VALID"),
                        Err(e) => println!("Verdict:   INVALID — {}", e),
                    }
                }
                _ => {
                    println!("Policy is not signed.");
                }
            }
        }

        RulesCommand::Sign { ref path } => {
            if !crate::policy::signing::has_signing_keys() {
                anyhow::bail!(
                    "No signing keys found. Run 'onus rules generate-keys' first."
                );
            }

            let data = std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read policy file: {}", e))?;
            let mut signed: crate::policy::SignedPolicy = serde_json::from_str(&data)
                .map_err(|e| anyhow::anyhow!("Invalid policy JSON: {}", e))?;

            let key_pair = crate::policy::signing::load_private_key()?;
            let sig = crate::policy::signing::sign_policy(&signed, &key_pair)?;

            signed.signature = Some(sig);
            let out_path = format!("{}.signed.json", path.trim_end_matches(".json"));
            let out_data = serde_json::to_string_pretty(&signed)?;
            std::fs::write(&out_path, &out_data)?;
            println!("Signed policy written to {}", out_path);
        }

        RulesCommand::Revoke { ref name } => {
            let candidates = [
                rules_dir.join(format!("{}.json", name)),
                rules_dir.join(format!("{}.toml", name)),
            ];
            let mut found = false;
            for candidate in &candidates {
                if candidate.exists() {
                    std::fs::remove_file(candidate)?;
                    println!("Revoked (removed): {}", candidate.display());
                    found = true;
                }
            }
            if !found {
                anyhow::bail!("No policy named '{}' found in {}", name, rules_dir.display());
            }
        }

        RulesCommand::GenerateKeys => {
            if crate::policy::signing::has_signing_keys() {
                anyhow::bail!(
                    "Signing keys already exist at {}. Delete them first to regenerate.",
                    crate::policy::signing::signing_dir().display()
                );
            }
            crate::policy::signing::generate_keys()?;
            println!("Ed25519 signing keys generated:");
            println!("  Private key: {}", crate::policy::signing::private_key_path().display());
            println!("  Public key:  {}", crate::policy::signing::public_key_path().display());
            let pub_hex = crate::policy::signing::load_public_key_hex()?;
            println!("  Public hex:  {}", pub_hex);
        }
    }

    Ok(())
}

/// Download a URL using whatever is available on the system (curl, wget, or PowerShell).
fn download_url(url: &str) -> anyhow::Result<String> {
    // Try curl first.
    if let Ok(output) = std::process::Command::new("curl")
        .args(["-fsSL", url])
        .output()
    {
        if output.status.success() {
            return Ok(String::from_utf8(output.stdout)?);
        }
    }

    // Try wget.
    if let Ok(output) = std::process::Command::new("wget")
        .args(["-qO-", url])
        .output()
    {
        if output.status.success() {
            return Ok(String::from_utf8(output.stdout)?);
        }
    }

    // Try PowerShell on Windows.
    #[cfg(windows)]
    {
        if let Ok(output) = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Invoke-WebRequest -Uri '{}').Content", url),
            ])
            .output()
        {
            if output.status.success() {
                return Ok(String::from_utf8(output.stdout)?);
            }
        }
    }

    anyhow::bail!(
        "No download tool found (tried curl, wget, powershell). Install curl and try again."
    )
}
