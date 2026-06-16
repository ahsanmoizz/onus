# Onus for Continue CLI

Status: `PROTOCOL_ONLY`.

Continue CLI is not installed in this local environment. This directory provides
only a bounded route for future Continue CLI configurations that can send tool
calls through Onus-owned wrappers or MCP.

Official control surfaces reviewed:

- <https://docs.continue.dev/>
- <https://github.com/continuedev/continue>

## Intended Onus Routes

Preferred routes, pending local runtime proof:

1. Continue tool permission or tool-executor configuration that calls Onus
   before side effects.
2. MCP server routing through `onus mcp-proxy`.
3. L3 workspace isolation for local filesystem/process/network containment.

## Claim Boundary

Safe claim:

```text
Continue CLI integration is protocol-only locally; native runtime proof is
blocked until Continue CLI is installed and configured.
```

Unsafe claims:

```text
Onus controls Continue CLI today.
Onus protects Continue CLI direct tools outside an Onus route.
```
