//! `onus uninstall` — remove Onus configuration, hooks, and optionally the binary.

use clap::Args;

#[derive(Args)]
pub struct UninstallArgs {
    /// Also delete the audit trail, rules, and config directory
    #[arg(long)]
    pub purge: bool,

    /// Remove only the Claude Code CLI hook
    #[arg(long)]
    pub claude: bool,

    /// Remove only the OpenAI Codex CLI MCP proxy entry
    #[arg(long)]
    pub codex: bool,

    /// Remove only the Google Antigravity extension and MCP config
    #[arg(long)]
    pub antigravity: bool,
}

/// Run the uninstall command.
pub fn run(args: UninstallArgs) -> anyhow::Result<()> {
    if args.claude {
        return crate::cli::setup::run_uninstall_claude();
    }
    if args.codex {
        return crate::cli::codex::uninstall_mcp_hook();
    }
    if args.antigravity {
        return crate::cli::antigravity::run_uninstall();
    }

    let data_dir = crate::data_dir();
    let config_dir = crate::config_dir();

    // 1. Stop the daemon if running.
    if crate::daemon::is_running() {
        eprintln!("Stopping daemon...");
        crate::daemon::stop_daemon()?;
    }

    // 2. Remove Claude Code hook.
    let claude_settings = dirs_data_dir().map(|d| d.join("Claude Code").join("settings.json"));
    if let Some(settings_path) = claude_settings {
        if settings_path.exists() {
            eprintln!("Removing Claude Code preToolUse hook...");
            remove_hook(&settings_path)?;
        }
    }

    // 3. Remove the config directory (or just rules if keeping config).
    if args.purge {
        eprintln!("Removing all configuration and data...");
        let _ = std::fs::remove_dir_all(&config_dir);
        let _ = std::fs::remove_dir_all(&data_dir);
    } else {
        // Remove PID file if exists.
        let pid_file = config_dir.join("onus.pid");
        let _ = std::fs::remove_file(pid_file);
        eprintln!("Configuration and audit trail preserved at:");
        eprintln!("  Config: {}", config_dir.display());
        eprintln!("  Data:   {}", data_dir.display());
        eprintln!("Use `onus uninstall --purge` to delete them.");
    }

    eprintln!();
    eprintln!("Onus has been removed.");
    if !args.purge {
        eprintln!("To reinstall: npx @onus/install");
    }
    Ok(())
}

/// Remove the preToolUse hook from Claude Code's settings.json.
fn remove_hook(settings_path: &std::path::Path) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(settings_path)?;
    let mut json: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(obj) = json.as_object_mut() {
        if let Some(hooks) = obj.get_mut("hooks") {
            if let Some(h) = hooks.as_object_mut() {
                if h.remove("preToolUse").is_some() {
                    // Also remove hooks if empty.
                    if h.is_empty() {
                        obj.remove("hooks");
                    }
                    let new_content = serde_json::to_string_pretty(&json)?;
                    std::fs::write(settings_path, new_content)?;
                    eprintln!("  Removed preToolUse hook from {}", settings_path.display());
                }
            }
        }
    }
    Ok(())
}

/// Get the platform data directory (e.g. %APPDATA% on Windows).
fn dirs_data_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(std::path::PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(std::path::PathBuf::from)
            .or_else(|| Some(std::path::PathBuf::from("/usr/local/share")))
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(std::path::PathBuf::from)
            .or_else(|| Some(std::path::PathBuf::from("/usr/local/share")))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}
