# Continue CLI Integration Report

Milestone surface: 12 of 20, Continue CLI.

Branch: `integration/continue-cli`.

Date: 2026-06-16.

## Claim

This surface is `PROTOCOL_ONLY` in Phase 15.

Continue CLI is unavailable locally. No native Continue CLI runtime behavior is
claimed.

## Official Control Surface

Official documentation reviewed:

- <https://docs.continue.dev/>
- <https://github.com/continuedev/continue>

## Files Added

- `integrations/continue-cli/README.md`

## Runtime Evidence

Local Continue CLI runtime:

```text
Unavailable in this environment.
```

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Security Notes

- No Continue CLI command was executed.
- No Continue tool call was intercepted.
- Future protection must use an Onus-owned tool executor, MCP gateway, or L3
  workspace boundary.

## Final Classification

```text
PROTOCOL_ONLY
```

Native Continue CLI proof remains blocked until the runtime is installed.
