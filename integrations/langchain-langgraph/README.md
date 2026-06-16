# Onus for LangChain Agents / LangGraph

Status: `BLOCKED`.

LangChain and LangGraph are not installed in this local environment. This
directory does not claim runtime-tested framework integration.

Official control surfaces reviewed:

- <https://docs.langchain.com/oss/python/langchain/agents>
- <https://docs.langchain.com/oss/python/langchain/middleware>

## Future Onus Route

A real adapter should wrap LangChain/LangGraph tool execution or middleware:

1. Normalize every tool call into canonical Onus action JSON.
2. Evaluate deterministic policy before invoking the tool.
3. Require exact approval binding for risky tools.
4. Return structured correction to the agent when denied.
5. Fail closed for critical evaluator or middleware failures.

## Claim Boundary

Safe claim:

```text
LangChain/LangGraph integration is blocked locally pending installed packages
and a runtime-tested Onus tool-wrapper or middleware adapter.
```

Unsafe claims:

```text
Onus controls LangChain or LangGraph agents today.
Manual Guardian usage proves framework integration.
Fixture-only tests prove live LangChain/LangGraph support.
```
