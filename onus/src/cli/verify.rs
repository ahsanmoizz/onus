//! `onus verify` — Verify the hash chain integrity of the audit trail.
//! Walks every action, recomputes each hash, and reports any tampered records.

use crate::audit::AuditTrail;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct VerifyArgs {
    /// Path to the audit database (default: data_dir/audit.db)
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Optional session ID to verify only that session
    #[arg(long)]
    pub session_id: Option<String>,
}

pub fn run(args: VerifyArgs) -> anyhow::Result<()> {
    let db_path = args.db.unwrap_or_else(|| crate::data_dir().join("audit.db"));

    if !db_path.exists() {
        anyhow::bail!("Audit database not found at {}", db_path.display());
    }

    let audit = AuditTrail::open(&db_path)?;

    let bad = if let Some(session_id) = &args.session_id {
        audit.verify_chain(session_id)?
    } else {
        audit.verify_all_actions()?
    };

    if bad.is_empty() {
        println!("Hash chain integrity verified: ALL PASS");
        Ok(())
    } else {
        println!("Hash chain integrity BROKEN ({} broken links):", bad.len());
        for (action_id, reason) in &bad {
            println!("  Action {}: {}", action_id, reason);
        }
        anyhow::bail!("Hash chain verification failed for {} action(s)", bad.len());
    }
}
