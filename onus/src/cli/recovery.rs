//! `onus checkpoint` and `onus rollback` — complete recovery CLI.
//!
//! Commands:
//!   checkpoint create
//!   checkpoint list
//!   checkpoint inspect
//!   checkpoint restore
//!   rollback action
//!   rollback group
//!   rollback session
//!   compensation inspect
//!   compensation execute

use crate::audit::db::AuditTrail;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct CheckpointArgs {
    #[command(subcommand)]
    pub command: CheckpointCommand,
}

#[derive(Subcommand)]
pub enum CheckpointCommand {
    /// Create a new checkpoint of the workspace
    Create(CheckpointCreateArgs),
    /// List all checkpoints
    List,
    /// Inspect a specific checkpoint
    Inspect(CheckpointInspectArgs),
    /// Restore workspace to a checkpoint
    Restore(CheckpointRestoreArgs),
}

#[derive(Args)]
pub struct CheckpointCreateArgs {
    /// Session ID
    #[arg(long)]
    pub session: String,
    /// Workspace root path
    #[arg(long, default_value = ".")]
    pub workspace: PathBuf,
    /// Description of the checkpoint
    #[arg(long, default_value = "manual checkpoint")]
    pub description: String,
}

#[derive(Args)]
pub struct CheckpointInspectArgs {
    /// Checkpoint ID
    #[arg(long)]
    pub id: String,
}

#[derive(Args)]
pub struct CheckpointRestoreArgs {
    /// Checkpoint ID
    #[arg(long)]
    pub id: String,
    /// Workspace root path to restore to
    #[arg(long, default_value = ".")]
    pub workspace: PathBuf,
}

#[derive(Args)]
pub struct RollbackArgs {
    #[command(subcommand)]
    pub command: RollbackCommand,
}

#[derive(Subcommand)]
pub enum RollbackCommand {
    /// Roll back a single action
    Action(RollbackActionArgs),
    /// Roll back a group of actions in reverse order
    Group(RollbackGroupArgs),
    /// Roll back an entire session
    Session(RollbackSessionArgs),
}

#[derive(Args)]
pub struct RollbackActionArgs {
    /// Action ID to roll back
    #[arg(long)]
    pub action: String,
    /// Workspace root path
    #[arg(long, default_value = ".")]
    pub workspace: PathBuf,
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

#[derive(Args)]
pub struct RollbackGroupArgs {
    /// Comma-separated action IDs
    #[arg(long)]
    pub actions: String,
    /// Workspace root path
    #[arg(long, default_value = ".")]
    pub workspace: PathBuf,
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

#[derive(Args)]
pub struct RollbackSessionArgs {
    /// Session ID
    #[arg(long)]
    pub session: String,
    /// Workspace root path
    #[arg(long, default_value = ".")]
    pub workspace: PathBuf,
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

#[derive(Args)]
pub struct CompensationArgs {
    #[command(subcommand)]
    pub command: CompensationCommand,
}

#[derive(Subcommand)]
pub enum CompensationCommand {
    /// Inspect available compensation for an action
    Inspect(CompensationInspectArgs),
    /// Execute compensation for an action
    Execute(CompensationExecuteArgs),
}

#[derive(Args)]
pub struct CompensationInspectArgs {
    /// Action ID
    #[arg(long)]
    pub action: String,
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

#[derive(Args)]
pub struct CompensationExecuteArgs {
    /// Action ID to compensate
    #[arg(long)]
    pub action: String,
    /// Workspace root path
    #[arg(long, default_value = ".")]
    pub workspace: PathBuf,
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

// --- Command implementations ---

pub fn run_checkpoint(args: CheckpointArgs) -> anyhow::Result<()> {
    match args.command {
        CheckpointCommand::Create(args) => checkpoint_create(args),
        CheckpointCommand::List => checkpoint_list(),
        CheckpointCommand::Inspect(args) => checkpoint_inspect(args),
        CheckpointCommand::Restore(args) => checkpoint_restore(args),
    }
}

pub fn run_rollback(args: RollbackArgs) -> anyhow::Result<()> {
    match args.command {
        RollbackCommand::Action(args) => rollback_action(args),
        RollbackCommand::Group(args) => rollback_group(args),
        RollbackCommand::Session(args) => rollback_session(args),
    }
}

pub fn run_compensation(args: CompensationArgs) -> anyhow::Result<()> {
    match args.command {
        CompensationCommand::Inspect(args) => compensation_inspect(args),
        CompensationCommand::Execute(args) => compensation_execute(args),
    }
}

fn checkpoint_create(args: CheckpointCreateArgs) -> anyhow::Result<()> {
    let manifest = crate::rollback::create_checkpoint(&args.session, &args.workspace, &args.description)?;
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "status": "created",
        "checkpoint_id": manifest.checkpoint_id,
        "session_id": manifest.session_id,
        "file_count": manifest.file_count,
        "manifest_hash": manifest.manifest_hash,
        "created_at_unix": manifest.created_at_unix,
    }))?);
    Ok(())
}

fn checkpoint_list() -> anyhow::Result<()> {
    let checkpoints = crate::rollback::list_checkpoints()?;
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "checkpoints": checkpoints,
        "count": checkpoints.len(),
    }))?);
    Ok(())
}

fn checkpoint_inspect(args: CheckpointInspectArgs) -> anyhow::Result<()> {
    let manifest = crate::rollback::inspect_checkpoint(&args.id)?;
    println!("{}", serde_json::to_string_pretty(&manifest)?);
    Ok(())
}

fn checkpoint_restore(args: CheckpointRestoreArgs) -> anyhow::Result<()> {
    let result = crate::rollback::restore_checkpoint(&args.id, &args.workspace)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn rollback_action(args: RollbackActionArgs) -> anyhow::Result<()> {
    let db_path = args.db.unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let audit = AuditTrail::open(&db_path)?;

    // Find the action by action_id
    let actions = audit.get_recent_actions(10000)?;
    let action = actions.into_iter().find(|a| a.id == args.action)
        .ok_or_else(|| anyhow::anyhow!("Action '{}' not found in audit database", args.action))?;

    let result = crate::rollback::rollback_action(&action, &args.workspace)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn rollback_group(args: RollbackGroupArgs) -> anyhow::Result<()> {
    let db_path = args.db.unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let audit = AuditTrail::open(&db_path)?;

    let action_ids: Vec<&str> = args.actions.split(',').map(|s| s.trim()).collect();
    let all_actions = audit.get_recent_actions(10000)?;
    let actions: Vec<_> = all_actions.into_iter()
        .filter(|a| action_ids.contains(&a.id.as_str()))
        .collect();

    if actions.is_empty() {
        anyhow::bail!("No matching actions found");
    }

    // Sort by sequence to ensure reverse order execution
    let mut sorted = actions;
    sorted.sort_by_key(|a| a.sequence);

    let result = crate::rollback::rollback_group(&sorted, &args.workspace)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn rollback_session(args: RollbackSessionArgs) -> anyhow::Result<()> {
    let db_path = args.db.clone().unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let audit = AuditTrail::open(&db_path)?;

    let actions = audit.get_session_actions(&args.session)?;
    if actions.is_empty() {
        anyhow::bail!("No actions found for session '{}'", args.session);
    }

    let result = crate::rollback::rollback_session(&args.session, &actions, &args.workspace, &audit)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn compensation_inspect(args: CompensationInspectArgs) -> anyhow::Result<()> {
    let db_path = args.db.unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let audit = AuditTrail::open(&db_path)?;

    let actions = audit.get_recent_actions(10000)?;
    let action = actions.into_iter().find(|a| a.id == args.action)
        .ok_or_else(|| anyhow::anyhow!("Action '{}' not found", args.action))?;

    let comp = crate::rollback::inspect_compensation(&action)?;
    println!("{}", serde_json::to_string_pretty(&comp)?);
    Ok(())
}

fn compensation_execute(args: CompensationExecuteArgs) -> anyhow::Result<()> {
    let db_path = args.db.unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let audit = AuditTrail::open(&db_path)?;

    let actions = audit.get_recent_actions(10000)?;
    let action = actions.into_iter().find(|a| a.id == args.action)
        .ok_or_else(|| anyhow::anyhow!("Action '{}' not found", args.action))?;

    let comp = crate::rollback::execute_compensation(&action, &args.workspace)?;
    println!("{}", serde_json::to_string_pretty(&comp)?);
    Ok(())
}
