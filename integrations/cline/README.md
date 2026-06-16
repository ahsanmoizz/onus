# Onus for Cline

Status: `PROTOCOL_ONLY`.

Cline is not installed in this local environment, so this directory does not
claim a runtime-tested Cline adapter. It provides the bounded MCP routing shape
for Cline configurations that launch MCP servers.

Official control surface reviewed:

- <https://docs.cline.bot/mcp/mcp-overview>

## MCP Gateway Route

Configure Cline's MCP server entry to launch Onus as the server command. Onus
then launches the real upstream MCP server after evaluating `tools/call`
requests.

Template for a Cline MCP server config:

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
        "ONUS_INTEGRATION": "cline",
        "ONUS_ENFORCEMENT_LEVEL": "L2_ROUTED_MCP_ONLY"
      }
    }
  }
}
```

## Claim Boundary

Safe claim:

```text
Onus can enforce L2 policy for Cline MCP tools that are explicitly routed
through `onus mcp-proxy`.
```

Unsafe claims:

```text
Onus natively controls Cline.
Onus protects Cline tools or edits that do not route through the MCP proxy.
Onus provides L3/L4 containment for Cline through this template.
```

Direct Cline access to an upstream MCP server bypasses Onus.
