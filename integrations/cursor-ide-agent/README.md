# Onus for Cursor Agent in Cursor IDE

Status: `BLOCKED`.

Cursor IDE is not installed in this local environment. The generic VS Code
extension is not proof of Cursor Agent interception.

Official documentation reviewed:

- <https://cursor.com/docs>

## Future Onus Routes

Use only routes that can be runtime-tested in Cursor itself:

1. A native Cursor extension/agent hook, if officially supported.
2. MCP routing through `onus mcp-proxy`, if Cursor Agent launches MCP servers
   for the relevant tools.
3. L3 workspace containment for local side effects on Linux.

## Claim Boundary

Safe claim:

```text
Cursor IDE Agent integration is blocked locally pending a real Cursor IDE
runtime and a documented control surface.
```

Unsafe claims:

```text
The VS Code extension proves Cursor Agent protection.
Onus controls Cursor IDE Agent today.
Onus protects Cursor IDE cloud/background actions through this adapter.
```
