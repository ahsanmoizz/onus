# Onus for JetBrains Junie CLI

Status: `PROTOCOL_ONLY`.

JetBrains Junie CLI is not installed or authenticated in this local environment.
This directory provides only a bounded MCP gateway route for Junie CLI
configurations that support MCP.

Official control surfaces reviewed:

- <https://junie.jetbrains.com/docs/>
- <https://junie.jetbrains.com/docs/junie-cli-usage.html>
- <https://junie.jetbrains.com/docs/junie-cli-mcp-configuration.html>

## MCP Gateway Route

Configure Junie CLI MCP server entries to launch Onus:

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
        "ONUS_INTEGRATION": "jetbrains-junie-cli",
        "ONUS_ENFORCEMENT_LEVEL": "L2_ROUTED_MCP_ONLY"
      }
    }
  }
}
```

## Claim Boundary

Safe claim:

```text
Onus can govern Junie CLI MCP tool calls only when those calls are explicitly
routed through `onus mcp-proxy`.
```

Unsafe claims:

```text
Onus natively controls Junie CLI.
Onus has runtime-tested Junie CLI integration in this environment.
```
