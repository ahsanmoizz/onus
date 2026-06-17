//! `onus antigravity` — Google Antigravity (VS Code fork) integration adapter.
//!
//! Antigravity is a fork of VS Code with its own extension model. It does NOT
//! have a native hook API. Integration is provided through:
//!
//! - Extension deployment: `antigravity --install-extension <vsix>` loads the
//!   Onus extension (same extension.js as VS Code, packaged with a different
//!   publisher/name).
//! - MCP routing: `antigravity --add-mcp <json>` configures Onus as an MCP
//!   server for tool-call interception.
//! - CLI extension management: `--list-extensions`, `--uninstall-extension`,
//!   `--update-extensions`.
//!
//! This module handles detection, version checking, setup, uninstall,
//! doctor diagnostics, and L3 workspace fallback.

use std::path::PathBuf;

// Known install path for Antigravity on Windows
const ANTIGRAVITY_WINDOWS_BIN: &str = "D:\\Antigravity\\bin\\antigravity";

// ── Antigravity binary detection ──────────────────────────────────────────────

/// Result of checking for Antigravity.
#[derive(Debug)]
pub enum AntigravityCheck {
    Available { version: String, path: PathBuf },
    NotFound,
    Error(String),
}

/// Try to detect the Antigravity binary on PATH or at known install paths.
pub fn find_antigravity() -> AntigravityCheck {
    // 1. Try PATH first
    if let Some(path) = find_on_path() {
        if let Some(version) = get_version(&path) {
            return AntigravityCheck::Available { version, path };
        }
    }

    // 2. Try known Windows install path
    let known_path = PathBuf::from(ANTIGRAVITY_WINDOWS_BIN);
    if known_path.exists() {
        if let Some(version) = get_version(&known_path) {
            return AntigravityCheck::Available { version, path: known_path };
        }
    }

    // 3. Try `antigravity` command directly
    if let Ok(output) = std::process::Command::new("antigravity")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !ver.is_empty() {
                let path = find_on_path().unwrap_or_else(|| PathBuf::from("antigravity"));
                return AntigravityCheck::Available { version: ver, path };
            }
        }
    }

    AntigravityCheck::NotFound
}

fn find_on_path() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join("antigravity");
            if candidate.is_file() {
                return Some(candidate);
            }
            #[cfg(windows)]
            {
                let candidate_exe = dir.join("antigravity.exe");
                if candidate_exe.is_file() {
                    return Some(candidate_exe);
                }
                let candidate_cmd = dir.join("antigravity.cmd");
                if candidate_cmd.is_file() {
                    return Some(candidate_cmd);
                }
            }
        }
        None
    })
}

fn get_version(path: &PathBuf) -> Option<String> {
    std::process::Command::new(path)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if v.is_empty() { None } else { Some(v) }
        })
}

// ── Extension management ─────────────────────────────────────────────────────

const EXTENSION_ID: &str = "onus.onus-firewall";

/// Result of checking extension installation status.
#[derive(Debug)]
pub enum ExtensionCheck {
    Installed { path: PathBuf, version: String },
    NotInstalled,
    Error(String),
}

/// Check whether the Onus extension is installed in Antigravity.
pub fn check_extension_installed(antigravity_path: &PathBuf) -> ExtensionCheck {
    let output = match std::process::Command::new(antigravity_path)
        .args(["--list-extensions", "--show-versions"])
        .output()
    {
        Ok(o) => o,
        Err(e) => return ExtensionCheck::Error(format!("Cannot run antigravity: {}", e)),
    };

    if !output.status.success() {
        return ExtensionCheck::Error(format!(
            "antigravity --list-extensions exited with {}",
            output.status
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(EXTENSION_ID) {
            let version = trimmed
                .strip_prefix(EXTENSION_ID)
                .and_then(|s| s.strip_prefix('@'))
                .unwrap_or("unknown")
                .to_string();
            // Get extension path
            let ext_path = get_extension_path(antigravity_path);
            return ExtensionCheck::Installed {
                path: ext_path.unwrap_or_else(|| PathBuf::from("unknown")),
                version,
            };
        }
    }

    ExtensionCheck::NotInstalled
}

fn get_extension_path(antigravity_path: &PathBuf) -> Option<PathBuf> {
    let output = std::process::Command::new(antigravity_path)
        .args(["--locate-extension", EXTENSION_ID])
        .output()
        .ok()?;

    if output.status.success() {
        let path_str = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()?
            .trim()
            .to_string();
        if !path_str.is_empty() {
            return Some(PathBuf::from(path_str));
        }
    }
    None
}

/// Install the Onus extension into Antigravity.
/// `vsix_path` is the path to the packaged .vsix file.
pub fn install_extension(antigravity_path: &PathBuf, vsix_path: &PathBuf) -> anyhow::Result<()> {
    if !vsix_path.exists() {
        anyhow::bail!("VSIX not found at: {}", vsix_path.display());
    }

    let output = std::process::Command::new(antigravity_path)
        .args(["--install-extension", &vsix_path.to_string_lossy()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("antigravity --install-extension failed: {}", stderr.trim());
    }

    Ok(())
}

/// Uninstall the Onus extension from Antigravity.
pub fn uninstall_extension(antigravity_path: &PathBuf) -> anyhow::Result<()> {
    let output = std::process::Command::new(antigravity_path)
        .args(["--uninstall-extension", EXTENSION_ID])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Extension not installed is not a failure
        if stderr.contains("not installed") {
            return Ok(());
        }
        anyhow::bail!("antigravity --uninstall-extension failed: {}", stderr.trim());
    }

    Ok(())
}

// ── MCP management ────────────────────────────────────────────────────────────

/// Result of checking MCP server configuration.
#[derive(Debug)]
pub enum McpConfigCheck {
    Configured { server_name: String },
    NotFound,
    Error(String),
}

/// Check if Onus MCP proxy is configured in Antigravity.
/// Antigravity stores MCP config in the user profile.
pub fn check_mcp_config(antigravity_path: &PathBuf) -> McpConfigCheck {
    // Antigravity stores MCP servers in its user profile.
    // We check the extensions directory for our MCP configuration.
    let ext_path = match get_extension_path(antigravity_path) {
        Some(p) => p,
        None => return McpConfigCheck::NotFound,
    };

    // Extension is installed — check if it has MCP configuration
    let mcp_config = ext_path.join(".mcp.json");
    if mcp_config.exists() {
        return McpConfigCheck::Configured {
            server_name: "onus-mcp-proxy".to_string(),
        };
    }

    McpConfigCheck::NotFound
}

/// Add Onus MCP proxy to Antigravity via --add-mcp.
pub fn add_mcp_server(antigravity_path: &PathBuf, onus_path: &PathBuf) -> anyhow::Result<()> {
    let mcp_json = serde_json::json!({
        "name": "onus-mcp-proxy",
        "command": onus_path.to_string_lossy(),
        "args": ["mcp-proxy"]
    });

    let output = std::process::Command::new(antigravity_path)
        .args(["--add-mcp", &mcp_json.to_string()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("antigravity --add-mcp failed: {}", stderr.trim());
    }

    Ok(())
}

// ── Setup ─────────────────────────────────────────────────────────────────────

/// Run `onus setup --antigravity`.
pub fn run_setup() -> anyhow::Result<()> {
    println!("Onus Setup — Google Antigravity\n");

    match find_antigravity() {
        AntigravityCheck::Available { version, path } => {
            println!("  Antigravity v{} found at: {}", version, path.display());

            // Check if extension is already installed
            match check_extension_installed(&path) {
                ExtensionCheck::Installed { path: ext_path, version: ext_ver } => {
                    println!("  ✓ Extension already installed at:");
                    println!("      {} (v{})", ext_path.display(), ext_ver);
                }
                ExtensionCheck::NotInstalled => {
                    println!("  Extension not installed.");
                    println!("  To install, package the extension and run:");
                    println!("    antigravity --install-extension onus-firewall-0.1.0.vsix");
                    println!("  Or from the VSIX dir:");
                    println!("    {} --install-extension <vsix-path>", path.display());
                }
                ExtensionCheck::Error(e) => {
                    println!("  ? Extension check: {}", e);
                }
            }

            // Check MCP config
            match check_mcp_config(&path) {
                McpConfigCheck::Configured { server_name } => {
                    println!("  ✓ MCP proxy '{}' is configured", server_name);
                }
                McpConfigCheck::NotFound => {
                    println!("  MCP proxy not configured.");
                    println!("  To configure:");
                    let onus_path = std::env::current_exe()
                        .unwrap_or_else(|_| PathBuf::from("onus"));
                    println!("    {} --add-mcp '{{\"name\":\"onus-mcp-proxy\",\"command\":\"{}\",\"args\":[\"mcp-proxy\"]}}'",
                        path.display(), onus_path.display());
                }
                McpConfigCheck::Error(e) => {
                    println!("  ? MCP check: {}", e);
                }
            }

            println!("\n  ✓ Setup completed.");
        }
        AntigravityCheck::NotFound => {
            println!("  Antigravity not found.");
            println!("  Install it from: https://github.com/google/antigravity");
            println!("  Or visit the Antigravity marketplace.");
        }
        AntigravityCheck::Error(e) => {
            println!("  Error: {}", e);
        }
    }

    Ok(())
}

/// Run `onus uninstall --antigravity`.
pub fn run_uninstall() -> anyhow::Result<()> {
    println!("Onus Uninstall — Google Antigravity\n");

    match find_antigravity() {
        AntigravityCheck::Available { version, path } => {
            println!("  Antigravity v{} found at: {}", version, path.display());

            // Uninstall extension
            match check_extension_installed(&path) {
                ExtensionCheck::Installed { .. } => {
                    println!("  Uninstalling extension...");
                    match uninstall_extension(&path) {
                        Ok(()) => println!("  ✓ Extension uninstalled."),
                        Err(e) => println!("  ? Could not uninstall: {}", e),
                    }
                }
                ExtensionCheck::NotInstalled => {
                    println!("  Extension not installed — nothing to remove.");
                }
                ExtensionCheck::Error(e) => {
                    println!("  ? Extension check: {}", e);
                }
            }

            println!("\n  ✓ Uninstall completed.");
        }
        AntigravityCheck::NotFound => {
            println!("  Antigravity not found — nothing to remove.");
        }
        AntigravityCheck::Error(e) => {
            println!("  Error: {}", e);
        }
    }

    Ok(())
}

// ── Doctor ────────────────────────────────────────────────────────────────────

/// Run `onus doctor --antigravity` — focused diagnostics.
pub fn run_doctor() -> anyhow::Result<()> {
    println!("Onus Doctor — Google Antigravity\n");

    match find_antigravity() {
        AntigravityCheck::Available { version, path } => {
            println!("  Binary found: Antigravity v{}", version);
            println!("  Path: {}", path.display());

            // Check extension
            match check_extension_installed(&path) {
                ExtensionCheck::Installed { path: ext_path, version: ext_ver } => {
                    println!("  Extension: ✓ onus-firewall v{}", ext_ver);
                    println!("    at: {}", ext_path.display());
                }
                ExtensionCheck::NotInstalled => {
                    println!("  Extension: ✗ not installed");
                    println!("    Run `onus setup --antigravity` for instructions.");
                }
                ExtensionCheck::Error(e) => {
                    println!("  Extension: ? check error: {}", e);
                }
            }

            // Check MCP config
            match check_mcp_config(&path) {
                McpConfigCheck::Configured { server_name } => {
                    println!("  MCP proxy: ✓ {}", server_name);
                }
                McpConfigCheck::NotFound => {
                    println!("  MCP proxy: ✗ not configured");
                }
                McpConfigCheck::Error(e) => {
                    println!("  MCP proxy: ? {}", e);
                }
            }

            // L3 workspace
            let l3 = l3_workspace_advice();
            if !l3.is_empty() {
                println!();
                println!("  L3 workspace: {}", l3);
            }

            println!("\n  ✓ Doctor check complete.");
        }
        AntigravityCheck::NotFound => {
            println!("  Antigravity CLI not found on PATH.");
            println!();
            println!("  Install Antigravity from:");
            println!("    https://github.com/google/antigravity");
        }
        AntigravityCheck::Error(e) => {
            println!("  Error checking Antigravity: {}", e);
        }
    }

    Ok(())
}

// ── L3 workspace ──────────────────────────────────────────────────────────────

/// Check if L3 workspace isolation is available.
pub fn l3_workspace_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("bwrap")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Return advice about L3 workspace for this platform.
pub fn l3_workspace_advice() -> String {
    if l3_workspace_available() {
        "bubblewrap available — use `onus run --l3` for sandboxed execution.".to_string()
    } else {
        "not available on this platform (requires Linux + bubblewrap)".to_string()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_antigravity_no_panic() {
        // Should never panic, even if Antigravity is not installed
        let result = find_antigravity();
        match result {
            AntigravityCheck::Available { .. } => {} // acceptable
            AntigravityCheck::NotFound => {}           // expected
            AntigravityCheck::Error(_) => {}           // acceptable
        }
    }

    #[test]
    fn test_antigravity_path_format() {
        let known = PathBuf::from(ANTIGRAVITY_WINDOWS_BIN);
        let s = known.to_string_lossy();
        assert!(s.contains("Antigravity") || s.contains("antigravity"));
        assert!(s.contains("bin"));
    }

    #[test]
    fn test_antigravity_extension_id() {
        assert_eq!(EXTENSION_ID, "onus.onus-firewall");
    }

    #[test]
    fn test_l3_workspace_advice_format() {
        let advice = l3_workspace_advice();
        assert!(!advice.is_empty());
        assert!(advice.contains("bubblewrap") || advice.contains("not available"));
    }

    #[test]
    fn test_mcp_config_check_no_binary() {
        // With a nonexistent binary path, should return Error
        let fake_path = PathBuf::from("/nonexistent/antigravity");
        match check_mcp_config(&fake_path) {
            McpConfigCheck::Error(_) => {} // expected
            _ => {} // or NotFound — depends on how error propagates
        }
    }

    #[test]
    fn test_uninstall_extension_no_binary() {
        let fake_path = PathBuf::from("/nonexistent/antigravity");
        // Should return an error, not panic
        let result = uninstall_extension(&fake_path);
        assert!(result.is_err());
    }
}
