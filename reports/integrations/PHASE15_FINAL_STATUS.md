# Phase 15 Final Status

Date: 2026-06-17.

Branch: `codex/phase15-integrations`.

Base checkpoint tag: `phase15-start-7ea6979`.

## Summary

All 20 requested integration surfaces were processed in the required order.

This phase did not modify locked documents or application runtime code. It
created integration templates and evidence reports, and it refused to claim
runtime support where the local runtime, package, credentials, or service access
was unavailable.

## Phase 15B Update

Phase 15B performed runtime verification on priority surfaces where the required
software was available. One surface was upgraded from `BLOCKED` to
`IMPLEMENTED AND RUNTIME VERIFIED` with 10 passing runtime tests.

## Counts

| Category | Count |
| --- | ---: |
| Surfaces processed | 20 |
| Verified with limitations | 2 |
| IMPLEMENTED AND RUNTIME VERIFIED (Phase 15B) | 1 |
| Protocol-only | 7 |
| Blocked with evidence | 10 |
| Remaining surfaces | 0 |

## Processed Surfaces

| Order | Surface | Final classification |
| ---: | --- | --- |
| 1 | Claude Code CLI | VERIFIED WITH LIMITATIONS |
| 2 | Windsurf Editor / Cascade | PROTOCOL_ONLY |
| 3 | Cline | PROTOCOL_ONLY |
| 4 | Visual Studio Code Agents | VERIFIED WITH LIMITATIONS |
| 5 | GitHub Copilot SDK | BLOCKED |
| 6 | Google Antigravity | PROTOCOL_ONLY |
| 7 | Cursor CLI | BLOCKED |
| 8 | Cursor Agent in Cursor IDE | BLOCKED |
| 9 | Cursor Background Agents | BLOCKED |
| 10 | OpenAI Codex CLI | PROTOCOL_ONLY |
| 11 | Gemini CLI | PROTOCOL_ONLY |
| 12 | Continue CLI | PROTOCOL_ONLY |
| 13 | Continue Agent for VS Code | BLOCKED |
| 14 | Continue Agent for JetBrains | BLOCKED |
| 15 | JetBrains Junie CLI | PROTOCOL_ONLY |
| 16 | JetBrains Junie IDE Agent | BLOCKED |
| 17 | Aider | BLOCKED |
| 18 | OpenAI Agents SDK | BLOCKED (Phase 15) / IMPLEMENTED AND RUNTIME VERIFIED (Phase 15B) |
| 19 | LangChain Agents / LangGraph | BLOCKED |
| 20 | CrewAI | BLOCKED |

## Runtime Evidence

Final validation on the merged phase branch:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

```text
git diff --name-only -- AGENTS.md docs
<no output>
```

```text
cargo test claude_hook -- --nocapture
3 passed; 0 failed
```

```text
python -m pytest -q -rs onus\bindings\python\tests\test_onus.py -k "claude_hook or mcp"
8 passed, 51 deselected
```

### Phase 15B Runtime Verification

```text
$ pip install openai-agents==0.17.5 openai==2.41.1
$ python -m pytest onus/bindings/python/tests/test_openai_agents_sdk.py -v
collected 10 items

test_openai_agents_sdk.py::TestOpenAIAgentsSDKAdapter::test_adapter_ready PASSED
test_openai_agents_sdk.py::TestOpenAIAgentsSDKAdapter::test_tool_interception_setup PASSED
test_openai_agents_sdk.py::TestOpenAIAgentsSDKAdapter::test_tool_unknown_action PASSED
test_openai_agents_sdk.py::TestOpenAIAgentsSDKAdapter::test_blocked_command PASSED
test_openai_agents_sdk.py::TestOpenAIAgentsSDKAdapter::test_blocked_command_produces_correction PASSED
test_openai_agents_sdk.py::TestOpenAIAgentsSDKAdapter::test_does_not_block_innocent_command PASSED
test_openai_agents_sdk.py::TestOpenAIAgentsSDKAdapter::test_sdk_function_tool_normalisation PASSED
test_openai_agents_sdk.py::TestOpenAIAgentsSDKAdapter::test_sdk_function_tool_correct_name PASSED
test_openai_agents_sdk.py::TestOpenAIAgentsSDKAdapter::test_sdk_needs_approval_interop PASSED
test_openai_agents_sdk.py::TestOpenAIAgentsSDKAdapter::test_interception_contract_complete PASSED

10 passed in 4.42s
```

```text
python -m pytest onus/bindings/python/tests/ -q
67 passed, 2 skipped
```

```text
cargo test
75 passed; 0 failed
```

```text
python tools/spec_lock/verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Branches And Commits

Phase branch:

- `codex/phase15-integrations`

Integration branches:

- `integration/claude-code-cli`
- `integration/windsurf-editor-cascade`
- `integration/cline`
- `integration/visual-studio-code-agents`
- `integration/github-copilot-sdk`
- `integration/google-antigravity`
- `integration/cursor-cli`
- `integration/cursor-agent-ide`
- `integration/cursor-background-agents`
- `integration/openai-codex-cli`
- `integration/gemini-cli`
- `integration/continue-cli`
- `integration/continue-agent-vscode`
- `integration/continue-agent-jetbrains`
- `integration/jetbrains-junie-cli`
- `integration/jetbrains-junie-ide-agent`
- `integration/aider`
- `integration/openai-agents-sdk`
- `integration/langchain-langgraph`
- `integration/crewai`

## Security Boundary

- L1 integrations remain `BEST-EFFORT`.
- L2 applies only to actions routed through Onus-owned SDK, gateway, or wrapper
  code.
- MCP-based integrations are protocol-only unless the named product runtime was
  installed and tested against the gateway.
- Blocked surfaces must not be marketed as supported integrations.
- No L3/L4 claim was added in this phase.

## Remaining Work

Runtime support requires installing and authenticating the blocked products or
framework packages, then implementing real adapters with end-to-end tests. The
next highest-value runtime proof is one installed MCP-capable product routed
through `onus mcp-proxy`, followed by one framework SDK with an Onus-owned
tool-wrapper test.
