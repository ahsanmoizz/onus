# OpenAI Codex CLI Integration Report

Milestone surface: 10 of 20, OpenAI Codex CLI.

Branch: `integration/openai-codex-cli`.

Date: 2026-06-16.

## Claim

This surface is `PROTOCOL_ONLY_WITH_LOCAL_RUNTIME_BLOCKER` in Phase 15.

Codex CLI has official MCP support, so Onus can be placed on that route. The
local Windows app binary could not be directly executed for version/runtime
proof, so no native Codex CLI claim is made.

## Official Control Surface

Official documentation reviewed:

- <https://developers.openai.com/codex/cli>
- <https://developers.openai.com/codex/cli/reference>
- <https://developers.openai.com/codex/mcp>

## Files Added

- `integrations/openai-codex-cli/README.md`

## Runtime Evidence

Local Codex runtime inventory:

```text
Codex Windows app binary present, but version probe failed with access denied.
```

Routed MCP evidence applies only if Codex CLI is configured to use Onus as its
MCP server command:

- `reports/current-state/MCP_GATEWAY_RUNTIME.md`
- `onus/src/mcp/proxy.rs`

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Security Notes

- No Codex CLI command was executed.
- No Codex CLI agent action was intercepted.
- MCP protection applies only to MCP traffic routed through `onus mcp-proxy`.
- Direct filesystem/shell/network actions outside Onus remain outside this
  report's proven boundary unless contained by a verified L3 workspace.

## Final Classification

```text
PROTOCOL_ONLY
```

Native Codex CLI runtime proof remains blocked by local executable access.
