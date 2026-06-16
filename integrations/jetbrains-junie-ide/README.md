# Onus for JetBrains Junie IDE Agent

Status: `BLOCKED`.

JetBrains IDE and Junie IDE Agent runtimes are not available in this local
environment. This directory does not claim runtime-tested IDE-agent support.

Official control surfaces reviewed:

- <https://www.jetbrains.com/help/ai-assistant/junie-agent.html>
- <https://www.jetbrains.com/help/ai-assistant/mcp.html>

## Future Onus Routes

Use only runtime-tested routes:

1. JetBrains/Junie native integration surface, if available.
2. MCP routing through `onus mcp-proxy` where JetBrains MCP configuration is
   used.
3. L3 workspace isolation for local side effects.

## Claim Boundary

Safe claim:

```text
JetBrains Junie IDE Agent integration is blocked locally pending JetBrains IDE
runtime and verified configuration.
```

Unsafe claims:

```text
Junie CLI evidence proves Junie IDE Agent protection.
Onus controls JetBrains Junie IDE Agent today.
```
