# Continue Agent for JetBrains Integration Report

Milestone surface: 14 of 20, Continue Agent for JetBrains.

Branch: `integration/continue-agent-jetbrains`.

Date: 2026-06-16.

## Claim

This surface is `BLOCKED` in Phase 15.

No JetBrains IDE runtime or Continue Agent for JetBrains runtime is available
locally.

## Official Control Surface

Official documentation reviewed:

- <https://docs.continue.dev/>
- <https://github.com/continuedev/continue>

## Files Added

- `integrations/continue-jetbrains/README.md`

## Runtime Evidence

Local JetBrains/Continue runtime:

```text
Unavailable in this environment.
```

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Security Notes

- No JetBrains IDE agent loop was run.
- No Continue JetBrains tool call was intercepted.
- Continue CLI or VS Code evidence does not prove JetBrains behavior.

## Final Classification

```text
BLOCKED
```

Runtime proof requires JetBrains and Continue Agent for JetBrains to be
installed and configured.
