//! MCP proxy — intercepts MCP tool calls, evaluates them through Onus Core,
//! and either forwards them to the real MCP server or blocks with a correction.
//!
//! ## Architecture
//!
//! ```text
//! Agent (MCP client)  ←→  Onus MCP Proxy  ←→  Real MCP Server
//!                                 │
//!                                 ▼
//!                          Onus Core eval
//!                         (allow / block / escalate)
//! ```
//!
//! The proxy runs as a stdio bridge: the agent connects to the proxy via stdio,
//! the proxy forwards JSON-RPC messages to the real server via a subprocess or socket.
//!
//! On `tools/call`, before forwarding, the proxy evaluates the tool + arguments
//! through Onus Core. If blocked, the proxy returns an MCP error with the correction
//! message — the agent re-plans naturally.

pub mod proxy;
