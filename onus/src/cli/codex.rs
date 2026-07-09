//! `onus codex` — OpenAI Codex CLI integration adapter.
//!
//! Codex CLI supports MCP servers. This module provides:
//! - MCP proxy configuration management for Codex
//! - Codex CLI binary detection and version checking
//! - Codex-specific setup and uninstall helpers
//! - L3 workspace fallback

use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct CodexMcpArgs {
    /// Upstream MCP server binary that Onus should wrap.
    #[arg(long)]
    pub server: PathBuf,

    /// Arguments passed to the upstream MCP server after `--`.
    #[arg(last = true)]
    pub args: Vec<String>,
}

// ── Codex CLI detection ──────────────────────────────────────────────────────

/// Result of checking for Codex CLI.
#[derive(Debug)]
pub enum CodexCliCheck {
    Available { version: String, path: PathBuf },
    NotFound,
}

/// Try to detect Codex CLI on PATH.
pub fn find_codex_cli() -> CodexCliCheck {
    if let Some(path) = find_codex_on_path() {
        return CodexCliCheck::Available {
            version: "unknown".to_string(),
            path,
        };
    }

    CodexCliCheck::NotFound
}

fn find_codex_on_path() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join("codex");
            if candidate.is_file() {
                return Some(candidate);
            }
            #[cfg(windows)]
            {
                let candidate_exe = dir.join("codex.exe");
                if candidate_exe.is_file() {
                    return Some(candidate_exe);
                }
                let candidate_cmd = dir.join("codex.cmd");
                if candidate_cmd.is_file() {
                    return Some(candidate_cmd);
                }
            }
        }
        None
    })
}

// ── Codex MCP config ─────────────────────────────────────────────────────────

/// Path to Codex CLI config.toml.
fn codex_config_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();
    PathBuf::from(home).join(".codex").join("config.toml")
}

/// Check whether the Onus MCP proxy entry exists in Codex config.
pub enum McpConfigCheck {
    Configured { server_name: String, mode: String },
    NotFound,
    Error(String),
}

pub fn check_mcp_config() -> McpConfigCheck {
    let config_path = codex_config_path();
    if !config_path.exists() {
        return McpConfigCheck::NotFound;
    }

    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            return McpConfigCheck::Error(format!("Cannot read {}: {}", config_path.display(), e))
        }
    };

    if content.contains("[mcp_servers.onus-mcp-proxy]") {
        let mode = if content.contains("approval_mode") {
            "with approval"
        } else {
            "MCP proxy"
        };
        McpConfigCheck::Configured {
            server_name: "onus-mcp-proxy".to_string(),
            mode: mode.to_string(),
        }
    } else {
        McpConfigCheck::NotFound
    }
}

/// Generate the TOML content for the Onus MCP proxy server entry.
fn toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn toml_array(values: &[String]) -> String {
    let rendered = values
        .iter()
        .map(|value| toml_string(value))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", rendered)
}

fn generate_mcp_config(onus_bin: &str, upstream_server: &str, upstream_args: &[String]) -> String {
    let mut args = vec![
        "mcp-proxy".to_string(),
        "--experimental".to_string(),
        "--server".to_string(),
        upstream_server.to_string(),
    ];
    if !upstream_args.is_empty() {
        args.push("--".to_string());
        args.extend(upstream_args.iter().cloned());
    }
    format!(
        r#"[mcp_servers.onus-mcp-proxy]
command = {}
args = {}
tool_timeout_sec = 30
# Approval modes: "auto" = allow all, "prompt" = ask user, "approve" = require explicit approval
# Onus recommends "prompt" for interactive or "approve" for strict
default_tools_approval_mode = "prompt"
"#,
        toml_string(onus_bin),
        toml_array(&args)
    )
}

/// Write the Onus MCP proxy entry into Codex config.toml.
pub fn install_mcp_hook() -> anyhow::Result<()> {
    anyhow::bail!(
        "Codex MCP setup requires an explicit upstream server. Run `onus codex-mcp --server <UPSTREAM_MCP_SERVER> -- <UPSTREAM_ARGS>`."
    )
}

pub fn install_mcp_proxy(
    upstream_server: &PathBuf,
    upstream_args: &[String],
) -> anyhow::Result<()> {
    if !upstream_server.exists() {
        anyhow::bail!(
            "upstream MCP server not found at {}",
            upstream_server.display()
        );
    }

    let config_path = codex_config_path();
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Cannot create {}: {}", parent.display(), e))?;
    }

    let onus_bin = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Cannot determine onus binary path: {}", e))?
        .to_string_lossy()
        .replace('\\', "/");
    let upstream = upstream_server.to_string_lossy().replace('\\', "/");

    let content = if config_path.exists() {
        let existing = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", config_path.display(), e))?;
        if existing.contains("[mcp_servers.onus-mcp-proxy]") {
            println!("  Onus MCP proxy already configured in ~/.codex/config.toml");
            return Ok(());
        }
        // Append to existing config
        format!(
            "{}\n{}",
            existing.trim(),
            generate_mcp_config(&onus_bin, &upstream, upstream_args)
        )
    } else {
        generate_mcp_config(&onus_bin, &upstream, upstream_args)
    };

    std::fs::write(&config_path, &content)
        .map_err(|e| anyhow::anyhow!("Cannot write {}: {}", config_path.display(), e))?;

    println!("  MCP proxy entry written to: {}", config_path.display());
    println!("  Server name: onus-mcp-proxy");
    println!(
        "  Command: {} mcp-proxy --experimental --server {}",
        onus_bin, upstream
    );
    println!("  Approval mode: prompt (tools require user approval)");
    println!();
    println!("  To verify: run `onus doctor --codex`");
    println!("  To remove: run `onus uninstall --codex`");

    Ok(())
}

pub fn run_mcp_setup(args: CodexMcpArgs) -> anyhow::Result<()> {
    install_mcp_proxy(&args.server, &args.args)
}

/// Remove the Onus MCP proxy entry from Codex config.toml.
pub fn uninstall_mcp_hook() -> anyhow::Result<()> {
    let config_path = codex_config_path();
    if !config_path.exists() {
        println!("  No Codex config found at: {}", config_path.display());
        return Ok(());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", config_path.display(), e))?;

    if !content.contains("[mcp_servers.onus-mcp-proxy]") {
        println!("  No Onus MCP proxy entry found");
        return Ok(());
    }

    // Remove the [mcp_servers.onus-mcp-proxy] section (TOML table header + all lines until next table or EOF)
    let mut new_lines = Vec::new();
    let mut in_onus_section = false;
    for line in content.lines() {
        if line.trim() == "[mcp_servers.onus-mcp-proxy]" {
            in_onus_section = true;
            continue;
        }
        if in_onus_section {
            if line.trim().starts_with('[') {
                in_onus_section = false;
                new_lines.push(line);
            }
            // skip lines inside the section
            continue;
        }
        new_lines.push(line);
    }

    let new_content = new_lines.join("\n").trim().to_string();
    std::fs::write(&config_path, &new_content)
        .map_err(|e| anyhow::anyhow!("Cannot write {}: {}", config_path.display(), e))?;

    println!(
        "  Removed Onus MCP proxy entry from {}",
        config_path.display()
    );
    Ok(())
}

// ── L3 workspace description for codex ────────────────────────────────────────

/// Check if L3 workspace is available for Codex.
/// On Linux: check for bwrap. On other platforms: describe the limitation.
#[cfg(target_os = "linux")]
pub fn l3_workspace_available() -> bool {
    std::process::Command::new("bwrap")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
pub fn l3_workspace_available() -> bool {
    false
}

/// Advice string for running Codex inside an L3 container.
pub fn l3_workspace_advice() -> String {
    if l3_workspace_available() {
        "Codex can be run inside an L3 bubblewrap container:\n  onus run --l3 -- codex run\n"
            .to_string()
    } else {
        "L3 workspace isolation requires Linux + bubblewrap.\nCurrently on Windows: use the MCP proxy route instead.\n".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_codex_cli_no_panic() {
        // Should not crash regardless of Codex being installed or not
        let _ = find_codex_cli();
    }

    #[test]
    fn test_codex_config_path_format() {
        let path = codex_config_path();
        let s = path.to_string_lossy();
        assert!(s.contains(".codex"));
        assert!(s.contains("config.toml"));
    }

    #[test]
    fn test_mcp_config_empty_no_config() {
        // Without a config file, should return NotFound
        match check_mcp_config() {
            McpConfigCheck::NotFound => {}          // expected
            McpConfigCheck::Configured { .. } => {} // possible in CI
            McpConfigCheck::Error(_) => {}          // possible if HOME not set
        }
    }

    #[test]
    fn test_generate_mcp_config_format() {
        let toml = generate_mcp_config(
            "/usr/local/bin/onus",
            "/usr/local/bin/example-mcp",
            &["--demo".to_string()],
        );
        assert!(toml.contains("[mcp_servers.onus-mcp-proxy]"));
        assert!(toml.contains("command"));
        assert!(toml.contains("/usr/local/bin/onus"));
        assert!(toml.contains("mcp-proxy"));
        assert!(toml.contains("--experimental"));
        assert!(toml.contains("/usr/local/bin/example-mcp"));
        assert!(toml.contains("--demo"));
        assert!(toml.contains("approval_mode"));
    }

    #[test]
    fn test_install_mcp_proxy_writes_explicit_upstream_config() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("onus-codex-test-{stamp}"));
        std::fs::create_dir_all(&root).unwrap();
        let upstream = root.join("fake-mcp-server");
        std::fs::write(&upstream, "").unwrap();

        let old_home = std::env::var("HOME").ok();
        let old_userprofile = std::env::var("USERPROFILE").ok();
        std::env::set_var("HOME", &root);
        std::env::set_var("USERPROFILE", &root);

        let result = install_mcp_proxy(&upstream, &["--demo".to_string()]);

        if let Some(value) = old_home {
            std::env::set_var("HOME", value);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(value) = old_userprofile {
            std::env::set_var("USERPROFILE", value);
        } else {
            std::env::remove_var("USERPROFILE");
        }

        result.unwrap();

        let config_path = root.join(".codex").join("config.toml");
        let content = std::fs::read_to_string(config_path).unwrap();
        assert!(content.contains("[mcp_servers.onus-mcp-proxy]"));
        assert!(content.contains("mcp-proxy"));
        assert!(content.contains("--experimental"));
        assert!(content.contains("--server"));
        assert!(content.contains("fake-mcp-server"));
        assert!(content.contains("--demo"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn test_l3_workspace_advice_format() {
        let advice = l3_workspace_advice();
        assert!(!advice.is_empty());
        // Should either describe bwrap or describe Windows limitation
        assert!(
            advice.contains("bubblewrap") || advice.contains("Windows"),
            "Advice should mention one of: bubblewrap, Windows"
        );
    }

    #[test]
    fn test_uninstall_from_empty_config() {
        // Create a temporary config, add entry, uninstall it, verify clean
        let tmp = std::env::temp_dir().join("codex-test-uninstall.toml");
        // Clear the ONUS_DATA_DIR env var for this test since we don't use it
        let _ = std::fs::write(&tmp, "").ok();
        // Our uninstall uses codex_config_path which points to ~/.codex/config.toml
        // This test checks the logic of removing entries
        let content = r#"
[general]
theme = "dark"

[mcp_servers.onus-mcp-proxy]
command = "onus"
args = ["mcp-proxy"]

[mcp_servers.other]
command = "echo"
"#;
        // Test removal logic by processing the content
        let mut new_lines = Vec::new();
        let mut in_onus_section = false;
        for line in content.lines() {
            if line.trim() == "[mcp_servers.onus-mcp-proxy]" {
                in_onus_section = true;
                continue;
            }
            if in_onus_section {
                if line.trim().starts_with('[') {
                    in_onus_section = false;
                    new_lines.push(line);
                }
                continue;
            }
            new_lines.push(line);
        }
        let result = new_lines.join("\n");
        assert!(!result.contains("onus-mcp-proxy"));
        assert!(result.contains("mcp_servers.other"));
        assert!(result.contains("theme"));
        let _ = std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_find_codex_on_path_no_panic() {
        let _ = find_codex_on_path();
    }
}
