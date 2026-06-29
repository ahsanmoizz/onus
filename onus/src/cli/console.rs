//! `onus console` — convenience shortcut for launching the dashboard/console UI.
//!
//! Delegates to `onus dashboard` logic with sensible defaults
//! (port 3001, local-only binding, auto-generated token).

use clap::Args;

#[derive(Args)]
pub struct ConsoleArgs {
    /// Port to bind the console HTTP server (default: 3001)
    #[arg(long, default_value = "3001")]
    pub port: u16,

    /// Path to audit database (auto-detected if omitted)
    #[arg(long)]
    pub db: Option<String>,

    /// Console authentication token (auto-generated if omitted)
    #[arg(long)]
    pub token: Option<String>,
}

pub fn run(args: ConsoleArgs) -> anyhow::Result<()> {
    let dashboard_args = crate::cli::dashboard::DashboardArgs {
        port: args.port,
        db: args.db.map(std::path::PathBuf::from),
        token: args.token,
    };
    crate::cli::dashboard::run(dashboard_args)
}
