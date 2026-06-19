//! `onus approvals` — manage the human approval workflow.
//!
//! Subcommands:
//!   list   — list pending/resolved approvals
//!   show   — inspect a single approval with binding verification
//!   approve — approve a pending escalation
//!   deny   — deny a pending escalation
//!   cancel — cancel an approval (reject it)
//!   serve  — serve the local approval UI

use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::audit::db::AuditTrail;
use crate::security;

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn open_audit(db: &Option<PathBuf>) -> anyhow::Result<AuditTrail> {
    let db_path = db
        .clone()
        .unwrap_or_else(|| crate::data_dir().join("audit.db"));
    AuditTrail::open(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open audit DB at {}: {}", db_path.display(), e))
}

// -- Args ----------------------------------------------------------------

#[derive(Args)]
pub struct ApprovalsArgs {
    #[command(subcommand)]
    pub command: ApprovalsCommand,
}

#[derive(Subcommand)]
pub enum ApprovalsCommand {
    /// List pending or resolved approvals
    List(ListArgs),
    /// Show a single approval with binding verification
    Show(ShowArgs),
    /// Approve a pending escalation
    Approve(ApproveArgs),
    /// Deny a pending escalation
    Deny(DenyArgs),
    /// Cancel an approval (alias for deny)
    Cancel(CancelArgs),
    /// Serve the local approval UI
    Serve(ServeArgs),
}

// -- List -----------------------------------------------------------------

#[derive(Args)]
pub struct ListArgs {
    /// Filter by status: pending, approved, rejected, expired, all
    #[arg(long, default_value = "pending")]
    pub status: String,
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_list(args: &ListArgs) -> anyhow::Result<()> {
    let audit = open_audit(&args.db)?;
    let all = audit.get_pending_approvals()?;
    let filtered: Vec<_> = match args.status.as_str() {
        "all" => all.into_iter().collect(),
        s => all.into_iter().filter(|a| a.status == s).collect(),
    };
    if filtered.is_empty() {
        println!("No approvals with status '{}'", args.status);
        return Ok(());
    }
    for a in &filtered {
        let expires_str = if a.expires_at > 0 {
            let remaining = a.expires_at - unix_now();
            if remaining > 0 {
                format!("{}s remaining", remaining)
            } else {
                "expired".to_string()
            }
        } else {
            "no expiry".to_string()
        };
        println!(
            "  [{:>4}] {} | {} | {} | {} | {}",
            a.id, a.action_id, a.action_type, a.status, a.rule_name, expires_str,
        );
    }
    Ok(())
}

// -- Show -----------------------------------------------------------------

#[derive(Args)]
pub struct ShowArgs {
    /// Action ID to inspect
    pub action_id: String,
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_show(args: &ShowArgs) -> anyhow::Result<()> {
    let audit = open_audit(&args.db)?;
    let all = audit.get_pending_approvals()?;
    let approval = all
        .into_iter()
        .find(|a| a.action_id == args.action_id)
        .ok_or_else(|| anyhow::anyhow!("No approval found with action_id '{}'", args.action_id))?;

    let binding_json = serde_json::to_string_pretty(&serde_json::json!({
        "payload_bound": !approval.canonical_payload_hash.is_empty(),
        "session_bound": !approval.session_id.is_empty(),
        "policy_version_present": !approval.policy_version.is_empty(),
        "has_expiry": approval.expires_at > 0,
        "action_id": approval.action_id,
        "session_id": approval.session_id,
        "canonical_payload_hash": approval.canonical_payload_hash,
        "policy_version": approval.policy_version,
        "environment_identity": approval.environment_identity,
        "expires_at": approval.expires_at,
        "approver": approval.approver,
    }))?;

    println!("Approval {}", approval.action_id);
    println!("  Status:               {}", approval.status);
    println!("  Action type:          {}", approval.action_type);
    println!("  Tool:                 {:?}", approval.tool_name);
    println!("  Rule:                 {} ({})", approval.rule_name, approval.rule_id);
    println!("  Correction:           {}", approval.correction);
    println!("  Guardian mode:        {:?}", approval.guardian_mode);
    println!("  Created at:           {:?}", approval.created_at);
    println!("  Resolved at:          {:?}", approval.resolved_at);
    println!();
    println!("Binding verification:");
    println!("{}", binding_json);
    Ok(())
}

// -- Approve --------------------------------------------------------------

#[derive(Args)]
pub struct ApproveArgs {
    /// Action ID to approve
    pub action_id: String,
    /// Approver name
    #[arg(long, default_value = "cli-user")]
    pub approver: String,
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_approve(args: &ApproveArgs) -> anyhow::Result<()> {
    let mut audit = open_audit(&args.db)?;
    let all = audit.get_pending_approvals()?;
    let approval = all
        .iter()
        .find(|a| a.action_id == args.action_id)
        .ok_or_else(|| anyhow::anyhow!("No approval found with action_id '{}'", args.action_id))?;

    if approval.status != "pending" {
        anyhow::bail!(
            "Approval '{}' is not pending (current status: {}). Cannot approve.",
            args.action_id, approval.status
        );
    }
    if approval.expires_at > 0 && approval.expires_at <= unix_now() {
        anyhow::bail!(
            "Approval '{}' has expired at {}. Cannot approve.",
            args.action_id, approval.expires_at
        );
    }
    audit.approve_action(&args.action_id, &args.approver)?;
    println!("Approved: {}", args.action_id);
    Ok(())
}

// -- Deny -----------------------------------------------------------------

#[derive(Args)]
pub struct DenyArgs {
    /// Action ID to deny
    pub action_id: String,
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_deny(args: &DenyArgs) -> anyhow::Result<()> {
    let mut audit = open_audit(&args.db)?;
    let all = audit.get_pending_approvals()?;
    let approval = all
        .iter()
        .find(|a| a.action_id == args.action_id)
        .ok_or_else(|| anyhow::anyhow!("No approval found with action_id '{}'", args.action_id))?;

    if approval.status != "pending" {
        anyhow::bail!(
            "Approval '{}' is not pending (current status: {}). Cannot deny.",
            args.action_id, approval.status
        );
    }
    audit.reject_action(&args.action_id)?;
    println!("Denied: {}", args.action_id);
    Ok(())
}

// -- Cancel ---------------------------------------------------------------

#[derive(Args)]
pub struct CancelArgs {
    /// Action ID to cancel
    pub action_id: String,
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_cancel(args: &CancelArgs) -> anyhow::Result<()> {
    let mut audit = open_audit(&args.db)?;
    let all = audit.get_pending_approvals()?;
    let approval = all
        .iter()
        .find(|a| a.action_id == args.action_id)
        .ok_or_else(|| anyhow::anyhow!("No approval found with action_id '{}'", args.action_id))?;

    if approval.status != "pending" {
        anyhow::bail!(
            "Approval '{}' is not pending (current status: {}). Cannot cancel.",
            args.action_id, approval.status
        );
    }
    audit.reject_action(&args.action_id)?;
    println!("Cancelled: {}", args.action_id);
    Ok(())
}

// -- Serve ----------------------------------------------------------------

#[derive(Args)]
pub struct ServeArgs {
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
    /// Port to serve the approval UI on
    #[arg(long, default_value_t = 9191)]
    pub port: u16,
    /// Local approval UI token. If omitted, a random token is generated.
    #[arg(long)]
    pub token: Option<String>,
}

fn run_serve(args: &ServeArgs) -> anyhow::Result<()> {
    let db_path = args.db.clone().unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let token = args.token.clone().unwrap_or_else(security::local_token);
    let state = crate::approval::ApprovalState::open(&db_path)?;
    println!("Onus approval UI: http://127.0.0.1:{}?token={}", args.port, token);
    println!("Reading audit DB: {}", db_path.display());
    crate::approval::serve(state, args.port, token)
}

// -- Dispatch -------------------------------------------------------------

pub fn run(args: ApprovalsArgs) -> anyhow::Result<()> {
    match args.command {
        ApprovalsCommand::List(a) => run_list(&a),
        ApprovalsCommand::Show(a) => run_show(&a),
        ApprovalsCommand::Approve(a) => run_approve(&a),
        ApprovalsCommand::Deny(a) => run_deny(&a),
        ApprovalsCommand::Cancel(a) => run_cancel(&a),
        ApprovalsCommand::Serve(a) => run_serve(&a),
    }
}
