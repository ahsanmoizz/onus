# Windsurf Editor / Cascade Integration Report

Milestone surface: 2 of 20, Windsurf Editor / Cascade.

Branch: `integration/windsurf-editor-cascade`.

Date: 2026-06-16.

## Claim

This surface is `PROTOCOL_ONLY` in Phase 15.

Onus does not yet include a native Windsurf/Cascade runtime adapter. The
supported narrow route is to configure Cascade MCP servers so the agent talks
to `onus mcp-proxy`, which then launches the upstream MCP server.

## Official Control Surface

Official documentation reviewed:

- <https://docs.devin.ai/desktop/cascade/mcp>
- <https://docs.devin.ai/desktop/cascade/hooks>

The MCP route is the strongest locally implementable route because the repo
already contains a runtime-tested Onus MCP gateway. Hooks may become a native
L1 route later, but no local Windsurf runtime was available to verify hook
installation or execution.

## Files Added

- `integrations/windsurf-cascade/README.md`

The file is a bounded MCP configuration template. It is not a runtime proof.

## Runtime Evidence

Local Windsurf runtime:

```text
Unavailable in this environment.
```

Onus MCP gateway help:

```text
onus mcp-proxy [OPTIONS] --server <SERVER> [-- <ARGS>...]
--experimental is required before the proxy runs.
```

Relevant existing MCP evidence:

- `reports/current-state/MCP_GATEWAY_RUNTIME.md`
- `onus/src/mcp/proxy.rs`
- `onus/src/cli/mcp_proxy.rs`
- `onus/bindings/python/tests/test_onus.py::TestMcpProxyRuntime`

Validation run for this branch:

```text
python -m pytest -q -rs onus\bindings\python\tests\test_onus.py -k mcp
4 passed, 55 deselected
```

## Security Notes

- Enforcement is L2 only for MCP traffic routed through `onus mcp-proxy`.
- Direct Cascade connections to upstream MCP servers bypass Onus.
- No native Windsurf hook execution was runtime-tested.
- No Windsurf UI, editor, or Cascade agent-loop behavior was runtime-tested.
- This template does not provide L3 process/filesystem/network containment or
  L4 authority.

## Final Classification

```text
PROTOCOL_ONLY
```

Native Windsurf/Cascade support remains `BLOCKED` until a real local Windsurf
runtime is available for hook/MCP configuration and agent-loop testing.
