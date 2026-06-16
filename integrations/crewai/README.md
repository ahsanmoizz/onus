# Onus for CrewAI

Status: `BLOCKED`.

CrewAI is not installed in this local environment. This directory does not claim
runtime-tested CrewAI integration.

Official control surfaces reviewed:

- <https://docs.crewai.com/en/introduction>
- <https://docs.crewai.com/>

## Future Onus Route

A real adapter should wrap CrewAI tool or flow execution:

1. Normalize every tool/flow side effect into canonical Onus action JSON.
2. Evaluate deterministic policy before invoking the tool.
3. Require exact approval binding for risky actions.
4. Return structured correction when denied.
5. Fail closed for critical evaluator or adapter failures.

## Claim Boundary

Safe claim:

```text
CrewAI integration is blocked locally pending installed CrewAI package,
credentials where needed, and a runtime-tested Onus tool/flow wrapper.
```

Unsafe claims:

```text
Onus controls CrewAI agents today.
Manual Guardian use proves CrewAI framework integration.
Fixture-only tests prove live CrewAI support.
```
