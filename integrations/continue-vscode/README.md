# Onus for Continue Agent for VS Code

Status: `BLOCKED`.

VS Code is installed locally, but Continue Agent for VS Code is not detected.
Generic VS Code extension checks do not prove Continue Agent interception.

Official control surfaces reviewed:

- <https://docs.continue.dev/>
- <https://github.com/continuedev/continue>

## Future Onus Routes

Use only runtime-tested routes:

1. Continue-specific tool permission/executor configuration.
2. MCP routing through `onus mcp-proxy`.
3. L3 workspace isolation for local side effects.

## Claim Boundary

Safe claim:

```text
Continue Agent for VS Code is blocked locally pending the Continue extension
runtime and a verified control surface.
```

Unsafe claims:

```text
The generic Onus VS Code extension proves Continue Agent protection.
Onus controls Continue Agent for VS Code today.
```
