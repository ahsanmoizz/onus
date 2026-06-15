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

    /// Enable the experimental MCP proxy. Without this flag, the command exits
    /// with an explicit unsupported error instead of implying production-ready
    /// MCP protection.
    #[arg(long)]
    pub experimental: bool,

    /// Existing governed session ID to bind MCP calls to. If omitted, the proxy
    /// creates a fresh session ID and contract enforcement follows missing-contract policy.
    #[arg(long)]
    pub session_id: Option<String>,

    /// Maximum time to wait for the upstream MCP server response.
    #[arg(long, default_value_t = 5000)]
    pub response_timeout_ms: u64,

    /// Maximum upstream MCP response size in bytes.
    #[arg(long, default_value_t = 1048576)]
    pub max_response_bytes: usize,
}

pub fn run(args: McpProxyArgs) -> anyhow::Result<()> {
    let env_enabled = std::env::var("ONUS_EXPERIMENTAL_MCP_PROXY")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if !args.experimental && !env_enabled {
        anyhow::bail!(
            "mcp-proxy is experimental and not enabled by default. Re-run with --experimental or set ONUS_EXPERIMENTAL_MCP_PROXY=1."
        );
    }

    crate::mcp::proxy::run_proxy(
        &args.server,
        &args.args,
        args.db_path,
        Some(args.approval_port),
        args.session_id,
        args.response_timeout_ms,
        args.max_response_bytes,
    )
}
