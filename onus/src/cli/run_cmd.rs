//! `onus run` — execute an agent command inside an Onus-controlled workspace.

use clap::Args;

#[derive(Args)]
pub struct RunArgs {
    /// Require L3 isolation. Without this flag, Onus refuses to execute.
    #[arg(long)]
    pub isolate: bool,

    /// Workspace/session id. Defaults to ONUS_WORKSPACE_ID or latest workspace.
    #[arg(long)]
    pub workspace: Option<String>,

    /// Allow host network egress for this run. Default is deny-all.
    #[arg(long)]
    pub allow_network: bool,

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

    /// Agent command to execute after `--`.
    #[arg(last = true, required = true)]
    pub command: Vec<String>,
}

pub fn run(args: RunArgs) -> anyhow::Result<()> {
    if !args.isolate {
        anyhow::bail!(
            "`onus run` is an L3 entrypoint and requires --isolate; refusing unmanaged execution"
        );
    }

    let session_id = match args.workspace {
        Some(id) => id,
        None => std::env::var("ONUS_WORKSPACE_ID")
            .ok()
            .filter(|value| !value.is_empty())
            .map(Ok)
            .unwrap_or_else(crate::workspace::latest_workspace_id)?,
    };

    let status = crate::workspace::run_isolated(crate::workspace::RunWorkspaceOptions {
        session_id,
        command: args.command,
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
