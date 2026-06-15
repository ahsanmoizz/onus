use crate::audit::AuditTrail;
use crate::task_contract::{CompletionEvidence, CompletionStatus, TaskContract};
use clap::{Args, Subcommand};
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Args)]
pub struct ContractArgs {
    #[command(subcommand)]
    pub command: ContractCommand,
}

#[derive(Subcommand)]
pub enum ContractCommand {
    /// Persist a task contract before the session starts
    Start(ContractStartArgs),
    /// Verify completion evidence against a persisted contract
    Complete(ContractCompleteArgs),
}

#[derive(Args)]
pub struct ContractStartArgs {
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Contract JSON file. If omitted, stdin is used.
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Workspace root to persist with the session
    #[arg(long)]
    pub workspace_root: Option<PathBuf>,

    /// Agent name for the session row
    #[arg(long, default_value = "contract")]
    pub agent_name: String,
}

#[derive(Args)]
pub struct ContractCompleteArgs {
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Session ID to complete
    #[arg(long)]
    pub session_id: String,

    /// Evidence JSON file. If omitted, stdin is used.
    #[arg(long)]
    pub evidence: Option<PathBuf>,
}

pub fn run(args: ContractArgs) -> anyhow::Result<()> {
    match args.command {
        ContractCommand::Start(args) => start(args),
        ContractCommand::Complete(args) => complete(args),
    }
}

fn start(args: ContractStartArgs) -> anyhow::Result<()> {
    let raw = read_input(args.file.as_ref())?;
    let contract: TaskContract = serde_json::from_str(&raw)?;
    let db_path = args
        .db
        .unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let workspace = args
        .workspace_root
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let mut audit = AuditTrail::open(&db_path)?;
    audit.start_session(
        &contract.session_id,
        &args.agent_name,
        None,
        &contract.normalized_objective,
        &workspace.display().to_string(),
    )?;
    let saved = audit.save_task_contract(&contract)?;
    println!(
        "{}",
        serde_json::json!({
            "status": "ok",
            "session_id": saved.session_id,
            "contract_hash": saved.canonical_hash,
            "environment_identity": saved.environment_identity,
            "policy_version": saved.policy_version
        })
    );
    Ok(())
}

fn complete(args: ContractCompleteArgs) -> anyhow::Result<()> {
    let raw = read_input(args.evidence.as_ref())?;
    let evidence: Vec<CompletionEvidence> = serde_json::from_str(&raw)?;
    let db_path = args
        .db
        .unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let mut audit = AuditTrail::open(&db_path)?;
    let status = audit.complete_task_contract(&args.session_id, &evidence)?;
    match status {
        CompletionStatus::CompletedVerified => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "COMPLETED_VERIFIED",
                    "missing_evidence": []
                })
            );
            Ok(())
        }
        CompletionStatus::CompletedWithExceptions { exceptions } => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "COMPLETED_WITH_EXCEPTIONS",
                    "missing_evidence": [],
                    "exceptions": exceptions
                })
            );
            Ok(())
        }
        CompletionStatus::HumanReviewRequired {
            missing_evidence,
            findings,
        } => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "HUMAN_REVIEW_REQUIRED",
                    "missing_evidence": missing_evidence,
                    "findings": findings,
                    "correction": format!(
                        "Completion rejected. Provide required evidence: {}",
                        missing_evidence.join(", ")
                    )
                })
            );
            std::process::exit(4);
        }
        CompletionStatus::FailedSafely { findings } => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "FAILED_SAFELY",
                    "missing_evidence": [],
                    "findings": findings,
                    "correction": "Completion rejected due to critical quality or security findings."
                })
            );
            std::process::exit(5);
        }
        CompletionStatus::RolledBack { findings } => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ROLLED_BACK",
                    "missing_evidence": [],
                    "findings": findings
                })
            );
            std::process::exit(6);
        }
        CompletionStatus::Terminated { findings } => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "TERMINATED",
                    "missing_evidence": [],
                    "findings": findings
                })
            );
            std::process::exit(7);
        }
    }
}

fn read_input(path: Option<&PathBuf>) -> anyhow::Result<String> {
    if let Some(path) = path {
        Ok(std::fs::read_to_string(path)?)
    } else {
        let mut raw = String::new();
        io::stdin().read_to_string(&mut raw)?;
        Ok(raw)
    }
}
