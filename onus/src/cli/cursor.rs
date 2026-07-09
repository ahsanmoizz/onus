//! `onus cursor` — Cursor IDE integration adapter.
//!
//! Cursor is an AI-native IDE (fork of VS Code) with its own extension model.
//! Integration surfaces:
//!
//! - **P15E-04: Cursor CLI (agent)** — MCP proxy routing via `.cursor/mcp.json`.
//!   Config: `~/.cursor/mcp.json` with `{ "mcpServers": { "onus": { "command": "...", "args": [] } } }`.
//!
//! - **P15E-12: Cursor Agent (IDE)** — Native hook API via `.cursor/hooks.json`.
//!   PreToolUse hook receives JSON on stdin, returns `{ "allowed": bool }`.
//!   Config: `~/.cursor/hooks.json` with `{ "preToolUse": { "command": "onus cursor-hook" } }`.
//!
//! - **P15E-16: Cursor Background Agents** — Cloud agent execution. Same hook
//!   infrastructure (command-based only). MCP proxy also relevant.
//!
//! This module handles binary detection, hook installation/removal, MCP config
//! management, setup, uninstall, doctor diagnostics, and L3 workspace fallback.

use clap::Args;
use std::path::{Path, PathBuf};

const MCP_SERVER_NAME: &str = "onus-mcp-proxy";

#[derive(Args)]
pub struct CursorMcpArgs {
    /// Upstream MCP server binary that Onus should wrap.
    #[arg(long)]
    pub server: PathBuf,

    /// Arguments passed to the upstream MCP server after `--`.
    #[arg(last = true)]
    pub args: Vec<String>,
}

// ── Path helpers ─────────────────────────────────────────────────────────────

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string())
        .into()
}

/// Cursor IDE hooks directory
pub fn cursor_config_dir() -> PathBuf {
    home_dir().join(".cursor")
}

/// Path to hooks.json
pub fn hooks_json_path() -> PathBuf {
    cursor_config_dir().join("hooks.json")
}

/// Path to mcp.json
pub fn mcp_json_path() -> PathBuf {
    cursor_config_dir().join("mcp.json")
}

// ── Cursor binary detection ──────────────────────────────────────────────────

/// Result of checking for Cursor.
#[derive(Debug)]
pub enum CursorCheck {
    Available { version: String, path: PathBuf },
    NotFound,
    Error(String),
}

/// Try to detect the Cursor binary.
pub fn find_cursor() -> CursorCheck {
    // 1. Try PATH first
    if let Some(path) = find_on_path() {
        if let Some(version) = get_version(&path) {
            return CursorCheck::Available { version, path };
        }
    }

    // 2. Try Windows known install paths
    for path_str in &[
        "C:\\Users\\A\\AppData\\Local\\Programs\\cursor\\Cursor.exe",
        "C:\\Users\\A\\AppData\\Local\\cursor\\Cursor.exe",
        "C:\\Program Files\\Cursor\\Cursor.exe",
    ] {
        let path = PathBuf::from(path_str);
        if path.exists() {
            if let Some(version) = get_version(&path) {
                return CursorCheck::Available { version, path };
            }
            return CursorCheck::Available {
                version: "unknown".to_string(),
                path,
            };
        }
    }

    // 3. Try `agent --version` (Cursor CLI agent binary)
    if let Ok(output) = std::process::Command::new("agent")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !ver.is_empty() {
                return CursorCheck::Available {
                    version: ver,
                    path: PathBuf::from("agent"),
                };
            }
        }
    }

    CursorCheck::NotFound
}

fn find_on_path() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        for dir in std::env::split_paths(&paths) {
            for name in &["cursor", "cursor.exe", "agent", "agent.exe"] {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    })
}

fn get_version(path: &PathBuf) -> Option<String> {
    if path.file_name().and_then(|s| s.to_str()) == Some("agent") {
        return std::process::Command::new(path)
            .arg("--version")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| {
                let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if v.is_empty() {
                    None
                } else {
                    Some(v)
                }
            });
    }
    // Cursor GUI binary might not support --version
    None
}

// ── Hook management ─────────────────────────────────────────────────────────

/// Result of checking hook installation status.
#[derive(Debug)]
pub enum HookCheck {
    Installed { command: String },
    NotInstalled,
    Error(String),
}

/// Check whether Onus cursor-hook is configured in hooks.json.
pub fn check_hook_installed() -> HookCheck {
    let hooks_path = hooks_json_path();
    if !hooks_path.exists() {
        return HookCheck::NotInstalled;
    }

    let content = match std::fs::read_to_string(&hooks_path) {
        Ok(c) => c,
        Err(e) => return HookCheck::Error(format!("Cannot read {}: {}", hooks_path.display(), e)),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => return HookCheck::Error(format!("Invalid JSON: {}", e)),
    };

    if let Some(pre_tool) = json.get("preToolUse") {
        if let Some(cmd) = pre_tool.get("command").and_then(serde_json::Value::as_str) {
            if cmd.contains("onus") {
                return HookCheck::Installed {
                    command: cmd.to_string(),
                };
            }
        }
    }

    HookCheck::NotInstalled
}

/// Install the Onus cursor-hook into hooks.json.
pub fn install_hook() -> anyhow::Result<()> {
    let hooks_path = hooks_json_path();
    let existing = if hooks_path.exists() {
        let content = std::fs::read_to_string(&hooks_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mut config = existing.as_object().cloned().unwrap_or_default();
    let onus_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("onus"));

    config.insert(
        "preToolUse".to_string(),
        serde_json::json!({
            "command": onus_path.to_string_lossy(),
            "args": ["cursor-hook"]
        }),
    );

    let config_dir = cursor_config_dir();
    std::fs::create_dir_all(&config_dir)?;

    let output = serde_json::to_string_pretty(&config)?;
    std::fs::write(&hooks_path, output)?;

    Ok(())
}

/// Remove the Onus cursor-hook from hooks.json.
pub fn remove_hook() -> anyhow::Result<()> {
    let hooks_path = hooks_json_path();
    if !hooks_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&hooks_path)?;
    let mut config: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(obj) = config.as_object_mut() {
        obj.remove("preToolUse");
    }

    let is_empty = config.as_object().is_none_or(|o| o.is_empty());
    if is_empty {
        std::fs::remove_file(&hooks_path)?;
    } else {
        let output = serde_json::to_string_pretty(&config)?;
        std::fs::write(&hooks_path, output)?;
    }

    Ok(())
}

// ── MCP management ──────────────────────────────────────────────────────────

/// Result of checking MCP configuration.
#[derive(Debug)]
pub enum McpConfigCheck {
    Configured { server_name: String },
    NotFound,
    Error(String),
}

/// Check whether Onus is configured as an MCP server in mcp.json.
pub fn check_mcp_configured() -> McpConfigCheck {
    let mcp_path = mcp_json_path();
    if !mcp_path.exists() {
        return McpConfigCheck::NotFound;
    }

    let content = match std::fs::read_to_string(&mcp_path) {
        Ok(c) => c,
        Err(e) => return McpConfigCheck::Error(format!("Cannot read {e}")),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return McpConfigCheck::NotFound,
    };

    if let Some(entry) = json
        .get("mcpServers")
        .and_then(serde_json::Value::as_object)
        .and_then(|servers| servers.get(MCP_SERVER_NAME).or_else(|| servers.get("onus")))
    {
        if has_valid_onus_proxy(entry) {
            return McpConfigCheck::Configured {
                server_name: MCP_SERVER_NAME.to_string(),
            };
        }
        return McpConfigCheck::Error(
            "mcp.json entry must run `onus mcp-proxy --experimental --server <upstream>`"
                .to_string(),
        );
    }

    McpConfigCheck::NotFound
}

fn proxy_entry(
    onus_path: &Path,
    upstream_server: &Path,
    upstream_args: &[String],
) -> serde_json::Value {
    let mut args = vec![
        "mcp-proxy".to_string(),
        "--experimental".to_string(),
        "--server".to_string(),
        upstream_server.to_string_lossy().to_string(),
    ];
    if !upstream_args.is_empty() {
        args.push("--".to_string());
        args.extend(upstream_args.iter().cloned());
    }

    serde_json::json!({
        "type": "stdio",
        "command": onus_path.to_string_lossy(),
        "args": args
    })
}

fn has_valid_onus_proxy(entry: &serde_json::Value) -> bool {
    let command_ok = entry
        .get("command")
        .and_then(serde_json::Value::as_str)
        .map(|command| command.to_ascii_lowercase().contains("onus"))
        .unwrap_or(false);
    let args = entry.get("args").and_then(serde_json::Value::as_array);
    let args_ok = args
        .map(|args| {
            let values: Vec<&str> = args.iter().filter_map(serde_json::Value::as_str).collect();
            values.contains(&"mcp-proxy")
                && values.contains(&"--experimental")
                && values.contains(&"--server")
                && values
                    .iter()
                    .position(|value| *value == "--server")
                    .and_then(|index| values.get(index + 1))
                    .is_some()
        })
        .unwrap_or(false);
    command_ok && args_ok
}

pub fn add_mcp_proxy_server(
    onus_path: &PathBuf,
    upstream_server: &Path,
    upstream_args: &[String],
) -> anyhow::Result<()> {
    if !upstream_server.exists() {
        anyhow::bail!(
            "upstream MCP server not found at {}",
            upstream_server.display()
        );
    }

    let mcp_path = mcp_json_path();
    let existing = if mcp_path.exists() {
        let content = std::fs::read_to_string(&mcp_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mut config = existing.as_object().cloned().unwrap_or_default();

    let mcp_servers = config
        .entry("mcpServers".to_string())
        .or_insert_with(|| serde_json::json!({}));
    let servers = mcp_servers.as_object_mut().unwrap();

    servers.insert(
        MCP_SERVER_NAME.to_string(),
        proxy_entry(onus_path, upstream_server, upstream_args),
    );

    let config_dir = cursor_config_dir();
    std::fs::create_dir_all(&config_dir)?;

    let output = serde_json::to_string_pretty(&config)?;
    std::fs::write(&mcp_path, output)?;

    Ok(())
}

/// Add Onus as an MCP server in mcp.json.
pub fn add_mcp_server() -> anyhow::Result<()> {
    anyhow::bail!(
        "Cursor MCP setup requires an explicit upstream server. Run `onus cursor-mcp --server <UPSTREAM_MCP_SERVER> -- <UPSTREAM_ARGS>`."
    )
}

/// Remove Onus as an MCP server from mcp.json.
pub fn remove_mcp_server() -> anyhow::Result<()> {
    let mcp_path = mcp_json_path();
    if !mcp_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&mcp_path)?;
    let mut config: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(obj) = config.as_object_mut() {
        if let Some(servers) = obj
            .get_mut("mcpServers")
            .and_then(serde_json::Value::as_object_mut)
        {
            servers.remove("onus");
            if servers.is_empty() {
                obj.remove("mcpServers");
            }
        }
    }

    let is_empty = config.as_object().is_none_or(|o| o.is_empty());
    if is_empty {
        let _ = std::fs::remove_file(&mcp_path);
    } else {
        let output = serde_json::to_string_pretty(&config)?;
        std::fs::write(&mcp_path, output)?;
    }

    Ok(())
}

// ── Setup ────────────────────────────────────────────────────────────────────

/// Run `onus setup --cursor`.
pub fn run_setup() -> anyhow::Result<()> {
    println!("Onus Setup — Cursor IDE\n");

    match find_cursor() {
        CursorCheck::Available { version, path } => {
            println!("  Cursor v{} found at: {}", version, path.display());

            match check_hook_installed() {
                HookCheck::Installed { command } => {
                    println!("  ✓ Hook already configured: {}", command);
                }
                HookCheck::NotInstalled => {
                    println!("  Installing PreToolUse hook...");
                    install_hook()?;
                    println!("  ✓ Hook installed at: {}", hooks_json_path().display());
                }
                HookCheck::Error(e) => {
                    println!("  ? Hook check: {}", e);
                }
            }

            match check_mcp_configured() {
                McpConfigCheck::Configured { server_name } => {
                    println!("  ✓ MCP server '{}' already configured", server_name);
                }
                McpConfigCheck::NotFound => {
                    println!("  MCP proxy not configured.");
                    println!("  To route a real MCP server through Onus, run:");
                    println!(
                        "    onus cursor-mcp --server <UPSTREAM_MCP_SERVER> -- <UPSTREAM_ARGS>"
                    );
                    return Ok(());
                }
                McpConfigCheck::Error(e) => {
                    println!("  ? MCP check: {}", e);
                }
            }

            println!("\n  ✓ Setup completed.");
        }
        CursorCheck::NotFound => {
            println!("  Cursor not found.");
            println!("  Install it from: https://cursor.com");
            println!("  Or via: curl -fsSL https://cursor.com/install | sh");
        }
        CursorCheck::Error(e) => {
            println!("  Error: {}", e);
        }
    }

    Ok(())
}

pub fn run_mcp_setup(args: CursorMcpArgs) -> anyhow::Result<()> {
    let onus_path = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("cannot determine onus binary path: {e}"))?;

    add_mcp_proxy_server(&onus_path, &args.server, &args.args)?;

    println!("Cursor MCP proxy configured.");
    println!("  Config: {}", mcp_json_path().display());
    println!("  Server: {}", args.server.display());
    println!("  Enforcement: L2 ROUTED ONLY");
    println!();
    println!("Only MCP calls routed through this proxy are governed by Onus.");
    Ok(())
}

/// Run `onus uninstall --cursor`.
pub fn run_uninstall() -> anyhow::Result<()> {
    println!("Onus Uninstall — Cursor IDE\n");

    match find_cursor() {
        CursorCheck::Available { version, path } => {
            println!("  Cursor v{} at {}", version, path.display());

            match check_hook_installed() {
                HookCheck::Installed { .. } => {
                    println!("  Removing hook...");
                    remove_hook()?;
                    println!("  ✓ Hook removed.");
                }
                HookCheck::NotInstalled => {
                    println!("  Hook not installed — nothing to remove.");
                }
                HookCheck::Error(e) => {
                    println!("  ? Hook removal: {}", e);
                }
            }

            match check_mcp_configured() {
                McpConfigCheck::Configured { .. } => {
                    println!("  Removing MCP server...");
                    remove_mcp_server()?;
                    println!("  ✓ MCP server removed.");
                }
                McpConfigCheck::NotFound => {
                    println!("  MCP server not configured — nothing to remove.");
                }
                McpConfigCheck::Error(e) => {
                    println!("  ? MCP removal: {}", e);
                }
            }

            println!("\n  ✓ Uninstall completed.");
        }
        CursorCheck::NotFound => {
            println!("  Cursor not found — nothing to remove.");
        }
        CursorCheck::Error(e) => {
            println!("  Error: {}", e);
        }
    }

    Ok(())
}

// ── Doctor ───────────────────────────────────────────────────────────────────

/// Run `onus doctor --cursor` — focused diagnostics for all Cursor surfaces.
pub fn run_doctor() -> anyhow::Result<()> {
    let ok = "\x1b[32mOK\x1b[0m";
    let warn = "\x1b[33mWARN\x1b[0m";
    let fail = "\x1b[31mFAIL\x1b[0m";

    println!("Onus Doctor — Cursor IDE\n");

    println!("  Integration surfaces:");
    println!("    P15E-04: Cursor CLI (MCP proxy)");
    println!("    P15E-12: Cursor IDE Agent (native hook)");
    println!("    P15E-16: Cursor Background Agents (hook + MCP)\n");

    match find_cursor() {
        CursorCheck::Available { version, path } => {
            println!(
                "  [{ok}]  Binary found: Cursor v{} at {}",
                version,
                path.display()
            );

            match check_hook_installed() {
                HookCheck::Installed { command } => {
                    println!("  [{ok}]  PreToolUse hook: configured: {}", command);
                }
                HookCheck::NotInstalled => {
                    println!("  [{warn}] PreToolUse hook: not configured");
                    println!("    Run `onus setup --cursor` to install.");
                }
                HookCheck::Error(e) => {
                    println!("  [{fail}] PreToolUse hook: error: {}", e);
                }
            }

            match check_mcp_configured() {
                McpConfigCheck::Configured { server_name } => {
                    println!("  [{ok}]  MCP server: '{}' configured", server_name);
                    println!("        Enforcement label: L2 ROUTED ONLY");
                }
                McpConfigCheck::NotFound => {
                    println!(
                        "  [{warn}] MCP server: not configured at {}",
                        mcp_json_path().display()
                    );
                    println!(
                        "    Run `onus cursor-mcp --server <UPSTREAM_MCP_SERVER> -- <UPSTREAM_ARGS>` to configure."
                    );
                }
                McpConfigCheck::Error(e) => {
                    println!("  [{fail}] MCP server: error: {}", e);
                }
            }

            let l3 = l3_workspace_advice();
            if !l3.is_empty() {
                println!("  [{ok}]  L3 workspace: {l3}");
            }
        }
        CursorCheck::NotFound => {
            println!("  [{warn}] Binary: Cursor not found on PATH");
            println!("\n  Install Cursor:");
            println!("    curl -fsSL https://cursor.com/install | sh");
            println!("  Or download from: https://cursor.com/downloads\n");

            match check_hook_installed() {
                HookCheck::Installed { command } => {
                    println!("  [{ok}]  PreToolUse hook: configured: {}", command);
                }
                HookCheck::NotInstalled => {}
                HookCheck::Error(_) => {}
            }

            match check_mcp_configured() {
                McpConfigCheck::Configured { server_name } => {
                    println!("  [{ok}]  MCP server: '{}' configured", server_name);
                }
                McpConfigCheck::NotFound => {}
                McpConfigCheck::Error(_) => {}
            }
        }
        CursorCheck::Error(e) => {
            println!("  [{fail}] Cursor: check error: {}", e);
        }
    }

    Ok(())
}

// ── L3 workspace ─────────────────────────────────────────────────────────────

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
    fn test_find_cursor_no_panic() {
        let result = find_cursor();
        match result {
            CursorCheck::Available { .. } => {}
            CursorCheck::NotFound => {}
            CursorCheck::Error(_) => {}
        }
    }

    #[test]
    fn test_cursor_config_dir() {
        let dir = cursor_config_dir();
        assert!(dir.to_string_lossy().contains(".cursor"));
    }

    #[test]
    fn test_hooks_json_path() {
        let path = hooks_json_path();
        assert!(path.to_string_lossy().contains("hooks.json"));
    }

    #[test]
    fn test_mcp_json_path() {
        let path = mcp_json_path();
        assert!(path.to_string_lossy().contains("mcp.json"));
    }

    #[test]
    fn test_hook_config_format() {
        let config = serde_json::json!({
            "preToolUse": {
                "command": "/usr/local/bin/onus",
                "args": ["cursor-hook"]
            }
        });
        let cmd = config["preToolUse"]["command"].as_str().unwrap();
        assert!(cmd.contains("onus"));
    }

    #[test]
    fn test_mcp_config_format() {
        let config = serde_json::json!({
            "mcpServers": {
                "onus-mcp-proxy": {
                    "type": "stdio",
                    "command": "/usr/local/bin/onus",
                    "args": ["mcp-proxy", "--experimental", "--server", "/usr/local/bin/example-mcp"]
                }
            }
        });
        let servers = config["mcpServers"].as_object().unwrap();
        assert!(servers.contains_key("onus-mcp-proxy"));
        assert_eq!(
            servers["onus-mcp-proxy"]["command"].as_str().unwrap(),
            "/usr/local/bin/onus"
        );
        assert!(has_valid_onus_proxy(&servers["onus-mcp-proxy"]));
    }

    #[test]
    fn test_dead_mcp_proxy_config_is_rejected() {
        let config = serde_json::json!({
            "command": "/usr/local/bin/onus",
            "args": ["mcp-proxy"]
        });
        assert!(!has_valid_onus_proxy(&config));
    }

    #[test]
    fn test_add_mcp_proxy_server_writes_valid_cursor_config() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("onus-cursor-test-{stamp}"));
        std::fs::create_dir_all(&root).unwrap();
        let upstream = root.join("fake-mcp-server");
        std::fs::write(&upstream, "").unwrap();

        let old_home = std::env::var("HOME").ok();
        let old_userprofile = std::env::var("USERPROFILE").ok();
        std::env::set_var("HOME", &root);
        std::env::set_var("USERPROFILE", &root);

        let result = add_mcp_proxy_server(
            &PathBuf::from("/usr/local/bin/onus"),
            &upstream,
            &["--demo".to_string()],
        );

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

        let config_path = root.join(".cursor").join("mcp.json");
        let content = std::fs::read_to_string(config_path).unwrap();
        let config: serde_json::Value = serde_json::from_str(&content).unwrap();
        let entry = &config["mcpServers"][MCP_SERVER_NAME];
        assert_eq!(entry["type"], "stdio");
        assert!(has_valid_onus_proxy(entry));
        assert_eq!(entry["args"][4], "--");
        assert_eq!(entry["args"][5], "--demo");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn test_l3_workspace_advice_format() {
        let advice = l3_workspace_advice();
        assert!(!advice.is_empty());
        assert!(advice.contains("bubblewrap") || advice.contains("not available"));
    }

    #[test]
    fn test_mcp_remove_from_config() {
        let mut config = serde_json::json!({
            "mcpServers": {
                "onus": { "command": "onus", "args": ["mcp-proxy"] },
                "other": { "command": "other", "args": [] }
            }
        });

        if let Some(servers) = config
            .get_mut("mcpServers")
            .and_then(serde_json::Value::as_object_mut)
        {
            servers.remove("onus");
        }
        assert!(config["mcpServers"]
            .as_object()
            .unwrap()
            .contains_key("other"));
        assert!(!config["mcpServers"]
            .as_object()
            .unwrap()
            .contains_key("onus"));
    }
}
