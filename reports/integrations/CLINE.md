# Cline Integration Report

Milestone surface: 3 of 20, Cline.

Branch: `integration/cline`.

Date: 2026-06-16.

## Claim

This surface is `PROTOCOL_ONLY` in Phase 15.

Onus does not yet include a native Cline runtime adapter. The supported narrow
route is to configure Cline MCP servers so Cline talks to `onus mcp-proxy`
instead of directly launching the upstream MCP server.

## Official Control Surface

Official documentation reviewed:

- <https://docs.cline.bot/mcp/mcp-overview>

Cline's MCP support is the strongest currently implementable control surface in
this repo because Onus already contains a runtime-tested MCP gateway.

## Files Added

- `integrations/cline/README.md`

The file is a bounded MCP configuration template. It is not a runtime proof of
Cline itself.

## Runtime Evidence

Local Cline runtime:

```text
Unavailable in this environment.
```

Routed Onus MCP gateway validation for this branch:

```text
python -m pytest -q -rs onus\bindings\python\tests\test_onus.py -k mcp
4 passed, 55 deselected
```

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Security Notes

- Enforcement is L2 only for MCP traffic routed through `onus mcp-proxy`.
- Direct Cline connections to upstream MCP servers bypass Onus.
- Native Cline tool execution, direct file edits, and editor behavior were not
  runtime-tested.
- This template does not provide L3 containment or L4 authority.

## Final Classification

```text
PROTOCOL_ONLY
```

Native Cline support remains `BLOCKED` until Cline is installed and configured
for real MCP gateway testing.
