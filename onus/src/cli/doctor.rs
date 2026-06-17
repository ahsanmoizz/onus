//! `onus doctor` — system health and integration surface diagnostics.
//!
//! Checks daemon health, rule-engine readiness, capability levels,
//! and integration-surface availability (Claude Code, VS Code, CLI, etc.).

use std::path::PathBuf;
use clap::Args;

#[derive(Args)]
pub struct DoctorArgs {
    /// Run only Claude Code CLI checks
    #[arg(long)]
    pub claude: bool,

    /// Run only OpenAI Codex CLI checks
    #[arg(long)]
    pub codex: bool,

    /// Run only Google Antigravity checks
    #[arg(long)]
    pub antigravity: bool,

    /// Run only Cursor IDE checks
    #[arg(long)]
    pub cursor: bool,
}

/// Run the main `onus doctor` command — check everything.
pub fn run(args: DoctorArgs) -> anyhow::Result<()> {
    if args.claude {
        return run_claude();
    }
    if args.codex {
        return run_codex();
    }
    if args.antigravity {
        return run_antigravity();
    }
    if args.cursor {
        return run_cursor();
    }
    let mut ok_count = 0u32;
    let mut warn_count = 0u32;
    let mut fail_count = 0u32;

    println!("Onus Doctor v{}", env!("CARGO_PKG_VERSION"));
    println!("Checking system health and surface readiness…\n");

    // ── Daemon ─────────────────────────────────────────────────────────────
    let daemon_running = crate::daemon::is_running();
    let daemon_pid = crate::daemon::get_pid();
    if daemon_running {
        log_ok("Daemon", format!("RUNNING (PID {})", daemon_pid.unwrap_or(0)));
        ok_count += 1;
    } else {
        log_warn("Daemon", format!("NOT RUNNING — some features require the daemon"));
        warn_count += 1;
    }

    // ── Policy Engine ──────────────────────────────────────────────────────
    let rules_path = crate::config_dir().join("rules").join("default.toml");
    if rules_path.exists() {
        let rules_count = count_rules(&rules_path);
        log_ok("Rule engine", format!("{} rules loaded from {}", rules_count, rules_path.display()));
        ok_count += 1;
    } else {
        log_warn("Rule engine", format!("No rules file at {}", rules_path.display()));
        warn_count += 1;
    }

    // ── Claude Code CLI ────────────────────────────────────────────────────
    match check_claude_cli() {
        ClaudeCliCheck::Available { version, path, tool_use_version } => {
            log_ok("Claude Code CLI", format!("v{} at {}", version, path.display()));
            if let Some(tuv) = tool_use_version {
                log_ok("Claude Code tool-use", format!("protocol v{}", tuv));
            }
            ok_count += 1;

            // Check hook installation
            match check_claude_hook_installed() {
                ClaudeHookCheck::Installed { hook_path, mode } => {
                    log_ok("Claude Code hook", format!("installed at {} ({})", hook_path.display(), mode));
                    ok_count += 1;
                }
                ClaudeHookCheck::NotInstalled => {
                    log_warn("Claude Code hook", "not installed — run `onus setup claude`".to_string());
                    warn_count += 1;
                }
                ClaudeHookCheck::Error(e) => {
                    log_warn("Claude Code hook", format!("check failed: {}", e));
                    warn_count += 1;
                }
            }
        }
        ClaudeCliCheck::NotFound => {
            log_fail("Claude Code CLI", "not found on PATH".to_string());
            fail_count += 1;
        }
        ClaudeCliCheck::Error(e) => {
            log_fail("Claude Code CLI", format!("error: {}", e));
            fail_count += 1;
        }
    }

    // ── OpenAI Codex CLI ─────────────────────────────────────────────────
    match crate::cli::codex::find_codex_cli() {
        crate::cli::codex::CodexCliCheck::Available { version, path } => {
            log_ok("OpenAI Codex CLI", format!("v{} at {}", version, path.display()));
            ok_count += 1;

            // Check MCP config
            match crate::cli::codex::check_mcp_config() {
                crate::cli::codex::McpConfigCheck::Configured { server_name, mode } => {
                    log_ok("Codex MCP config", format!("server '{}' ({})", server_name, mode));
                    ok_count += 1;
                }
                crate::cli::codex::McpConfigCheck::NotFound => {
                    log_warn("Codex MCP config", "Onus not configured — run `onus setup --codex`".to_string());
                    warn_count += 1;
                }
                crate::cli::codex::McpConfigCheck::Error(e) => {
                    log_warn("Codex MCP config", format!("check error: {}", e));
                    warn_count += 1;
                }
            }
        }
        crate::cli::codex::CodexCliCheck::NotFound => {
            // Not installed — not a failure, just informational
            log_ok("OpenAI Codex CLI", "not installed".to_string());
            ok_count += 1;
        }
    }

    // ── Google Antigravity ─────────────────────────────────────────────────
    match crate::cli::antigravity::find_antigravity() {
        crate::cli::antigravity::AntigravityCheck::Available { version, path } => {
            log_ok("Google Antigravity", format!("v{} at {}", version, path.display()));
            ok_count += 1;

            // Check extension
            match crate::cli::antigravity::check_extension_installed(&path) {
                crate::cli::antigravity::ExtensionCheck::Installed { path: ext_path, version: ext_ver } => {
                    log_ok("Antigravity extension", format!("onus-firewall v{} at {}", ext_ver, ext_path.display()));
                    ok_count += 1;
                }
                crate::cli::antigravity::ExtensionCheck::NotInstalled => {
                    log_warn("Antigravity extension", "onus-firewall not installed".to_string());
                    warn_count += 1;
                }
                crate::cli::antigravity::ExtensionCheck::Error(e) => {
                    log_warn("Antigravity extension", format!("check error: {}", e));
                    warn_count += 1;
                }
            }

            // Check MCP config
            match crate::cli::antigravity::check_mcp_config(&path) {
                crate::cli::antigravity::McpConfigCheck::Configured { server_name } => {
                    log_ok("Antigravity MCP proxy", format!("'{}' configured", server_name));
                    ok_count += 1;
                }
                crate::cli::antigravity::McpConfigCheck::NotFound => {
                    log_warn("Antigravity MCP proxy", "not configured".to_string());
                    warn_count += 1;
                }
                crate::cli::antigravity::McpConfigCheck::Error(e) => {
                    log_warn("Antigravity MCP proxy", format!("check error: {}", e));
                    warn_count += 1;
                }
            }

            // L3 advice
            let l3 = crate::cli::antigravity::l3_workspace_advice();
            if !l3.is_empty() {
                log_ok("Antigravity L3 workspace", l3);
                ok_count += 1;
            }
        }
        crate::cli::antigravity::AntigravityCheck::NotFound => {
            log_ok("Google Antigravity", "not installed".to_string());
            ok_count += 1;
        }
        crate::cli::antigravity::AntigravityCheck::Error(e) => {
            log_fail("Google Antigravity", format!("check error: {}", e));
            fail_count += 1;
        }
    }

    // ── Cursor IDE ─────────────────────────────────────────────────────────
    match crate::cli::cursor::find_cursor() {
        crate::cli::cursor::CursorCheck::Available { version, path } => {
            log_ok("Cursor IDE", format!("v{} at {}", version, path.display()));
            ok_count += 1;

            match crate::cli::cursor::check_hook_installed() {
                crate::cli::cursor::HookCheck::Installed { command } => {
                    log_ok("Cursor hook", command);
                    ok_count += 1;
                }
                crate::cli::cursor::HookCheck::NotInstalled => {
                    log_warn("Cursor hook", "not configured".to_string());
                    warn_count += 1;
                }
                crate::cli::cursor::HookCheck::Error(e) => {
                    log_warn("Cursor hook", format!("check error: {}", e));
                    warn_count += 1;
                }
            }

            match crate::cli::cursor::check_mcp_configured() {
                crate::cli::cursor::McpConfigCheck::Configured { server_name } => {
                    log_ok("Cursor MCP proxy", format!("'{}' configured", server_name));
                    ok_count += 1;
                }
                crate::cli::cursor::McpConfigCheck::NotFound => {
                    log_warn("Cursor MCP proxy", "not configured".to_string());
                    warn_count += 1;
                }
                crate::cli::cursor::McpConfigCheck::Error(e) => {
                    log_warn("Cursor MCP proxy", format!("check error: {}", e));
                    warn_count += 1;
                }
            }

            let l3 = crate::cli::cursor::l3_workspace_advice();
            if !l3.is_empty() {
                log_ok("Cursor L3 workspace", l3);
                ok_count += 1;
            }
        }
        crate::cli::cursor::CursorCheck::NotFound => {
            log_ok("Cursor IDE", "not installed".to_string());
            ok_count += 1;
        }
        crate::cli::cursor::CursorCheck::Error(e) => {
            log_fail("Cursor IDE", format!("check error: {}", e));
            fail_count += 1;
        }
    }

    // ── L3 Workspace Isolation (Linux only) ────────────────────────────────
    #[cfg(target_os = "linux")]
    check_l3_isolation(&mut ok_count, &mut warn_count, &mut fail_count);

    #[cfg(not(target_os = "linux"))]
    {
        log_warn("L3 isolation", "not available on this platform (requires Linux + bubblewrap)".to_string());
        warn_count += 1;
    }

    // ── Audit trail ────────────────────────────────────────────────────────
    let db_path = crate::data_dir().join("audit.db");
    if db_path.exists() {
        let size_kb = std::fs::metadata(&db_path).map(|m| m.len() / 1024).unwrap_or(0);
        let status_str = audit_status_string(&db_path);
        log_ok("Audit trail", format!("{} ({size_kb} KB) at {}", status_str, db_path.display()));
        ok_count += 1;
    } else {
        log_ok("Audit trail", "empty (no actions evaluated yet)".to_string());
        ok_count += 1;
    }

    // ── Summary ────────────────────────────────────────────────────────────
    println!("\n─── Results ──────────────────────────────────");
    println!("  {}  OK", ok_count);
    if warn_count > 0 {
        println!("  {}  Warning{}", warn_count, if warn_count == 1 { "" } else { "s" });
    }
    if fail_count > 0 {
        println!("  {}  Failure{}", fail_count, if fail_count == 1 { "" } else { "s" });
    }

    if fail_count > 0 {
        anyhow::bail!("{} surface{} not ready", fail_count, if fail_count == 1 { "" } else { "s" });
    }
    Ok(())
}

/// Run only the Claude Code CLI checks.
pub fn run_claude() -> anyhow::Result<()> {
    println!("Onus Doctor — Claude Code CLI\n");

    match check_claude_cli() {
        ClaudeCliCheck::Available { version, path, tool_use_version } => {
            log_ok("Binary found", format!("Claude Code CLI v{} at {}", version, path.display()));
            if let Some(tuv) = tool_use_version {
                log_ok("Tool-use protocol", format!("v{}", tuv));
            }

            match check_claude_hook_installed() {
                ClaudeHookCheck::Installed { hook_path, mode } => {
                    log_ok("Hook installed", format!("{} ({})", hook_path.display(), mode));
                    match check_hook_works() {
                        HookHealth::Ok => log_ok("Hook works", "onus claude-hook responds correctly".to_string()),
                        HookHealth::Error(e) => log_fail("Hook test", format!("failed: {}", e)),
                    }
                }
                ClaudeHookCheck::NotInstalled => {
                    log_warn("Hook", "not installed. Run `onus setup claude`".to_string());
                }
                ClaudeHookCheck::Error(e) => {
                    log_fail("Hook check", format!("error: {}", e));
                }
            }
        }
        ClaudeCliCheck::NotFound => {
            log_fail("Binary", "Claude Code CLI not found on PATH".to_string());
            println!("\n  Install it with:");
            println!("    npm install -g @anthropic-ai/claude-code");
            println!("  Or visit: https://docs.anthropic.com/en/docs/claude-code");
        }
        ClaudeCliCheck::Error(e) => {
            log_fail("Binary", format!("check error: {}", e));
        }
    }

    Ok(())
}

/// Run only the OpenAI Codex CLI checks.
pub fn run_codex() -> anyhow::Result<()> {
    println!("Onus Doctor — OpenAI Codex CLI\n");

    match crate::cli::codex::find_codex_cli() {
        crate::cli::codex::CodexCliCheck::Available { version, path } => {
            log_ok("Binary found", format!("Codex CLI v{} at {}", version, path.display()));

            match crate::cli::codex::check_mcp_config() {
                crate::cli::codex::McpConfigCheck::Configured { server_name, mode } => {
                    log_ok("MCP config", format!("server '{}' ({})", server_name, mode));
                }
                crate::cli::codex::McpConfigCheck::NotFound => {
                    log_warn("MCP config", "Onus MCP proxy not configured. Run `onus setup --codex`".to_string());
                    println!("\n  Run `onus setup --codex` to configure the MCP proxy.");
                }
                crate::cli::codex::McpConfigCheck::Error(e) => {
                    log_fail("MCP config", format!("error: {}", e));
                }
            }

            // L3 workspace advice
            let l3 = crate::cli::codex::l3_workspace_advice();
            if !l3.is_empty() {
                println!();
                log_ok("L3 workspace", l3);
            }
        }
        crate::cli::codex::CodexCliCheck::NotFound => {
            log_fail("Binary", "Codex CLI not found on PATH".to_string());
            println!("\n  Install it with:");
            println!("    pip install openai-codex");
            println!("  Or visit: https://developers.openai.com/codex");
        }
    }

    Ok(())
}

/// Run only the Google Antigravity checks.
pub fn run_antigravity() -> anyhow::Result<()> {
    println!("Onus Doctor — Google Antigravity\n");

    match crate::cli::antigravity::find_antigravity() {
        crate::cli::antigravity::AntigravityCheck::Available { version, path } => {
            log_ok("Binary found", format!("Antigravity v{} at {}", version, path.display()));

            match crate::cli::antigravity::check_extension_installed(&path) {
                crate::cli::antigravity::ExtensionCheck::Installed { path: ext_path, version: ext_ver } => {
                    log_ok("Extension", format!("onus-firewall v{} at {}", ext_ver, ext_path.display()));
                }
                crate::cli::antigravity::ExtensionCheck::NotInstalled => {
                    log_warn("Extension", "onus-firewall not installed".to_string());
                    println!("\n  Run `onus setup --antigravity` for installation instructions.");
                }
                crate::cli::antigravity::ExtensionCheck::Error(e) => {
                    log_fail("Extension check", format!("error: {}", e));
                }
            }

            match crate::cli::antigravity::check_mcp_config(&path) {
                crate::cli::antigravity::McpConfigCheck::Configured { server_name } => {
                    log_ok("MCP proxy", format!("'{}' configured", server_name));
                }
                crate::cli::antigravity::McpConfigCheck::NotFound => {
                    log_warn("MCP proxy", "Onus not configured as MCP server".to_string());
                }
                crate::cli::antigravity::McpConfigCheck::Error(e) => {
                    log_fail("MCP config", format!("error: {}", e));
                }
            }

            // L3 workspace advice
            let l3 = crate::cli::antigravity::l3_workspace_advice();
            if !l3.is_empty() {
                println!();
                log_ok("L3 workspace", l3);
            }
        }
        crate::cli::antigravity::AntigravityCheck::NotFound => {
            log_fail("Binary", "Antigravity not found on PATH".to_string());
            println!("\n  Install it from: https://github.com/google/antigravity");
        }
        crate::cli::antigravity::AntigravityCheck::Error(e) => {
            log_fail("Antigravity", format!("check error: {}", e));
        }
    }

    Ok(())
}

/// Run only the Cursor IDE checks.
pub fn run_cursor() -> anyhow::Result<()> {
    println!("Onus Doctor — Cursor IDE\n");

    match crate::cli::cursor::find_cursor() {
        crate::cli::cursor::CursorCheck::Available { version, path } => {
            log_ok("Binary found", format!("Cursor v{} at {}", version, path.display()));

            match crate::cli::cursor::check_hook_installed() {
                crate::cli::cursor::HookCheck::Installed { command } => {
                    log_ok("PreToolUse hook", command);
                }
                crate::cli::cursor::HookCheck::NotInstalled => {
                    log_warn("PreToolUse hook", "not configured".to_string());
                    println!("\n  Run `onus setup --cursor` to install.");
                }
                crate::cli::cursor::HookCheck::Error(e) => {
                    log_fail("PreToolUse hook", format!("error: {}", e));
                }
            }

            match crate::cli::cursor::check_mcp_configured() {
                crate::cli::cursor::McpConfigCheck::Configured { server_name } => {
                    log_ok("MCP server", format!("'{}' configured", server_name));
                }
                crate::cli::cursor::McpConfigCheck::NotFound => {
                    log_warn("MCP server", "not configured".to_string());
                    println!("\n  Run `onus setup --cursor` to configure.");
                }
                crate::cli::cursor::McpConfigCheck::Error(e) => {
                    log_fail("MCP server", format!("error: {}", e));
                }
            }

            let l3 = crate::cli::cursor::l3_workspace_advice();
            if !l3.is_empty() {
                println!();
                log_ok("L3 workspace", l3);
            }
        }
        crate::cli::cursor::CursorCheck::NotFound => {
            log_fail("Binary", "Cursor not found on PATH".to_string());
            println!("\n  Install Cursor:");
            println!("    https://cursor.com/downloads");
        }
        crate::cli::cursor::CursorCheck::Error(e) => {
            log_fail("Cursor", format!("check error: {}", e));
        }
    }

    Ok(())
}

// ── Claude Code CLI detection ─────────────────────────────────────────────────

#[allow(dead_code)]
enum ClaudeCliCheck {
    Available { version: String, path: PathBuf, tool_use_version: Option<String> },
    NotFound,
    Error(String),
}

fn check_claude_cli() -> ClaudeCliCheck {
    // Try `claude --version`
    let output = match std::process::Command::new("claude")
        .arg("--version")
        .output()
    {
        Ok(o) => o,
        Err(_e) => {
            // Try npx fallback
            return match std::process::Command::new("npx.cmd")
                .args(["--yes", "@anthropic-ai/claude-code", "--version"])
                .output()
            {
                Ok(o) if o.status.success() => {
                    let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    let path = find_claude_on_path().unwrap_or_else(|| PathBuf::from("npx"));
                    ClaudeCliCheck::Available { version: ver, path, tool_use_version: None }
                }
                _ => ClaudeCliCheck::NotFound,
            };
        }
    };

    if !output.status.success() {
        return ClaudeCliCheck::NotFound;
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let path = find_claude_on_path().unwrap_or_else(|| PathBuf::from("claude"));

    // Try to get tool-use protocol version from `claude.json` schema
    let tool_use_version = get_tool_use_version();

    ClaudeCliCheck::Available { version, path, tool_use_version }
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
            }
        }
        None
    })
}

fn get_tool_use_version() -> Option<String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    let claude_config = PathBuf::from(home).join(".claude").join("claude.json");
    if !claude_config.exists() {
        return None;
    }
    let content = std::fs::read_to_string(claude_config).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("tool_use_version")
        .or_else(|| json.get("hooks").and_then(|h| h.get("version")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

// ── Hook installation check ───────────────────────────────────────────────────

enum ClaudeHookCheck {
    Installed { hook_path: PathBuf, mode: String },
    NotInstalled,
    Error(String),
}

fn claude_config_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();
    PathBuf::from(home).join(".claude").join("claude.json")
}

fn check_claude_hook_installed() -> ClaudeHookCheck {
    let claude_config = claude_config_path();

    if !claude_config.exists() {
        return ClaudeHookCheck::NotInstalled;
    }

    let content = match std::fs::read_to_string(&claude_config) {
        Ok(c) => c,
        Err(e) => return ClaudeHookCheck::Error(format!("Cannot read {}: {}", claude_config.display(), e)),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(e) => return ClaudeHookCheck::Error(format!("Cannot parse {}: {}", claude_config.display(), e)),
    };

    let hooks = match json.get("hooks") {
        Some(h) => h,
        None => return ClaudeHookCheck::NotInstalled,
    };

    let hook_arr = match hooks.as_array() {
        Some(a) => a,
        None => return ClaudeHookCheck::NotInstalled,
    };

    for hook in hook_arr {
        if let Some(cmd) = hook.get("command").and_then(|c| c.as_str()) {
            if cmd.contains("onus") && cmd.contains("claude-hook") {
                let hook_path = PathBuf::from(cmd.split_whitespace().next().unwrap_or(cmd));
                let mode = hook.get("mode").and_then(|m| m.as_str()).unwrap_or("best_effort");
                return ClaudeHookCheck::Installed { hook_path, mode: mode.to_string() };
            }
        }
    }

    ClaudeHookCheck::NotInstalled
}

// ── Hook health test ──────────────────────────────────────────────────────────

enum HookHealth {
    Ok,
    Error(String),
}

fn check_hook_works() -> HookHealth {
    let onus_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("onus"));
    let test_payload = r#"{"tool":"Bash","input":{"command":"echo test"},"session_id":"doctor-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}"#;

    let mut cmd = std::process::Command::new(&onus_path);
    cmd.arg("claude-hook");
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return HookHealth::Error(format!("Cannot spawn onus claude-hook: {}", e)),
    };

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(test_payload.as_bytes());
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return HookHealth::Error(format!("Hook process error: {}", e)),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return HookHealth::Error(format!("Hook exited with {}: {}", output.status, stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.is_empty() {
        return HookHealth::Error("Hook produced no output".to_string());
    }

    match serde_json::from_str::<serde_json::Value>(&stdout) {
        Ok(json) => {
            if json.get("decision").and_then(|d| d.as_str()).is_some() {
                HookHealth::Ok
            } else {
                HookHealth::Error("Hook output missing 'decision' field".to_string())
            }
        }
        Err(e) => HookHealth::Error(format!("Hook output is not valid JSON: {}", e)),
    }
}

// ── L3 isolation check ────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn check_l3_isolation(ok_count: &mut u32, warn_count: &mut u32, fail_count: &mut u32) {
    let bwrap = std::process::Command::new("bwrap")
        .arg("--version")
        .output();

    match bwrap {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
            log_ok("L3 isolation (bubblewrap)", format!("{} available", ver));
            *ok_count += 1;
        }
        _ => {
            log_fail("L3 isolation (bubblewrap)", "bwrap not found — install bubblewrap for L3 containment".to_string());
            *fail_count += 1;
        }
    }
}

// ── Audit helpers ─────────────────────────────────────────────────────────────

fn count_rules(path: &PathBuf) -> usize {
    if let Ok(content) = std::fs::read_to_string(path) {
        content.matches("[[rule]]").count()
    } else {
        0
    }
}

fn audit_status_string(db_path: &PathBuf) -> String {
    match crate::audit::AuditTrail::open(db_path) {
        Ok(audit) => match audit.get_status() {
            Ok(status) => format!(
                "{} evaluated, {} blocked, {} escalated",
                status.total_actions, status.blocked_actions, status.escalated_actions
            ),
            Err(_) => "present (read error)".to_string(),
        },
        Err(_) => "present (open error)".to_string(),
    }
}

// ── Doctor format helpers ─────────────────────────────────────────────────────

fn log_ok(check: &str, msg: String) {
    println!("  [{OK}]  {check}: {msg}");
}
fn log_warn(check: &str, msg: String) {
    println!("  [{WARN}] {check}: {msg}");
}
fn log_fail(check: &str, msg: String) {
    println!("  [{FAIL}] {check}: {msg}");
}

const OK: &str = "\x1b[32mOK\x1b[0m";
const WARN: &str = "\x1b[33mWARN\x1b[0m";
const FAIL: &str = "\x1b[31mFAIL\x1b[0m";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_runs_without_panic() {
        let args = DoctorArgs { claude: false, codex: false, antigravity: false, cursor: false };
        let _ = run(args);
        let _ = run_claude();
        let _ = run_codex();
    }

    #[test]
    fn test_claude_cli_check_format() {
        let result = check_claude_cli();
        match result {
            ClaudeCliCheck::Available { version, path, .. } => {
                assert!(!version.is_empty(), "version should not be empty");
                assert!(path.as_os_str().len() > 0, "path should not be empty");
            }
            ClaudeCliCheck::NotFound | ClaudeCliCheck::Error(_) => {
                // Acceptable in test environment
            }
        }
    }

    #[test]
    fn test_count_rules_empty_path() {
        let p = PathBuf::from("/nonexistent/rules.toml");
        assert_eq!(count_rules(&p), 0);
    }

    #[test]
    fn test_claude_config_path_format() {
        let path = claude_config_path();
        assert!(path.to_string_lossy().contains(".claude"));
        assert!(path.to_string_lossy().contains("claude.json"));
    }

    #[test]
    fn test_check_hook_works_no_onus() {
        // Should produce an error, not crash
        match check_hook_works() {
            HookHealth::Ok => {} // acceptable if onus is available
            HookHealth::Error(_) => {} // expected
        }
    }

    #[test]
    fn test_ok_warn_fail_functions() {
        // Verify helper functions compile and produce output
        log_ok("Test", "ok".to_string());
        log_warn("Test", "warn".to_string());
        log_fail("Test", "fail".to_string());
    }
}
