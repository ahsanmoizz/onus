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
