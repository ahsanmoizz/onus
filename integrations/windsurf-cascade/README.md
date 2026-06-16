# Onus for Windsurf Editor / Cascade

Status: `PROTOCOL_ONLY`.

Windsurf/Cascade is not installed in this local environment, so this directory
does not claim a runtime-tested native Windsurf adapter. It provides the
bounded MCP routing shape to place Onus between Cascade and an upstream MCP
server when Cascade is configured to use MCP.

Official control surfaces reviewed:

- <https://docs.devin.ai/desktop/cascade/mcp>
- <https://docs.devin.ai/desktop/cascade/hooks>

## MCP Gateway Route

Configure Cascade to launch the Onus MCP proxy instead of launching the
upstream MCP server directly.

Template:

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
        "ONUS_INTEGRATION": "windsurf-cascade",
        "ONUS_ENFORCEMENT_LEVEL": "L2_ROUTED_MCP_ONLY"
      }
    }
  }
}
```

Replace:

- `D:\\Onus\\onus\\target\\debug\\onus.exe` with the installed Onus binary path.
- `C:\\path\\to\\real-mcp-server.exe` with the upstream MCP server command.
- `--real-server-arg` with upstream server arguments, or remove it.

## Claim Boundary

Safe claim:

```text
Onus can enforce L2 policy for Windsurf/Cascade MCP traffic that is explicitly
routed through `onus mcp-proxy`.
```

Unsafe claims:

```text
Onus natively controls Windsurf/Cascade.
Onus protects Cascade actions that bypass the MCP proxy.
Onus provides L3/L4 containment for Windsurf/Cascade through this template.
```

Direct connections from Cascade to the real MCP server bypass Onus.
