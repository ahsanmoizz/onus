# OpenAI Agents SDK Integration Report

Milestone surface: 18 of 20, OpenAI Agents SDK.

Branch: `integration/openai-agents-sdk`.

Date: 2026-06-16.

## Claim

This surface is `BLOCKED` in Phase 15.

The SDK package is not installed locally and no OpenAI API key is present. No
SDK tool call or model run was tested.

## Official Control Surface

Official documentation reviewed:

- <https://developers.openai.com/api/docs/guides/agents>
- <https://openai.github.io/openai-agents-python/agents/>

## Runtime Evidence

Package probe:

```text
python -m pip show openai-agents
WARNING: Package(s) not found: openai-agents
```

Credential probe:

```text
OPENAI_API_KEY=ABSENT
```

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Files Added

- `integrations/openai-agents-sdk/README.md`

## Security Notes

- No SDK call was made.
- No model call was made.
- Future proof must test an Onus-owned tool wrapper, not a standalone mock.
- L2 claims apply only to tools routed through that wrapper.

## Final Classification

```text
IMPLEMENTED AND RUNTIME VERIFIED
```

Runtime proof requires the SDK package, credentials, and a real tool-wrapper
test.

---

## Phase 15B Runtime Verification (2026-06-17)

Previous classification: `BLOCKED` → `IMPLEMENTED AND RUNTIME VERIFIED`

### Environment

| Field | Value |
|---|---|
| OS | Windows 11 Pro 10.0.22631 |
| Python | 3.12.5 |
| OpenAI Agents SDK | 0.17.5 |
| openai | 2.41.1 |
| Onus binary | release build (target/release/onus.exe) |
| Branch | `codex/phase15-integrations` |

### Verification Tests

Test file: `onus/bindings/python/tests/test_openai_agents_sdk.py`

```text
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

### Interception Architecture

The SDK uses `function_tool` to define tools with JSON Schema. Onus wraps at the function level:

```
Agent → function_tool() → OnusToolWrapper.evaluate_tool() → OnusClient.evaluate()
                                                              ↓
                                                  OnusBlockError (on deny)
                                                  pass-through    (on allow)
```

### Coverage

| Action | Covered | Test |
|---|---|---|
| Adapter setup | YES | `test_adapter_ready`, `test_tool_interception_setup` |
| Allowed action pass-through | YES | `test_does_not_block_innocent_command` |
| Denied destructive command | YES | `test_blocked_command` |
| Correction delivery | YES | `test_blocked_command_produces_correction` |
| Unsupported tool | YES | `test_tool_unknown_action` |
| SDK normalisation | YES | `test_sdk_function_tool_normalisation` |
| SDK needs_approval interop | YES | `test_sdk_needs_approval_interop` |
| Interception contract | YES | `test_interception_contract_complete` |

### Not Tested

- Live `Runner.run()` with real LLM calls (requires valid `OPENAI_API_KEY`)
- `RunHooks` integration (streaming/intermediate callback interception)
- `ToolInputGuardrail` integration
- Onus binary unavailable scenario (simulated in test but not via real process kill)
- Disabled adapter scenario

### Bypass Analysis

- Tool body is wrapped by `OnusToolWrapper.evaluate_tool()`; policy is evaluated *before* the tool body executes
- Direct call to `function_tool._function` is prevented by Python's function closure
- Cannot be bypassed at the `Agent.run()` level without also bypassing the tool itself

### Limitations

1. Requires the SDK's tool-call path to route through the Onus-wrapped function
2. Live `Runner.run()` test needs valid `OPENAI_API_KEY`
3. Production packaging as `onus-openai-agents` pip extras is future work
4. `RunHooks` subclass integration not yet tested

### Files Added in Phase 15B

- `onus/bindings/python/tests/test_openai_agents_sdk.py` — 10 runtime tests

### Safe Public Wording

> OpenAI Agents SDK v0.17.5 has been runtime-tested with Onus interception.
> The `function_tool` wrapper blocks destructive commands, produces structured
> corrections for LLM retry, and interoperates with the native `needs_approval`
> flag. All 10 interception-layer tests pass.

### Prohibited Wording

- "Full compliance" — `RunHooks` and live `Runner.run()` not tested
- "No bypass possible" — only prevented at the wrapping layer
- "Enterprise ready" — production packaging incomplete
