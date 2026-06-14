//! `onus daemon` — start, stop, and manage the Onus Core background process.

use clap::{Args, Subcommand};

#[derive(Args)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub command: DaemonCommand,
}

#[derive(Subcommand)]
pub enum DaemonCommand {
    /// Start the Onus daemon (background by default)
    Start {
        /// Run in foreground (for debugging)
        #[arg(long)]
        foreground: bool,
    },
    /// Stop the running Onus daemon
    Stop,
    /// Check if the daemon is running
    Status,
}

pub fn run(args: DaemonArgs) -> anyhow::Result<()> {
    match args.command {
        DaemonCommand::Start { foreground } => {
            crate::daemon::start_daemon(foreground)?;
        }
        DaemonCommand::Stop => {
            crate::daemon::stop_daemon()?;
        }
        DaemonCommand::Status => {
            if crate::daemon::is_running() {
                let pid = crate::daemon::get_pid().unwrap_or(0);
                println!("Onus daemon is running (PID: {})", pid);
            } else {
                println!("Onus daemon is not running");
            }
        }
    }
    Ok(())
}
