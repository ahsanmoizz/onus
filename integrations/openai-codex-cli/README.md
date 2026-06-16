# Onus for OpenAI Codex CLI

Status: `PROTOCOL_ONLY_WITH_LOCAL_RUNTIME_BLOCKER`.

The local environment contains a Windows app Codex binary path, but direct
version probing failed with access denied. Therefore this directory does not
claim a runtime-tested Codex CLI adapter.

Official control surfaces reviewed:

- <https://developers.openai.com/codex/cli>
- <https://developers.openai.com/codex/cli/reference>
- <https://developers.openai.com/codex/mcp>

## MCP Gateway Route

Where Codex CLI is configured to use MCP servers, configure the server command
to launch Onus:

```toml
[mcp_servers.example-through-onus]
command = "D:\\Onus\\onus\\target\\debug\\onus.exe"
args = [
  "mcp-proxy",
  "--experimental",
  "--server",
  "C:\\path\\to\\real-mcp-server.exe",
  "--",
  "--real-server-arg"
]
```

## Claim Boundary

Safe claim:

```text
Onus can govern Codex CLI MCP tool calls only when Codex CLI is configured to
route those MCP calls through `onus mcp-proxy`.
```

Unsafe claims:

```text
Onus controls Codex CLI directly in this environment.
Onus protects Codex CLI actions outside MCP or L3 routing.
The Windows app Codex binary has been runtime-tested.
```
