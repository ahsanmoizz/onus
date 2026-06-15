//! `onus workspace` — create and manage Linux L3 isolated workspaces.

use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct WorkspaceArgs {
    #[command(subcommand)]
    pub command: WorkspaceCommand,
}

#[derive(Subcommand)]
pub enum WorkspaceCommand {
    /// Create a writable session worktree from a read-only original repository
    Create(WorkspaceCreateArgs),

    /// Inspect workspace metadata and checkpoints
    Inspect(WorkspaceInspectArgs),

    /// Export controlled artifacts from the writable worktree
    Export(WorkspaceExportArgs),

    /// Destroy a workspace and its writable worktree
    Destroy(WorkspaceDestroyArgs),
}

#[derive(Args)]
pub struct WorkspaceCreateArgs {
    /// Original repository path. This is mounted read-only during isolated runs.
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,

    /// Optional session/workspace id. Defaults to a generated id.
    #[arg(long)]
    pub session: Option<String>,

    /// Allow host network inside the isolated workspace. Default is deny-all.
    #[arg(long)]
    pub allow_network: bool,
}

#[derive(Args)]
pub struct WorkspaceInspectArgs {
    /// Workspace/session id.
    #[arg(long)]
    pub session: String,
}

#[derive(Args)]
pub struct WorkspaceExportArgs {
    /// Workspace/session id.
    #[arg(long)]
    pub session: String,

    /// Destination directory. The export creates <dest>/<session>.
    #[arg(long)]
    pub dest: PathBuf,
}

#[derive(Args)]
pub struct WorkspaceDestroyArgs {
    /// Workspace/session id.
    #[arg(long)]
    pub session: String,
}

pub fn run(args: WorkspaceArgs) -> anyhow::Result<()> {
    match args.command {
        WorkspaceCommand::Create(args) => create(args),
        WorkspaceCommand::Inspect(args) => inspect(args),
        WorkspaceCommand::Export(args) => export(args),
        WorkspaceCommand::Destroy(args) => destroy(args),
    }
}

fn create(args: WorkspaceCreateArgs) -> anyhow::Result<()> {
    let metadata = crate::workspace::create_workspace(crate::workspace::CreateWorkspaceOptions {
        repo: args.repo,
        session_id: args.session,
        allow_network: args.allow_network,
    })?;
    println!("{}", serde_json::to_string_pretty(&metadata)?);
    Ok(())
}

fn inspect(args: WorkspaceInspectArgs) -> anyhow::Result<()> {
    let metadata = crate::workspace::inspect_workspace(&args.session)?;
    println!("{}", serde_json::to_string_pretty(&metadata)?);
    Ok(())
}

fn export(args: WorkspaceExportArgs) -> anyhow::Result<()> {
    let path = crate::workspace::export_workspace(&args.session, &args.dest)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "session_id": args.session,
            "export_path": path,
            "contents": ["workspace.json", "worktree"],
            "note": "controlled export contains the writable worktree and Onus workspace metadata only"
        }))?
    );
    Ok(())
}

fn destroy(args: WorkspaceDestroyArgs) -> anyhow::Result<()> {
    crate::workspace::destroy_workspace(&args.session)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "session_id": args.session,
            "destroyed": true
        }))?
    );
    Ok(())
}
