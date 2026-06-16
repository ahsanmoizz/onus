# Onus for OpenAI Agents SDK

Status: `BLOCKED`.

The OpenAI Agents SDK package is not installed locally and no model API key is
present. This directory does not claim runtime-tested SDK support.

Official control surfaces reviewed:

- <https://developers.openai.com/api/docs/guides/agents>
- <https://openai.github.io/openai-agents-python/agents/>

## Future Onus Route

A real adapter should wrap SDK tool execution:

1. Normalize each tool call into canonical Onus action JSON.
2. Evaluate deterministic policy before calling the tool.
3. Enforce exact approval binding for risky tools.
4. Return structured correction to the agent when denied.
5. Fail closed on evaluator or adapter failure for critical decisions.

## Claim Boundary

Safe claim:

```text
OpenAI Agents SDK integration is blocked locally pending installed SDK package,
credentials, and a tested Onus-owned tool wrapper.
```

Unsafe claims:

```text
Onus controls OpenAI Agents SDK tools today.
Fixture-only wrappers prove live OpenAI Agents SDK support.
```
