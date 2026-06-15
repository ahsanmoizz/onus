//! `onus authority` — narrow L4 disposable authority proof.

use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct AuthorityArgs {
    #[command(subcommand)]
    pub command: AuthorityCommand,
}

#[derive(Subcommand)]
pub enum AuthorityCommand {
    /// Create a disposable SQLite authority owned by Onus
    InitDisposableDb(InitDisposableDbArgs),
    /// Issue a short-lived exact-payload capability after human approval
    Authorize(AuthorizeArgs),
    /// Execute the broker-owned privileged action
    Execute(ExecuteArgs),
    /// Revoke an unused short-lived capability
    Revoke(RevokeArgs),
    /// Execute supported compensation for a receipt
    Compensate(CompensateArgs),
    /// Inspect authority metadata without exposing raw credentials
    Inspect(InspectArgs),
    /// Print authority receipts
    Receipts(ReceiptsArgs),
}

#[derive(Args)]
pub struct InitDisposableDbArgs {
    #[arg(long)]
    pub authority: String,
    #[arg(long)]
    pub db: PathBuf,
    #[arg(long)]
    pub environment: String,
}

#[derive(Args)]
pub struct AuthorizeArgs {
    #[arg(long)]
    pub authority: String,
    #[arg(long)]
    pub session: String,
    #[arg(long)]
    pub payload: PathBuf,
    #[arg(long)]
    pub approver: String,
    #[arg(long, default_value_t = 300)]
    pub ttl_seconds: i64,
    /// Explicit proof of human approval for this narrow L4 capability.
    #[arg(long)]
    pub human_approved: bool,
}

#[derive(Args)]
pub struct ExecuteArgs {
    #[arg(long)]
    pub authority: String,
    #[arg(long)]
    pub capability: String,
    #[arg(long)]
    pub payload: PathBuf,
}

#[derive(Args)]
pub struct RevokeArgs {
    #[arg(long)]
    pub authority: String,
    #[arg(long)]
    pub capability: String,
}

#[derive(Args)]
pub struct CompensateArgs {
    #[arg(long)]
    pub authority: String,
    #[arg(long)]
    pub receipt: String,
}

#[derive(Args)]
pub struct InspectArgs {
    #[arg(long)]
    pub authority: String,
}

#[derive(Args)]
pub struct ReceiptsArgs {
    #[arg(long)]
    pub authority: String,
}

pub fn run(args: AuthorityArgs) -> anyhow::Result<()> {
    match args.command {
        AuthorityCommand::InitDisposableDb(args) => init_disposable_db(args),
        AuthorityCommand::Authorize(args) => authorize(args),
        AuthorityCommand::Execute(args) => execute(args),
        AuthorityCommand::Revoke(args) => revoke(args),
        AuthorityCommand::Compensate(args) => compensate(args),
        AuthorityCommand::Inspect(args) => inspect(args),
        AuthorityCommand::Receipts(args) => receipts(args),
    }
}

fn init_disposable_db(args: InitDisposableDbArgs) -> anyhow::Result<()> {
    let metadata =
        crate::authority::init_disposable_db(crate::authority::InitDisposableDbOptions {
            authority_id: args.authority,
            db_path: args.db,
            environment_identity: args.environment,
        })?;
    println!("{}", serde_json::to_string_pretty(&metadata)?);
    Ok(())
}

fn authorize(args: AuthorizeArgs) -> anyhow::Result<()> {
    let (record, token) = crate::authority::authorize(crate::authority::AuthorizeOptions {
        authority_id: args.authority,
        session_id: args.session,
        payload_path: args.payload,
        approver: args.approver,
        ttl_seconds: args.ttl_seconds,
        human_approved: args.human_approved,
    })?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "capability": token,
            "capability_id": record.capability_id,
            "action_id": record.action_id,
            "canonical_payload_hash": record.canonical_payload_hash,
            "environment_identity": record.environment_identity,
            "expires_at_unix": record.expires_at_unix,
            "approver": record.approver,
            "policy_version": record.policy_version,
            "note": "capability is short-lived, scoped, and not a long-lived credential"
        }))?
    );
    Ok(())
}

fn execute(args: ExecuteArgs) -> anyhow::Result<()> {
    let receipt = crate::authority::execute(crate::authority::ExecuteOptions {
        authority_id: args.authority,
        capability_token: args.capability,
        payload_path: args.payload,
    })?;
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}

fn revoke(args: RevokeArgs) -> anyhow::Result<()> {
    let record = crate::authority::revoke(crate::authority::RevokeOptions {
        authority_id: args.authority,
        capability_token: args.capability,
    })?;
    println!("{}", serde_json::to_string_pretty(&record)?);
    Ok(())
}

fn compensate(args: CompensateArgs) -> anyhow::Result<()> {
    let receipt = crate::authority::compensate(crate::authority::CompensateOptions {
        authority_id: args.authority,
        receipt_id: args.receipt,
    })?;
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}

fn inspect(args: InspectArgs) -> anyhow::Result<()> {
    let metadata = crate::authority::inspect(&args.authority)?;
    println!("{}", serde_json::to_string_pretty(&metadata)?);
    Ok(())
}

fn receipts(args: ReceiptsArgs) -> anyhow::Result<()> {
    let receipts = crate::authority::load_receipts(&args.authority)?;
    println!("{}", serde_json::to_string_pretty(&receipts)?);
    Ok(())
}
