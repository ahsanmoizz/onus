//! CLI entry point and command routing.

pub mod approvals;
pub mod antigravity;
pub mod authority;
pub mod claude_hook;
pub mod codex;
pub mod contract;
pub mod cursor;
pub mod cursor_hook;
pub mod daemon_cmd;
pub mod dashboard;
pub mod doctor;
pub mod evaluate;
pub mod intake;
pub mod log_cmd;
pub mod memory;
pub mod recovery;
pub mod run_cmd;
pub mod rules;
pub mod session;
pub mod setup;
pub mod shell;
pub mod status;
pub mod uninstall;
pub mod upgrade;
pub mod verify;
pub mod workspace;
pub mod handoff;
pub mod lease_cli;

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

    /// Authority — request, approve, execute, or inspect controlled operations
    Authority(authority::AuthorityArgs),

    /// Claude Code tool-use hook
    ClaudeHook(claude_hook::ClaudeHookArgs),

    /// Evaluate a single action and return a verdict (used by Claude Code preToolUse hook)
    Evaluate(evaluate::EvaluateArgs),

    /// Start, stop, or check the Onus daemon
    Daemon(daemon_cmd::DaemonArgs),

    /// Create and manage task contracts
    Contract(contract::ContractArgs),

    /// Serve a local dashboard backed by the audit database
    Dashboard(dashboard::DashboardArgs),

    /// Prompt Intake Guardian — analyze an agentic task before execution
    Intake(intake::IntakeArgs),

    /// Show daemon and session status
    Status,

    /// View the audit trail
    Log(log_cmd::LogArgs),

    /// Run a command through Onus evaluation
    Run(run_cmd::RunArgs),

    /// View a specific session summary
    Session(session::SessionArgs),

    /// Manage safety rules
    Rules(rules::RulesArgs),

    /// Download and install the latest version of Onus
    Upgrade,

    /// Run system diagnostics
    Doctor(doctor::DoctorArgs),

    /// Set up and configure integrations
    Setup(setup::SetupArgs),

    /// Remove Onus and all its configuration, preserving audit trail (add --purge to delete all)
    Uninstall(uninstall::UninstallArgs),

    /// Run as an MCP proxy — intercepts MCP tool calls and evaluates through Onus Core
    McpProxy(mcp_proxy::McpProxyArgs),

    /// Install or remove the shell wrapper for terminal-based agents
    Shell(shell::ShellArgs),

    /// VS Code Cursor agent hook
    CursorHook(cursor_hook::CursorHookArgs),

    /// Verify hash chain integrity of the audit trail
    Verify(verify::VerifyArgs),

    /// Create, list, inspect, and restore workspace checkpoints
    Checkpoint(recovery::CheckpointArgs),

    /// Roll back individual actions, groups, or entire sessions
    Rollback(recovery::RollbackArgs),

    /// Inspect or execute compensation for previously evaluated actions
    Compensation(recovery::CompensationArgs),

    /// Containerized workspace management
    Workspace(workspace::WorkspaceArgs),

    /// Manage memory lifecycle operations
    Memory(memory::MemoryArgs),

    /// Create, import, and display cross-agent handoff manifests
    Handoff(handoff::HandoffArgs),

    /// Acquire, release, heartbeat, and manage session leases
    Lease(lease_cli::LeaseArgs),
}

pub mod mcp_proxy;
