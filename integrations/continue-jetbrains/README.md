# Onus for Continue Agent for JetBrains

Status: `BLOCKED`.

JetBrains IDEs and Continue Agent for JetBrains are not available in this local
environment. This directory does not claim runtime-tested JetBrains support.

Official control surfaces reviewed:

- <https://docs.continue.dev/>
- <https://github.com/continuedev/continue>

## Future Onus Routes

Use only runtime-tested routes:

1. Continue-specific JetBrains tool permission or executor configuration.
2. MCP routing through `onus mcp-proxy`, if supported in the relevant setup.
3. L3 workspace isolation for local side effects.

## Claim Boundary

Safe claim:

```text
Continue Agent for JetBrains is blocked locally pending JetBrains and Continue
runtime access.
```

Unsafe claims:

```text
Onus controls Continue Agent for JetBrains today.
Continue CLI or VS Code evidence proves JetBrains support.
```
