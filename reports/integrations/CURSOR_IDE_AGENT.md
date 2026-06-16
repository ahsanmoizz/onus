# Cursor Agent in Cursor IDE Integration Report

Milestone surface: 8 of 20, Cursor Agent in Cursor IDE.

Branch: `integration/cursor-agent-ide`.

Date: 2026-06-16.

## Claim

This surface is `BLOCKED` in Phase 15.

Cursor IDE is not installed locally. Onus cannot claim Cursor Agent
interception from the VS Code extension or from Cursor CLI evidence.

## Official Control Surface

Official documentation reviewed:

- <https://cursor.com/docs>

## Files Added

- `integrations/cursor-ide-agent/README.md`

## Runtime Evidence

Local Cursor IDE runtime:

```text
Unavailable in this environment.
```

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Security Notes

- No Cursor IDE agent loop was run.
- No Cursor IDE tool call was intercepted.
- Generic VS Code extension checks are not Cursor IDE proof.
- Future evidence must come from Cursor itself or from a routed MCP/L3 boundary.

## Final Classification

```text
BLOCKED
```

Runtime proof requires an installed Cursor IDE and a documented integration
surface.
