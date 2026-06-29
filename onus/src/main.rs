//! Onus — AI Agent Firewall
//!
//! Entry point for the `onus` CLI binary.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    onus_core::config_env::load_default_env_file()?;

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let cli = onus_core::cli::Cli::parse();

    match cli.command {
        onus_core::cli::Commands::Approvals(args) => onus_core::cli::approvals::run(args)?,
        onus_core::cli::Commands::Authority(args) => onus_core::cli::authority::run(args)?,
        onus_core::cli::Commands::ClaudeHook(args) => onus_core::cli::claude_hook::run(args)?,
        onus_core::cli::Commands::Evaluate(args) => onus_core::cli::evaluate::run(args)?,
        onus_core::cli::Commands::Daemon(args) => onus_core::cli::daemon_cmd::run(args)?,
        onus_core::cli::Commands::Contract(args) => onus_core::cli::contract::run(args)?,
        onus_core::cli::Commands::Dashboard(args) => onus_core::cli::dashboard::run(args)?,
        onus_core::cli::Commands::Intake(args) => onus_core::cli::intake::run(args)?,
        onus_core::cli::Commands::Status => onus_core::cli::status::run()?,
        onus_core::cli::Commands::Log(args) => onus_core::cli::log_cmd::run(args)?,
        onus_core::cli::Commands::Run(args) => onus_core::cli::run_cmd::run(args)?,
        onus_core::cli::Commands::Session(args) => onus_core::cli::session::run(args)?,
        onus_core::cli::Commands::Rules(args) => onus_core::cli::rules::run(args)?,
        onus_core::cli::Commands::Upgrade => onus_core::cli::upgrade::run()?,
        onus_core::cli::Commands::Doctor(args) => onus_core::cli::doctor::run(args)?,
        onus_core::cli::Commands::Setup(args) => onus_core::cli::setup::run(args)?,
        onus_core::cli::Commands::Uninstall(args) => onus_core::cli::uninstall::run(args)?,
        onus_core::cli::Commands::McpProxy(args) => onus_core::cli::mcp_proxy::run(args)?,
        onus_core::cli::Commands::Shell(args) => onus_core::cli::shell::run(args)?,
        onus_core::cli::Commands::CursorHook(args) => onus_core::cli::cursor_hook::run(args)?,
        onus_core::cli::Commands::Verify(args) => onus_core::cli::verify::run(args)?,
        onus_core::cli::Commands::Checkpoint(args) => onus_core::cli::recovery::run_checkpoint(args)?,
        onus_core::cli::Commands::Rollback(args) => onus_core::cli::recovery::run_rollback(args)?,
        onus_core::cli::Commands::Compensation(args) => onus_core::cli::recovery::run_compensation(args)?,
        onus_core::cli::Commands::Workspace(args) => onus_core::cli::workspace::run(args)?,
        onus_core::cli::Commands::Memory(args) => onus_core::cli::memory::run(args)?,
        onus_core::cli::Commands::Handoff(args) => onus_core::cli::handoff::run(args)?,
        onus_core::cli::Commands::Lease(args) => onus_core::cli::lease_cli::run(args)?,

        // ── Convenience aliases ────────────────────────────────────────────
        onus_core::cli::Commands::Start(args) => onus_core::cli::start::run(args)?,
        onus_core::cli::Commands::Stop(args) => onus_core::cli::stop::run(args)?,
        onus_core::cli::Commands::Restart(args) => onus_core::cli::restart::run(args)?,
        onus_core::cli::Commands::Console(args) => onus_core::cli::console::run(args)?,
    }

    Ok(())
}
