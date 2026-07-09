//! `onus setup` / `onus uninstall` — surface integration management.
//!
//! Installs and removes Onus hooks for Claude Code CLI, VS Code, and other
//! integration surfaces. Detects the surface and writes the appropriate
//! configuration.

use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct SetupArgs {
    /// Set up only the Claude Code CLI hook
    #[arg(long)]
    pub claude: bool,

    /// Set up only the OpenAI Codex CLI MCP proxy
    #[arg(long)]
    pub codex: bool,

    /// Set up only the Google Antigravity extension and MCP proxy
    #[arg(long)]
    pub antigravity: bool,

    /// Set up only the VS Code extension
    #[arg(long)]
    pub vscode: bool,

    /// Set up only the Cursor IDE hooks and MCP proxy
    #[arg(long)]
    pub cursor: bool,
}

// ── Main setup / uninstall dispatch ───────────────────────────────────────────

pub fn run(args: SetupArgs) -> anyhow::Result<()> {
    if args.claude {
        return run_claude();
    }
    if args.codex {
        return crate::cli::codex::install_mcp_hook();
    }
    if args.antigravity {
        return crate::cli::antigravity::run_setup();
    }
    if args.vscode {
        println!("VS Code setup is not yet implemented as a CLI command.");
        println!("Use the VS Code extension or run `onus setup` interactively.");
        return Ok(());
    }
    if args.cursor {
        return crate::cli::cursor::run_setup();
    }

    println!("Onus Setup — Interactive Surface Integration");
    println!();

    let surfaces = detect_surfaces();

    if surfaces.is_empty() {
        println!("No supported integration surfaces detected.");
        println!();
        println!("Onus can integrate with:");
        println!("  - Claude Code CLI   (requires `claude` on PATH)");
        println!("  - VS Code Extension  (requires `code` on PATH)");
        println!();
        println!("Install a surface first, then re-run `onus setup`.");
        return Ok(());
    }

    for surface in &surfaces {
        match surface {
            DetectedSurface::ClaudeCode { path } => {
                println!("  Found Claude Code CLI at: {}", path.display());
            }
            DetectedSurface::Codex { path } => {
                println!("  Found OpenAI Codex CLI at: {}", path.display());
            }
            DetectedSurface::Antigravity { path } => {
                println!("  Found Google Antigravity at: {}", path.display());
            }
            DetectedSurface::Cursor { path } => {
                println!("  Found Cursor IDE at: {}", path.display());
            }
            DetectedSurface::VSCode => {
                println!("  Found VS Code");
            }
        }
    }

    println!();
    for surface in &surfaces {
        match surface {
            DetectedSurface::ClaudeCode { .. } => setup_claude_hook()?,
            DetectedSurface::Codex { .. } => {
                println!("  Codex CLI: configuring MCP proxy...");
                crate::cli::codex::install_mcp_hook()?;
            }
            DetectedSurface::Antigravity { .. } => {
                crate::cli::antigravity::run_setup()?;
            }
            DetectedSurface::Cursor { .. } => {
                crate::cli::cursor::run_setup()?;
            }
            DetectedSurface::VSCode => {
                println!("  VS Code: run `onus setup vscode` for VS Code integration.");
            }
        }
    }

    println!();
    println!("Setup complete. Run `onus doctor` to verify.");
    Ok(())
}

pub fn run_claude() -> anyhow::Result<()> {
    println!("Onus Setup — Claude Code CLI\n");
    setup_claude_hook()?;
    println!("\nClaude Code hook setup complete. Run `onus doctor --claude` to verify.");
    Ok(())
}

pub fn run_uninstall() -> anyhow::Result<()> {
    println!("Onus Uninstall\n");

    remove_claude_hook()?;

    println!("\nUninstall complete.");
    Ok(())
}

pub fn run_uninstall_claude() -> anyhow::Result<()> {
    println!("Onus Uninstall — Claude Code CLI\n");
    remove_claude_hook()?;
    println!("\nClaude Code hook removed.");
    Ok(())
}

// ── Surface detection ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum DetectedSurface {
    ClaudeCode { path: PathBuf },
    Codex { path: PathBuf },
    Antigravity { path: PathBuf },
    Cursor { path: PathBuf },
    VSCode,
}

pub fn detect_surfaces() -> Vec<DetectedSurface> {
    let mut surfaces = Vec::new();

    // Check for Claude Code CLI
    if let Some(path) = find_claude_on_path() {
        surfaces.push(DetectedSurface::ClaudeCode { path });
    }

    // Check for VS Code `code` CLI
    if let Ok(output) = std::process::Command::new("code").arg("--version").output() {
        if output.status.success() {
            surfaces.push(DetectedSurface::VSCode);
        }
    }

    // Check for Google Antigravity
    if let crate::cli::antigravity::AntigravityCheck::Available { path, .. } =
        crate::cli::antigravity::find_antigravity()
    {
        surfaces.push(DetectedSurface::Antigravity { path });
    }

    // Check for Cursor IDE
    if let crate::cli::cursor::CursorCheck::Available { path, .. } =
        crate::cli::cursor::find_cursor()
    {
        surfaces.push(DetectedSurface::Cursor { path });
    }

    surfaces
}

fn find_claude_on_path() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join("claude");
            if candidate.is_file() {
                return Some(candidate);
            }
            #[cfg(windows)]
            {
                let candidate_exe = dir.join("claude.exe");
                if candidate_exe.is_file() {
                    return Some(candidate_exe);
                }
                let candidate_cmd = dir.join("claude.cmd");
                if candidate_cmd.is_file() {
                    return Some(candidate_cmd);
                }
                let candidate_ps1 = dir.join("claude.ps1");
                if candidate_ps1.is_file() {
                    return Some(candidate_ps1);
                }
            }
        }
        None
    })
}

// ── Claude Code hook setup ────────────────────────────────────────────────────

fn claude_config_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".claude").join("settings.json")
}

fn legacy_claude_config_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".claude").join("claude.json")
}

fn shell_quote_path(path: &std::path::Path) -> String {
    format!(
        "\"{}\"",
        path.to_string_lossy()
            .replace('\\', "/")
            .replace('"', "\\\"")
    )
}

fn onus_claude_command(onus_path: &std::path::Path) -> String {
    format!(
        "{} claude-hook --disabled-behavior deny --timeout-ms 10000",
        shell_quote_path(onus_path)
    )
}

fn official_claude_hook_group(command: String) -> serde_json::Value {
    serde_json::json!({
        "matcher": "Bash|Write|Edit|MultiEdit|NotebookEdit|Read|Glob|Grep|LS|WebFetch|WebSearch|Task|mcp__.*",
        "hooks": [
            {
                "type": "command",
                "command": command,
                "timeout": 30
            }
        ]
    })
}

fn command_has_onus_claude_hook(command: &str) -> bool {
    command.contains("onus") && command.contains("claude-hook")
}

fn config_has_onus_claude_hook(config: &serde_json::Value) -> bool {
    config
        .get("hooks")
        .and_then(|hooks| hooks.get("PreToolUse"))
        .and_then(|groups| groups.as_array())
        .map(|groups| {
            groups.iter().any(|group| {
                group
                    .get("hooks")
                    .and_then(|handlers| handlers.as_array())
                    .map(|handlers| {
                        handlers.iter().any(|handler| {
                            handler
                                .get("command")
                                .and_then(|command| command.as_str())
                                .map(command_has_onus_claude_hook)
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn remove_onus_claude_hook_groups(config: &mut serde_json::Value) -> usize {
    let Some(groups) = config
        .get_mut("hooks")
        .and_then(|hooks| hooks.get_mut("PreToolUse"))
        .and_then(|groups| groups.as_array_mut())
    else {
        return 0;
    };

    let before = groups.len();
    groups.retain(|group| {
        !group
            .get("hooks")
            .and_then(|handlers| handlers.as_array())
            .map(|handlers| {
                handlers.iter().any(|handler| {
                    handler
                        .get("command")
                        .and_then(|command| command.as_str())
                        .map(command_has_onus_claude_hook)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });
    before - groups.len()
}

fn setup_claude_hook() -> anyhow::Result<()> {
    let config_path = claude_config_path();
    let onus_path = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Cannot determine onus binary path: {}", e))?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Cannot create {}: {}", parent.display(), e))?;
    }

    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", config_path.display(), e))?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if config_has_onus_claude_hook(&config) {
        println!("  Onus hook already registered in settings.json");
        return Ok(());
    }

    if !config.get("hooks").map(|h| h.is_object()).unwrap_or(false) {
        config["hooks"] = serde_json::json!({});
    }
    if !config["hooks"]
        .get("PreToolUse")
        .map(|h| h.is_array())
        .unwrap_or(false)
    {
        config["hooks"]["PreToolUse"] = serde_json::json!([]);
    }

    config["hooks"]["PreToolUse"]
        .as_array_mut()
        .expect("PreToolUse initialized as array")
        .push(official_claude_hook_group(onus_claude_command(&onus_path)));

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| anyhow::anyhow!("Cannot serialize config: {}", e))?;
    std::fs::write(&config_path, content)
        .map_err(|e| anyhow::anyhow!("Cannot write {}: {}", config_path.display(), e))?;

    println!("  Hook written to: {}", config_path.display());
    println!("  Event: PreToolUse");
    println!("  Command: onus claude-hook --disabled-behavior deny --timeout-ms 10000");
    println!("  Enforcement: L1 BEST-EFFORT (Claude Code hooks can be disabled outside Onus)");
    println!();
    println!("  To verify: run `onus doctor --claude`");
    println!("  To remove: run `onus uninstall --claude`");

    Ok(())
}
// ── Claude Code hook removal ──────────────────────────────────────────────────

fn remove_claude_hook() -> anyhow::Result<()> {
    let config_path = claude_config_path();

    if !config_path.exists() {
        println!("  No Claude settings found at: {}", config_path.display());
    } else {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", config_path.display(), e))?;
        let mut config: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Cannot parse {}: {}", config_path.display(), e))?;

        let removed = remove_onus_claude_hook_groups(&mut config);
        if removed == 0 {
            println!("  No Onus PreToolUse hook found in settings.json");
        } else {
            println!(
                "  Removed Onus PreToolUse hook from settings.json (removed {})",
                removed
            );
            let content = serde_json::to_string_pretty(&config)
                .map_err(|e| anyhow::anyhow!("Cannot serialize config: {}", e))?;
            std::fs::write(&config_path, content)
                .map_err(|e| anyhow::anyhow!("Cannot write {}: {}", config_path.display(), e))?;
        }
    }

    let legacy_path = legacy_claude_config_path();
    if legacy_path.exists() {
        let content = std::fs::read_to_string(&legacy_path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", legacy_path.display(), e))?;
        let mut config: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Cannot parse {}: {}", legacy_path.display(), e))?;
        if let Some(hooks) = config.get_mut("hooks").and_then(|h| h.as_array_mut()) {
            let before = hooks.len();
            hooks.retain(|h| {
                !h.get("command")
                    .and_then(|c| c.as_str())
                    .map(command_has_onus_claude_hook)
                    .unwrap_or(false)
            });
            let removed = before - hooks.len();
            if removed > 0 {
                println!(
                    "  Removed legacy Onus hook from claude.json (removed {})",
                    removed
                );
                let content = serde_json::to_string_pretty(&config)
                    .map_err(|e| anyhow::anyhow!("Cannot serialize config: {}", e))?;
                std::fs::write(&legacy_path, content)
                    .map_err(|e| anyhow::anyhow!("Cannot write {}: {}", legacy_path.display(), e))?;
            }
        }
    }

    Ok(())
}
// ── Help text ─────────────────────────────────────────────────────────────────

pub fn help_text() -> String {
    r#"onus setup     — auto-detect surfaces and install hooks
onus setup --claude — install Onus hook for Claude Code CLI
onus setup vscode — install Onus VS Code extension (TBD)

onus uninstall     — remove all Onus hooks
onus uninstall --claude — remove Claude Code hook only"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_config_path_uses_official_settings_json() {
        let path = claude_config_path();
        assert!(path.to_string_lossy().contains(".claude"));
        assert!(path.to_string_lossy().ends_with("settings.json"));
    }

    #[test]
    fn official_claude_hook_group_matches_pre_tool_use_shape() {
        let group = official_claude_hook_group("\"/opt/onus/bin/onus\" claude-hook".to_string());
        assert_eq!(group["matcher"], "Bash|Write|Edit|MultiEdit|NotebookEdit|Read|Glob|Grep|LS|WebFetch|WebSearch|Task|mcp__.*");
        let handler = &group["hooks"][0];
        assert_eq!(handler["type"], "command");
        assert_eq!(handler["command"], "\"/opt/onus/bin/onus\" claude-hook");
        assert_eq!(handler["timeout"], 30);
    }

    #[test]
    fn detects_official_onus_claude_hook() {
        let config = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    official_claude_hook_group("\"/opt/onus/bin/onus\" claude-hook".to_string())
                ]
            }
        });
        assert!(config_has_onus_claude_hook(&config));
    }

    #[test]
    fn removes_only_onus_claude_hook_group() {
        let mut config = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    official_claude_hook_group("\"/opt/onus/bin/onus\" claude-hook".to_string()),
                    {
                        "matcher": "Bash",
                        "hooks": [
                            {"type": "command", "command": "echo keep", "timeout": 30}
                        ]
                    }
                ]
            }
        });
        assert_eq!(remove_onus_claude_hook_groups(&mut config), 1);
        assert_eq!(config["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
        assert_eq!(
            config["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "echo keep"
        );
    }
}
