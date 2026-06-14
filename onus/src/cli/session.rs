//! `onus session` — view a specific session's details.

use crate::audit::AuditTrail;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct SessionArgs {
    /// Session ID to view
    pub session_id: String,

    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

pub fn run(args: SessionArgs) -> anyhow::Result<()> {
    let db_path = args
        .db
        .unwrap_or_else(|| crate::data_dir().join("audit.db"));

    let audit = AuditTrail::open(&db_path)?;

    match audit.get_session(&args.session_id)? {
        Some(session) => {
            println!("Session:        {}", session.id);
            println!(
                "Agent:          {} {}",
                session.agent_name,
                session.agent_version.as_deref().unwrap_or("")
            );
            println!("Task:           {}", session.task_description);
            println!("Workspace:      {}", session.workspace_root);
            println!("──────────────────────────────────────────");
            println!("Total actions:  {}", session.total_actions);
            println!("Blocked:        {}", session.blocked_actions);
            println!("Escalated:      {}", session.escalated_actions);

            println!("Status:         {}", session.status);

            if let Some(ended) = session.ended_at {
                let duration = ended - session.started_at;
                println!("Duration:       {}s", duration);
            }

            let actions = audit.get_session_actions(&args.session_id)?;
            if !actions.is_empty() {
                println!();
                println!("Replay:");
                println!(
                    "{:>4}  {:8}  {:12}  {:20}  PAYLOAD",
                    "STEP", "VERDICT", "TYPE", "TOOL"
                );
                println!("{}", "─".repeat(120));
                for action in actions {
                    let tool = action.tool_name.as_deref().unwrap_or("-");
                    let mut payload = crate::security::mask_text_for_display(&action.payload)
                        .replace('\n', "\\n");
                    if payload.len() > 140 {
                        payload.truncate(137);
                        payload.push_str("...");
                    }
                    println!(
                        "{:>4}  {:8}  {:12}  {:20}  {}",
                        action.sequence, action.verdict, action.action_type, tool, payload
                    );
                    if let Some(correction) = action.correction {
                        println!("      correction: {}", correction);
                    }
                }
            }
        }
        None => {
            println!("Session '{}' not found.", args.session_id);
        }
    }

    Ok(())
}
