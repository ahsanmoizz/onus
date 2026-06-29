//! `onus restart` — convenience shortcut for restarting the daemon.
//!
//! Delegates to `onus daemon stop; onus daemon start`.

use clap::Args;

#[derive(Args)]
pub struct RestartArgs;

pub fn run(_args: RestartArgs) -> anyhow::Result<()> {
    if crate::daemon::is_running() {
        eprintln!("Stopping daemon...");
        crate::daemon::stop_daemon()?;
    }
    eprintln!("Starting daemon...");
    crate::daemon::start_daemon(false)
}
