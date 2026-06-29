//! `onus start` — convenience shortcut for starting the daemon.
//!
//! Delegates to `onus daemon start` logic.

use clap::Args;

#[derive(Args)]
pub struct StartArgs {
    /// Run in foreground (for debugging)
    #[arg(long)]
    pub foreground: bool,
}

pub fn run(args: StartArgs) -> anyhow::Result<()> {
    crate::daemon::start_daemon(args.foreground)
}
