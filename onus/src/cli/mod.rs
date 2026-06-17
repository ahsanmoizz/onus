pub mod antigravity;
pub mod approvals;
pub mod authority;
pub mod claude_hook;
pub mod codex;
pub mod contract;
pub mod daemon_cmd;
pub mod dashboard;
pub mod doctor;
pub mod evaluate;
pub mod intake;
pub mod log_cmd;
pub mod rules;
pub mod run_cmd;
pub mod session;
pub mod shell;
pub mod status;
pub mod uninstall;
pub mod setup;
pub mod upgrade;
pub mod verify;
pub mod workspace;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "onus")]
#[command(about = "AI Agent Firewall — intercepts, evaluates, and blocks dangerous agent actions")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// Serve the local approval UI for pending escalations
    Approvals(approvals::ApprovalsArgs),

    /// Manage narrow L4 broker-owned authority proofs
    Authority(authority::AuthorityArgs),

    /// Evaluate a single action and return a verdict (used by Claude Code preToolUse hook)
    Evaluate(evaluate::EvaluateArgs),

    /// Run as a Claude Code PreToolUse hook adapter (L1 BEST-EFFORT)
    ClaudeHook(claude_hook::ClaudeHookArgs),

    /// Start, stop, or check the Onus daemon
    Daemon(daemon_cmd::DaemonArgs),

    /// Manage task contracts for governed sessions
    Contract(contract::ContractArgs),

    /// Serve a local dashboard backed by the audit database
    Dashboard(dashboard::DashboardArgs),

    /// Inspect an original prompt and optionally start a governed session
    Intake(intake::IntakeArgs),

    /// Show daemon and session status
    Status,

    /// View the audit trail
    Log(log_cmd::LogArgs),

    /// Run an agent command inside an Onus-controlled workspace
    Run(run_cmd::RunArgs),

    /// View a specific session summary
    Session(session::SessionArgs),

    /// Manage safety rules
    Rules(rules::RulesArgs),

    /// Diagnose integration surface health
    Doctor(doctor::DoctorArgs),

    /// Install Onus hooks for integration surfaces (Claude Code, etc.)
    Setup(setup::SetupArgs),

    /// Download and install the latest version of Onus
    Upgrade,

    /// Remove Onus and all its configuration, preserving audit trail (add --purge to delete all)
    Uninstall(uninstall::UninstallArgs),

    /// Run as an MCP proxy — intercepts MCP tool calls and evaluates through Onus Core
    McpProxy(mcp_proxy::McpProxyArgs),

    /// Install or remove the shell wrapper for terminal-based agents
    Shell(shell::ShellArgs),

    /// Verify hash chain integrity of the audit trail
    Verify(verify::VerifyArgs),

    /// Create, inspect, export, or destroy an Onus L3 workspace
    Workspace(workspace::WorkspaceArgs),
}

pub mod mcp_proxy;
