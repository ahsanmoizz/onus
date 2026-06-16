# Google Antigravity Integration Report

Milestone surface: 6 of 20, Google Antigravity.

Branch: `integration/google-antigravity`.

Date: 2026-06-16.

## Claim

This surface is `PROTOCOL_ONLY` in Phase 15.

No local Antigravity runtime is available. Onus can only provide a bounded MCP
gateway route for Antigravity configurations that explicitly launch
`onus mcp-proxy`.

## Official Control Surface

Official documentation reviewed:

- <https://antigravity.google/docs/mcp>
- <https://ai.google.dev/gemini-api/docs/antigravity-agent>

## Files Added

- `integrations/google-antigravity/README.md`

## Runtime Evidence

Local Antigravity runtime:

```text
Unavailable in this environment.
```

Existing routed MCP gateway evidence applies only to the Onus proxy boundary:

- `reports/current-state/MCP_GATEWAY_RUNTIME.md`
- `onus/src/mcp/proxy.rs`
- `onus/bindings/python/tests/test_onus.py::TestMcpProxyRuntime`

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Security Notes

- Enforcement is L2 only for Antigravity MCP traffic routed through Onus.
- Direct Antigravity access to an upstream MCP server bypasses Onus.
- No Antigravity UI, agent loop, or model action was runtime-tested.
- This template does not provide L3 containment or L4 authority.

## Final Classification

```text
PROTOCOL_ONLY
```

Native Antigravity runtime proof remains blocked until Antigravity is installed
and configured locally.
