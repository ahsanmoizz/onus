# Onus for Cursor CLI

Status: `BLOCKED`.

Cursor CLI is not installed in this local environment, so Onus cannot claim a
runtime-tested Cursor CLI adapter.

Official documentation reviewed:

- <https://cursor.com/docs>

## Future Onus Routes

Possible routes, pending a real Cursor CLI runtime:

1. Native Cursor CLI hook or tool-executor surface, if officially supported.
2. MCP routing through `onus mcp-proxy`, if Cursor CLI exposes MCP server
   configuration for the relevant actions.
3. `onus run --isolate -- <cursor command>` inside a verified Linux L3
   workspace when filesystem/process/network containment is required.

## Claim Boundary

Safe claim:

```text
Cursor CLI integration is blocked locally pending an installed Cursor CLI
runtime and official control-surface verification.
```

Unsafe claims:

```text
Onus protects Cursor CLI today.
VS Code extension evidence proves Cursor CLI protection.
Onus controls Cursor CLI background or cloud actions.
```
