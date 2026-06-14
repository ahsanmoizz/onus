//! `onus mcp-proxy` — run as an MCP proxy.
//!
//! Spawns a real MCP server as a subprocess and bridges stdio between the
//! agent (MCP client) and the server. On `tools/call`, the payload is
//! evaluated through Onus Core before forwarding.
//!
//! ## Usage
//!
//! ```bash
//! onus mcp-proxy --server /path/to/mcp-server -- args to server
//! ```
//!
//! Agent connects to the proxy instead of the real server:
//! ```json
//! { "command": "onus", "args": ["mcp-proxy", "--server", "/path/to/server", "--", "arg1"] }
//! ```

use clap::Args;

#[derive(Args)]
pub struct McpProxyArgs {
    /// Path to the MCP server binary
    #[arg(long)]
    pub server: String,

    /// Arguments to pass to the MCP server (after --)
    #[arg(last = true)]
    pub args: Vec<String>,

    /// Path to the Onus audit database (default: ~/.local/share/onus/audit.db or
    /// equivalent on this platform)
    #[arg(long)]
    pub db_path: Option<std::path::PathBuf>,

    /// Port for the approval web UI (default: not started). Set to 9191 to enable.
    #[arg(long, default_value_t = 9191)]
    pub approval_port: u16,
}

pub fn run(args: McpProxyArgs) -> anyhow::Result<()> {
    crate::mcp::proxy::run_proxy(
        &args.server,
        &args.args,
        args.db_path,
        Some(args.approval_port),
    )
}
