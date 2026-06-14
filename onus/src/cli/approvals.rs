//! `onus approvals` — serve the local approval gate UI.

use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ApprovalsArgs {
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Port to serve the approval UI on
    #[arg(long, default_value_t = 9191)]
    pub port: u16,

    /// Local approval UI token. If omitted, a random token is generated.
    #[arg(long)]
    pub token: Option<String>,
}

pub fn run(args: ApprovalsArgs) -> anyhow::Result<()> {
    let db_path = args
        .db
        .unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let token = args.token.unwrap_or_else(crate::security::local_token);
    let state = crate::approval::ApprovalState::open(&db_path)?;
    println!(
        "Onus approval UI: http://127.0.0.1:{}?token={}",
        args.port, token
    );
    println!("Reading audit DB: {}", db_path.display());
    crate::approval::serve(state, args.port, token)
}
