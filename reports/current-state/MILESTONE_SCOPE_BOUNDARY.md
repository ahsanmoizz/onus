# Onus Milestone Scope Boundary

Generated: 2026-06-15

This file is not a locked product specification. It records the current working-tree boundary so the implementation is not presented as a single clean, isolated milestone.

## Certification Status

The current working tree is not commit-ready as one milestone.

It contains work from multiple roadmap phases:

- specification lock infrastructure;
- repository truth/audit cleanup;
- security corrections;
- task-contract lifecycle;
- prompt intake guardian;
- semantic reviewer provider architecture;
- approval binding and local approval/dashboard hardening;
- MCP proxy runtime behavior;
- DEMO_ONLY reality demo updates;
- site/UI disclaimer updates.

## Safe Claim for Current Runtime

Onus can currently claim L2-style enforcement only for actions routed through implemented Onus SDK/CLI/proxy paths.

Supported claim:

> Onus can evaluate and block or escalate governed actions that are routed through its Python SDK, CLI evaluator, and experimental MCP proxy path, with audit persistence and hash-chain verification for tested local scenarios.

## Unsupported Claims

Do not claim:

- production-ready universal agent firewall;
- L3/L4 containment;
- non-bypassability outside routed Onus paths;
- real cloud LLM integration proven end to end;
- production MCP compatibility across arbitrary clients and servers;
- full provider architecture production certification;
- warning-free clippy baseline.

## Proven Runtime Evidence In This Worktree

- deterministic policy block dominates approval escalation in CLI evaluation;
- hardcoded secret file write is blocked and redacted before audit persistence;
- audit persistence failure blocks instead of silently allowing;
- prompt intake Scenario A produces a safe contract;
- local semantic adapter fixture is invoked through the SDK/CLI path;
- malformed semantic output is rejected and falls back deterministically;
- critical semantic provider failure fails closed;
- experimental MCP proxy creates pending approval, forwards exact approved payload, and requires new approval when payload changes;
- DEMO_ONLY reality demo runs and receipt-chain verification passes.

## Remaining Blockers

- Canonical filename mismatch remains unresolved:
  - `AGENTS.md` references `docs/Onus_Whitepaper.md` and `docs/ONUS_CURRENT_STATE.md`.
  - The repository contains `docs/Onus_Whitepaper.txt` and `docs/Onus_current_state.md`.
  - This must be resolved only through the approved specification process or repository normalization authorized by the owner.
- Real remote semantic provider integration is skipped unless `ONUS_SEMANTIC_REAL_PROVIDER=1`, `ONUS_SEMANTIC_ENDPOINT`, `ONUS_SEMANTIC_MODEL`, and credentials or a localhost endpoint are configured.
- Clippy still reports the existing warning baseline.

## Commit Strategy Recommendation

Do not commit the entire worktree as one milestone.

Split into separate commits or PRs in this order:

1. spec lock and repository audit artifacts;
2. security corrections;
3. task-contract lifecycle;
4. prompt intake guardian;
5. semantic reviewer provider architecture;
6. experimental MCP proxy proof and claim bounding;
7. dashboard/site disclaimers.

Each commit should include only its own source changes, tests, and runtime evidence.
