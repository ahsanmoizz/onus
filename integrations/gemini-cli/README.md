# Onus for Gemini CLI

Status: `PROTOCOL_ONLY`.

Gemini CLI is not installed in this local environment. This directory provides
only the bounded MCP gateway route for Gemini CLI configurations that support
MCP servers.

Official control surfaces reviewed:

- <https://developers.google.com/gemini-code-assist/docs/gemini-cli>
- <https://github.com/google-gemini/gemini-cli>

## MCP Gateway Route

Configure Gemini CLI MCP server entries to launch Onus:

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
        "ONUS_INTEGRATION": "gemini-cli",
        "ONUS_ENFORCEMENT_LEVEL": "L2_ROUTED_MCP_ONLY"
      }
    }
  }
}
```

## Claim Boundary

Safe claim:

```text
Onus can govern Gemini CLI MCP tool calls only when those calls are explicitly
routed through `onus mcp-proxy`.
```

Unsafe claims:

```text
Onus controls Gemini CLI natively.
Onus protects Gemini CLI direct shell/file tools outside MCP or L3 routing.
Onus has runtime-tested Gemini CLI integration in this environment.
```
