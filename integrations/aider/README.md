# Onus for Aider

Status: `BLOCKED`.

Aider is not installed in this local environment, and no model credentials are
available for an authenticated Aider run. This directory does not claim
runtime-tested Aider protection.

Official control surfaces reviewed:

- <https://aider.chat/>
- <https://aider.chat/docs/>
- <https://aider.chat/docs/usage.html>

## Future Onus Route

Aider edits repository files directly, so L1 shell hooks are not sufficient for
credible containment. Preferred future route:

```text
onus workspace create
onus run --isolate -- aider <args>
onus workspace inspect
onus workspace export
```

This requires a verified Linux L3 workspace boundary for the actual Aider run.

## Claim Boundary

Safe claim:

```text
Aider integration is blocked locally pending installed Aider, model credentials,
and a verified L3 or Onus-owned execution route.
```

Unsafe claims:

```text
Onus controls Aider today.
Shell wrapper evidence proves Aider file-write containment.
```
