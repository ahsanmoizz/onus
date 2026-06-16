# JetBrains Junie IDE Agent Integration Report

Milestone surface: 16 of 20, JetBrains Junie IDE Agent.

Branch: `integration/jetbrains-junie-ide-agent`.

Date: 2026-06-16.

## Claim

This surface is `BLOCKED` in Phase 15.

No JetBrains IDE or Junie IDE Agent runtime is available locally. Junie CLI
evidence is not IDE-agent proof.

## Official Control Surface

Official documentation reviewed:

- <https://www.jetbrains.com/help/ai-assistant/junie-agent.html>
- <https://www.jetbrains.com/help/ai-assistant/mcp.html>

## Files Added

- `integrations/jetbrains-junie-ide/README.md`

## Runtime Evidence

Local JetBrains/Junie IDE runtime:

```text
Unavailable in this environment.
```

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Security Notes

- No Junie IDE action was run.
- No JetBrains plugin/agent hook was tested.
- Junie CLI evidence does not prove IDE-agent behavior.
- Future proof must come from JetBrains IDE runtime or routed MCP/L3 boundary.

## Final Classification

```text
BLOCKED
```

Runtime proof requires JetBrains IDE and Junie Agent configuration.
