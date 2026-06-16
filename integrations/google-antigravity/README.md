# Onus for Google Antigravity

Status: `PROTOCOL_ONLY`.

Google Antigravity is not installed in this local environment. This directory
provides only a bounded MCP gateway route for Antigravity configurations that
support MCP servers.

Official control surfaces reviewed:

- <https://antigravity.google/docs/mcp>
- <https://ai.google.dev/gemini-api/docs/antigravity-agent>

## MCP Gateway Route

Configure Antigravity to launch `onus mcp-proxy` instead of the upstream MCP
server directly:

```json
{
  "mcpServers": {
    "example-through-onus": {
      "command": "D:\\Onus\\onus\\target\\debug\\onus.exe",
      "args": [
        "mcp-proxy",
        "--experimental",
        "--server",
        "C:\\path\\to\\real-mcp-server.exe",
        "--",
        "--real-server-arg"
      ],
      "env": {
        "ONUS_INTEGRATION": "google-antigravity",
        "ONUS_ENFORCEMENT_LEVEL": "L2_ROUTED_MCP_ONLY"
      }
    }
  }
}
```

## Claim Boundary

Safe claim:

```text
Onus can enforce L2 policy for Antigravity MCP tools explicitly routed through
`onus mcp-proxy`.
```

Unsafe claims:

```text
Onus natively controls Google Antigravity.
Onus protects Antigravity actions that bypass MCP routing.
Onus has runtime-tested Antigravity agent-loop integration.
```
