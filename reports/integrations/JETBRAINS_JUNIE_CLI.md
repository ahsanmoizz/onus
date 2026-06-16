# JetBrains Junie CLI Integration Report

Milestone surface: 15 of 20, JetBrains Junie CLI.

Branch: `integration/jetbrains-junie-cli`.

Date: 2026-06-16.

## Claim

This surface is `PROTOCOL_ONLY` in Phase 15.

Junie CLI is not installed or authenticated locally. The bounded route is MCP
traffic explicitly routed through `onus mcp-proxy`.

## Official Control Surface

Official documentation reviewed:

- <https://junie.jetbrains.com/docs/>
- <https://junie.jetbrains.com/docs/junie-cli-usage.html>
- <https://junie.jetbrains.com/docs/junie-cli-mcp-configuration.html>

## Files Added

- `integrations/jetbrains-junie-cli/README.md`

## Runtime Evidence

Local Junie CLI runtime:

```text
Unavailable in this environment.
```

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Security Notes

- No Junie CLI command was executed.
- No Junie CLI agent action was intercepted.
- MCP enforcement applies only to routed MCP traffic.
- Direct Junie execution outside Onus remains outside this report's proven
  boundary.

## Final Classification

```text
PROTOCOL_ONLY
```

Native Junie CLI proof remains blocked until installed and authenticated.
