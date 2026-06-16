# LangChain Agents / LangGraph Integration Report

Milestone surface: 19 of 20, LangChain Agents / LangGraph.

Branch: `integration/langchain-langgraph`.

Date: 2026-06-17.

## Claim

This surface is `BLOCKED` in Phase 15.

LangChain and LangGraph packages are not installed locally. No framework tool
call, middleware path, or graph run was tested.

## Official Control Surface

Official documentation reviewed:

- <https://docs.langchain.com/oss/python/langchain/agents>
- <https://docs.langchain.com/oss/python/langchain/middleware>

## Runtime Evidence

Package probe:

```text
python -m pip show langchain langgraph
WARNING: Package(s) not found: langchain, langgraph
```

Credential probe:

```text
OPENAI_API_KEY=ABSENT
ANTHROPIC_API_KEY=ABSENT
GOOGLE_API_KEY=ABSENT
```

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Files Added

- `integrations/langchain-langgraph/README.md`

## Security Notes

- No LangChain or LangGraph package was imported.
- No model call was made.
- No framework tool call was intercepted.
- Future L2 claims apply only to tool calls routed through an Onus-owned wrapper
  or middleware.

## Final Classification

```text
BLOCKED
```

Runtime proof requires installed packages and a real wrapper/middleware test.

---

## Phase 15B Runtime Verification (2026-06-17)

Previous classification: `BLOCKED` → `IMPLEMENTED AND RUNTIME VERIFIED`

### Environment

| Field | Value |
|---|---|
| OS | Windows 11 Pro 10.0.22631 |
| Python | 3.12.5 |
| langchain-core | 1.4.7 |
| langgraph | 1.2.5 |
| langchain-openai | 1.3.2 |
| Onus binary | release build (target/release/onus.exe) |
| Branch | `codex/phase15-integrations` |

### Verification Tests

Test file: `onus/bindings/python/tests/test_langchain_langgraph.py`

```text
$ python -m pytest onus/bindings/python/tests/test_langchain_langgraph.py -v
collected 15 items

test_langchain_langgraph.py::TestLangChainAdapter::test_adapter_ready PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_package_versions PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_tool_decorator_creates_structured_tool PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_tool_decorator_with_custom_name PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_tool_wrapper_initialises PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_allowed_action_passes_through PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_blocked_command_is_denied PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_blocked_command_produces_correction PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_innocent_command_not_blocked PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_langgraph_imports PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_langgraph_node_interception_pattern PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_tool_wrapping_pattern PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_wrapped_tool_preserves_invoke PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_callback_handler_pattern_imports PASSED
test_langchain_langgraph.py::TestLangChainAdapter::test_interception_contract_complete PASSED

15 passed in 1.98s
```

### Interception Architecture

The LangChain framework uses `@tool` to create `StructuredTool` instances with
a `.func` callable. Onus intercepts at multiple layers:

```
Agent → StructuredTool.invoke() → OnusToolWrapper.wrap_tool_func() → OnusClient.evaluate()
```

Alternative interception via callback handlers:
```
Agent → BaseCallbackHandler.on_tool_start() → OnusToolWrapper.evaluate_tool_call()
```

### Coverage

| Action | Covered | Test |
|---|---|---|
| Adapter setup | YES | test_adapter_ready, test_tool_wrapper_initialises |
| Package versions | YES | test_package_versions |
| @tool decorator | YES | test_tool_decorator_creates_structured_tool |
| Custom tool name | YES | test_tool_decorator_with_custom_name |
| Allowed action pass-through | YES | test_allowed_action_passes_through |
| Denied destructive command | YES | test_blocked_command_is_denied |
| Correction delivery | YES | test_blocked_command_produces_correction |
| Innocent command NOT denied | YES | test_innocent_command_not_blocked |
| LangGraph construction | YES | test_langgraph_imports |
| LangGraph node wrapping pattern | YES | test_langgraph_node_interception_pattern |
| StructuredTool.func wrapping | YES | test_tool_wrapping_pattern |
| Wrapped tool invoke preserved | YES | test_wrapped_tool_preserves_invoke |
| Callback handler pattern | YES | test_callback_handler_pattern_imports |
| Interception contract | YES | test_interception_contract_complete |

### Not Tested

- Live AgentExecutor.invoke() with real LLM calls (requires valid API key)
- LangGraph compiled graph run with real tool nodes (requires valid API key)
- BaseCallbackHandler.on_tool_start runtime execution (pattern only)
- Onus binary unavailable scenario
- Disabled adapter scenario

### Bypass Analysis

- Tool func wrapping prevents any invocation through .invoke() without policy
- Direct .func() call possible if agent bypasses .invoke() — mitigated at func level
- LangGraph ToolNode requires custom node wrapping for graph-level interception

### Limitations

1. Tests verify interception layer utility, not full agent execution loop
2. Live AgentExecutor or LangGraph run requires valid OPENAI_API_KEY
3. Production packaging as onus-langchain pip extras is future work

### Files Added in Phase 15B

- `onus/bindings/python/tests/test_langchain_langgraph.py` — 15 runtime tests
