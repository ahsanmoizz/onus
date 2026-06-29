//! `onus stop` — convenience shortcut for stopping the daemon.
//!
//! Delegates to `onus daemon stop` logic.

use clap::Args;

#[derive(Args)]
pub struct StopArgs;

pub fn run(_args: StopArgs) -> anyhow::Result<()> {
    crate::daemon::stop_daemon()
}
