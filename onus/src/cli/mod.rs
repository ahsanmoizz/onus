pub mod approvals;
pub mod daemon_cmd;
pub mod dashboard;
pub mod evaluate;
pub mod log_cmd;
pub mod rules;
pub mod session;
pub mod shell;
pub mod status;
pub mod upgrade;
pub mod uninstall;
pub mod verify;

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
pub enum Commands {
    /// Serve the local approval UI for pending escalations
    Approvals(approvals::ApprovalsArgs),

    /// Evaluate a single action and return a verdict (used by Claude Code preToolUse hook)
    Evaluate(evaluate::EvaluateArgs),

    /// Start, stop, or check the Onus daemon
    Daemon(daemon_cmd::DaemonArgs),

    /// Serve a local dashboard backed by the audit database
    Dashboard(dashboard::DashboardArgs),

    /// Show daemon and session status
    Status,

    /// View the audit trail
    Log(log_cmd::LogArgs),

    /// View a specific session summary
    Session(session::SessionArgs),

    /// Manage safety rules
    Rules(rules::RulesArgs),

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
}

pub mod mcp_proxy;
