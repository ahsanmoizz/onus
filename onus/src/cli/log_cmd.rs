//! `onus log` — view the audit trail.

use crate::audit::AuditTrail;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct LogArgs {
    /// Number of recent actions to show (default: 20)
    #[arg(short, long, default_value = "20")]
    pub limit: u32,

    /// Filter by verdict: allow, warn, block, escalate
    #[arg(short, long)]
    pub verdict: Option<String>,

    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

pub fn run(args: LogArgs) -> anyhow::Result<()> {
    let db_path = args
        .db
        .unwrap_or_else(|| crate::data_dir().join("audit.db"));

    if !db_path.exists() {
        println!("No audit data yet. Start the Onus daemon and run an agent session.");
        return Ok(());
    }

    let audit = AuditTrail::open(&db_path)?;
    let actions = audit.get_recent_actions(args.limit * 5)?;

    let filtered: Vec<_> = actions
        .iter()
        .filter(|a| {
            if let Some(ref v) = args.verdict {
                a.verdict == *v
            } else {
                true
            }
        })
        .take(args.limit as usize)
        .collect();

    if filtered.is_empty() {
        println!("No actions found.");
        return Ok(());
    }

    println!(
        "{:36}  {:8}  {:12}  {:6}  {:20}  RULE/INFO",
        "ID", "VERDICT", "TYPE", "SEQ", "TOOL"
    );
    println!("{}", "─".repeat(120));

    for action in &filtered {
        let verdict_styled = match action.verdict.as_str() {
            "allow" => format!("\x1b[32m{}\x1b[0m", action.verdict), // green
            "warn" => format!("\x1b[33m{}\x1b[0m", action.verdict),  // yellow
            "block" => format!("\x1b[31m{}\x1b[0m", action.verdict), // red
            "escalate" => format!("\x1b[35m{}\x1b[0m", action.verdict), // magenta
            _ => action.verdict.clone(),
        };

        let short_id = &action.id[..action.id.len().min(8)];
        let rule_info = action.rule_id.as_deref().unwrap_or("-");
        let tool = action.tool_name.as_deref().unwrap_or("-");

        println!(
            "{:8}..  {}  {:12}  {:>4}  {:20}  {}",
            short_id, verdict_styled, action.action_type, action.sequence as i64, tool, rule_info,
        );

        if let Some(ref correction) = action.correction {
            if action.verdict == "block" || action.verdict == "escalate" {
                println!("  └─ Correction: {}", correction);
            }
        }
    }

    Ok(())
}
