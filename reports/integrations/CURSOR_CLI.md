# Cursor CLI Integration Report

Milestone surface: 7 of 20, Cursor CLI.

Branch: `integration/cursor-cli`.

Date: 2026-06-16.

## Claim

This surface is `BLOCKED` in Phase 15.

Cursor CLI was not available in the local runtime inventory. The existing VS
Code extension is not Cursor CLI evidence and is not used as proof for this
surface.

## Official Control Surface

Official documentation reviewed:

- <https://cursor.com/docs>

## Files Added

- `integrations/cursor-cli/README.md`

## Runtime Evidence

Local Cursor CLI runtime:

```text
Unavailable in this environment.
```

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Security Notes

- No Cursor CLI command was executed.
- No Cursor agent action was intercepted.
- Future protection must use a native hook/tool surface, MCP routing, or a
  proven L3 workspace wrapper.
- Cursor IDE, Cursor CLI, and Cursor background agents are separate surfaces and
  must not share unearned evidence.

## Final Classification

```text
BLOCKED
```

Runtime proof requires an installed Cursor CLI and a documented control surface.
