# MCP Gateway Runtime Evidence

This report documents the local MCP gateway boundary after the runtime gateway
milestone.

## Verified Routed Path

Onus MCP protection applies when an MCP client connects to `onus mcp-proxy`
instead of connecting directly to an upstream MCP server.

Verified runtime path:

```text
local MCP client harness
  -> onus mcp-proxy
  -> local fake MCP server subprocess
```

The runtime harness verifies:

- initialization and capability negotiation;
- tool discovery;
- normalized `tools/call` payloads;
- deterministic policy evaluation;
- allow;
- deny;
- human approval;
- correction text;
- exact canonical payload binding;
- timeout handling;
- malformed JSON handling;
- upstream response-size limits;
- server identity metadata;
- secret redaction in audit persistence;
- receipts on forwarded and blocked tool calls;
- denied synthetic side effects do not execute.

## Direct-Server Bypass

Onus cannot protect MCP traffic that bypasses the gateway.

If an MCP client is configured to launch or connect to the upstream MCP server
directly, Onus does not see initialization, tool discovery, tool calls, tool
arguments, responses, approvals, or side effects.

Safe claim:

```text
Onus provides L2 enforcement for MCP actions routed through the Onus MCP proxy.
```

Unsafe claim:

```text
Onus universally protects all MCP clients and servers.
```

## Current Enforcement Level

MCP gateway enforcement is L2 for traffic routed through `onus mcp-proxy`.
It is not L3/L4 containment and does not control direct server access,
credentials, process isolation, filesystem containment, or network egress.
