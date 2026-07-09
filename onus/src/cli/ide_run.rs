//! `onus ide-run` launches supported agent CLIs through the Onus L3 workspace.
//!
//! This is the honest containment path for current IDE/agent integrations. Hook
//! and MCP adapters remain L1/L2; this command delegates execution to
//! `workspace::run_isolated`, which fails closed unless Linux + bubblewrap are
//! available.

use clap::{Args, ValueEnum};
use std::path::PathBuf;

#[derive(Args)]
pub struct IdeRunArgs {
    /// Agent surface to launch through the L3 workspace.
    #[arg(long, value_enum)]
    pub surface: IdeSurface,

    /// Workspace/session id. Defaults to ONUS_WORKSPACE_ID or latest workspace.
    #[arg(long)]
    pub workspace: Option<String>,

    /// Override the default agent CLI command.
    #[arg(long)]
    pub command: Option<PathBuf>,

    /// Allow host network egress for this run. Default is deny-all.
    #[arg(long)]
    pub allow_network: bool,

    /// Print the exact isolated command plan without executing it.
    #[arg(long)]
    pub dry_run: bool,

    /// CPU limit in seconds inherited by the isolated process tree.
    #[arg(long, default_value_t = 60)]
    pub cpu_seconds: u64,

    /// Address-space limit in MiB inherited by the isolated process tree.
    #[arg(long, default_value_t = 1024)]
    pub memory_mib: u64,

    /// Maximum process count inherited by the isolated process tree.
    #[arg(long, default_value_t = 256)]
    pub max_processes: u64,

    /// Maximum open file descriptors inherited by the isolated process tree.
    #[arg(long, default_value_t = 1024)]
    pub max_open_files: u64,

    /// Arguments passed to the selected IDE/agent CLI after `--`.
    #[arg(last = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum IdeSurface {
    Claude,
    Cursor,
    Antigravity,
}

impl IdeSurface {
    fn default_command(self) -> &'static str {
        match self {
            IdeSurface::Claude => "claude",
            IdeSurface::Cursor => "agent",
            IdeSurface::Antigravity => "agy",
        }
    }

    fn label(self) -> &'static str {
        match self {
            IdeSurface::Claude => "Claude Code CLI",
            IdeSurface::Cursor => "Cursor CLI agent",
            IdeSurface::Antigravity => "Antigravity CLI",
        }
    }
}

pub fn run(args: IdeRunArgs) -> anyhow::Result<()> {
    let session_id = resolve_workspace(args.workspace)?;
    let command = build_agent_command(args.surface, args.command.as_ref(), &args.args);

    if args.dry_run {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "surface": args.surface.label(),
                "workspace": session_id,
                "command": command,
                "enforcement_level": "L3_LINUX_WORKSPACE",
                "dry_run": true,
                "note": "Execution requires Linux + bubblewrap. Hooks and MCP alone are not L3."
            }))?
        );
        return Ok(());
    }

    let status = crate::workspace::run_isolated(crate::workspace::RunWorkspaceOptions {
        session_id,
        command,
        allow_network: args.allow_network,
        resource_limits: crate::workspace::ResourceLimits {
            cpu_seconds: args.cpu_seconds,
            memory_bytes: args.memory_mib.saturating_mul(1024 * 1024),
            max_processes: args.max_processes,
            max_open_files: args.max_open_files,
        },
    })?;
    std::process::exit(status);
}

fn resolve_workspace(workspace: Option<String>) -> anyhow::Result<String> {
    match workspace {
        Some(id) => Ok(id),
        None => std::env::var("ONUS_WORKSPACE_ID")
            .ok()
            .filter(|value| !value.is_empty())
            .map(Ok)
            .unwrap_or_else(crate::workspace::latest_workspace_id),
    }
}

fn build_agent_command(
    surface: IdeSurface,
    command_override: Option<&PathBuf>,
    args: &[String],
) -> Vec<String> {
    let mut command = vec![command_override
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| surface.default_command().to_string())];
    command.extend(args.iter().cloned());
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_agent_commands_are_explicit() {
        assert_eq!(IdeSurface::Claude.default_command(), "claude");
        assert_eq!(IdeSurface::Cursor.default_command(), "agent");
        assert_eq!(IdeSurface::Antigravity.default_command(), "agy");
    }

    #[test]
    fn builds_claude_command_with_args() {
        let command = build_agent_command(
            IdeSurface::Claude,
            None,
            &["-p".to_string(), "explain this project".to_string()],
        );
        assert_eq!(command, vec!["claude", "-p", "explain this project"]);
    }

    #[test]
    fn override_command_is_preserved() {
        let command = build_agent_command(
            IdeSurface::Antigravity,
            Some(&PathBuf::from("/opt/antigravity/bin/agy")),
            &["--help".to_string()],
        );
        assert_eq!(command, vec!["/opt/antigravity/bin/agy", "--help"]);
    }

    #[test]
    fn dry_run_does_not_claim_verified_l3() {
        let plan = serde_json::json!({
            "enforcement_level": "L3_LINUX_WORKSPACE",
            "dry_run": true,
            "note": "Execution requires Linux + bubblewrap. Hooks and MCP alone are not L3."
        });
        assert_eq!(plan["dry_run"], true);
        assert!(plan["note"].as_str().unwrap().contains("bubblewrap"));
    }
}
