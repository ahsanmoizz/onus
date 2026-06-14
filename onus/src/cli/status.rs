//! `onus status` — show daemon and audit summary.

use crate::audit::AuditTrail;
use std::path::PathBuf;

pub fn run() -> anyhow::Result<()> {
    // Check daemon status.
    let daemon_running = crate::daemon::is_running();
    let daemon_pid = crate::daemon::get_pid();

    println!("Onus v{}", env!("CARGO_PKG_VERSION"));
    println!("─────────────────────────────");

    if daemon_running {
        println!("Daemon:    RUNNING (PID: {})", daemon_pid.unwrap_or(0));
    } else {
        println!("Daemon:    STOPPED");
    }

    // Load audit trail summary.
    let db_path: PathBuf = crate::data_dir().join("audit.db");

    if db_path.exists() {
        let audit = AuditTrail::open(&db_path)?;
        match audit.get_status() {
            Ok(status) => {
                println!("Actions:   {} evaluated", status.total_actions);
                println!("Blocked:   {}", status.blocked_actions);
                println!("Escalated: {}", status.escalated_actions);
            }
            Err(e) => {
                println!("Audit:     error reading ({})", e);
            }
        }
    } else {
        println!("Audit:     no data yet");
    }

    println!("─────────────────────────────");
    println!("Config:    {}", crate::config_dir().display());
    println!("Data:      {}", crate::data_dir().display());

    Ok(())
}
