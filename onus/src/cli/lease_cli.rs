//! `onus lease` — Session lease management.
//!
//! Subcommands:
//!   acquire  — acquire an exclusive session lease
//!   release  — release a session lease
//!   heartbeat — extend a lease TTL
//!   status   — show lease status for a session
//!   takeover — force-takeover a lease (requires approval ID)

use crate::lease::{LeaseError, LeaseManager};
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct LeaseArgs {
    #[command(subcommand)]
    pub command: LeaseCommand,
}

#[derive(Subcommand)]
pub enum LeaseCommand {
    /// Acquire an exclusive session lease
    Acquire(AcquireArgs),
    /// Release a session lease
    Release(ReleaseArgs),
    /// Extend a lease TTL (heartbeat)
    Heartbeat(HeartbeatArgs),
    /// Show lease status for a session
    Status(LeaseStatusArgs),
    /// Force-takeover a lease (requires human approval ID)
    Takeover(TakeoverArgs),
}

#[derive(Args)]
pub struct AcquireArgs {
    /// Session ID to lease
    #[arg(long)]
    pub session: String,

    /// Surface name (claude_code_cli, codex_cli, cursor_ide, antigravity)
    #[arg(long, default_value = "claude_code_cli")]
    pub surface: String,

    /// Identity of the holder (e.g. agent name, hostname)
    #[arg(long, default_value = "default")]
    pub identity: String,

    /// TTL in seconds (default: 3600 = 1 hour)
    #[arg(long, default_value_t = 3600)]
    pub ttl: i64,

    /// Path to lease database (default: <data_dir>/leases.db)
    #[arg(long)]
    pub db: Option<PathBuf>,
}

#[derive(Args)]
pub struct ReleaseArgs {
    /// Lease ID to release
    #[arg(long)]
    pub lease_id: String,

    /// Path to lease database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

#[derive(Args)]
pub struct HeartbeatArgs {
    /// Lease ID to extend
    #[arg(long)]
    pub lease_id: String,

    /// Extend TTL by this many seconds (default: 3600)
    #[arg(long, default_value_t = 3600)]
    pub extend: i64,

    /// Path to lease database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

#[derive(Args)]
pub struct LeaseStatusArgs {
    /// Session ID
    #[arg(long)]
    pub session: String,

    /// Path to lease database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

#[derive(Args)]
pub struct TakeoverArgs {
    /// Lease ID to take over
    #[arg(long)]
    pub lease_id: String,

    /// Previous human approval ID authorising the takeover
    #[arg(long)]
    pub approval_id: String,

    /// Path to lease database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

// ── DB path resolution ───────────────────────────────────────────────────────

fn resolve_db(custom: Option<PathBuf>) -> PathBuf {
    custom.unwrap_or_else(|| {
        let data = crate::data_dir();
        data.join("leases.db")
    })
}

// ── Command implementations ──────────────────────────────────────────────────

pub fn run_acquire(args: AcquireArgs) -> anyhow::Result<()> {
    let db_path = resolve_db(args.db);
    let mut lm = LeaseManager::open(&db_path)?;

    match lm.acquire(&args.session, &args.surface, &args.identity, args.ttl) {
        Ok(lease) => {
            println!("Lease acquired:");
            println!("  Lease ID:    {}", lease.lease_id);
            println!("  Session:     {}", lease.session_id);
            println!("  Holder:      {} ({})", lease.holder_identity, lease.holder_surface);
            println!("  Expires at:  {}", lease.expires_at);
            println!("  DB:          {}", db_path.display());
            Ok(())
        }
        Err(LeaseError::AlreadyHeld { lease_id, holder, surface, expires_at }) => {
            println!("Lease ALREADY HELD:");
            println!("  Lease ID:    {}", lease_id);
            println!("  Holder:      {} ({})", holder, surface);
            println!("  Expires at:  {}", expires_at);
            println!();
            println!("To force takeover (requires human approval):");
            println!("  onus lease takeover --lease-id {} --approval-id <id>", lease_id);
            Err(anyhow::anyhow!("Session {} is already leased by {} ({})", args.session, holder, surface))
        }
        Err(e) => Err(anyhow::anyhow!("Failed to acquire lease: {}", e)),
    }
}

pub fn run_release(args: ReleaseArgs) -> anyhow::Result<()> {
    let db_path = resolve_db(args.db);
    let mut lm = LeaseManager::open(&db_path)?;

    lm.release(&args.lease_id)?;
    println!("Lease {} released.", args.lease_id);
    Ok(())
}

pub fn run_heartbeat(args: HeartbeatArgs) -> anyhow::Result<()> {
    let db_path = resolve_db(args.db);
    let mut lm = LeaseManager::open(&db_path)?;

    lm.heartbeat(&args.lease_id, args.extend)?;
    println!("Lease {} heartbeat OK (extended by {}s).", args.lease_id, args.extend);
    Ok(())
}

pub fn run_status(args: LeaseStatusArgs) -> anyhow::Result<()> {
    let db_path = resolve_db(args.db);
    let lm = LeaseManager::open(&db_path)?;

    let active = lm.find_active(&args.session)?;
    if let Some(lease) = active {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let remaining = lease.expires_at - now;
        println!("Session {}:", args.session);
        println!("  Status:     ACTIVE");
        println!("  Lease ID:   {}", lease.lease_id);
        println!("  Holder:     {} ({})", lease.holder_identity, lease.holder_surface);
        println!("  Remaining:  {}s", remaining.max(0));
    } else {
        let all = lm.list_for_session(&args.session)?;
        if all.is_empty() {
            println!("Session {}: NO LEASES (never acquired)", args.session);
        } else {
            println!("Session {}: INACTIVE (last status: {})", args.session, all[0].status);
        }
    }
    Ok(())
}

pub fn run_takeover(args: TakeoverArgs) -> anyhow::Result<()> {
    let db_path = resolve_db(args.db);
    let mut lm = LeaseManager::open(&db_path)?;

    lm.force_takeover(&args.lease_id, &args.approval_id)?;
    println!("Lease {} taken over (approval: {}).", args.lease_id, args.approval_id);
    println!("The previous holder's lease is now 'taken_over'.");
    println!("A new agent should now acquire a fresh lease:");
    println!("  onus lease acquire --session <session-id> --surface <surface>");
    Ok(())
}

// ── Main dispatch ────────────────────────────────────────────────────────────

pub fn run(args: LeaseArgs) -> anyhow::Result<()> {
    match args.command {
        LeaseCommand::Acquire(a) => run_acquire(a),
        LeaseCommand::Release(a) => run_release(a),
        LeaseCommand::Heartbeat(a) => run_heartbeat(a),
        LeaseCommand::Status(a) => run_status(a),
        LeaseCommand::Takeover(a) => run_takeover(a),
    }
}

// ── Help text ────────────────────────────────────────────────────────────────

pub fn help_text() -> String {
    r#"onus lease acquire --session <id>      — acquire an exclusive session lease
onus lease release --lease-id <id>      — release a session lease
onus lease heartbeat --lease-id <id>    — extend a lease TTL
onus lease status --session <id>        — show lease status for a session
onus lease takeover --lease-id <id> --approval-id <id>  — force-takeover a lease"#
        .to_string()
}
