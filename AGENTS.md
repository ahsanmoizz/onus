# ONUS Repository Operating Contract

This file governs the entire repository.

## Canonical product documents

Read these documents before planning or modifying code:

1. `docs/Onus_Whitepaper.md`
2. `docs/ONUS_PRODUCT_VISION.md`
3. `docs/ONUS_TARGET_ARCHITECTURE.md`
4. `docs/ONUS_SECURITY_REQUIREMENTS.md`
5. `docs/ONUS_ACCEPTANCE_TESTS.md`
6. `docs/ONUS_IMPLEMENTATION_ROADMAP.md`
7. `docs/ONUS_CURRENT_STATE.md`

## Document authority

The following documents define the locked target product and must not be modified, shortened, rewritten, weakened, renamed, or deleted:

* `docs/Onus_Whitepaper.md`
* `docs/ONUS_PRODUCT_VISION.md`
* `docs/ONUS_TARGET_ARCHITECTURE.md`
* `docs/ONUS_SECURITY_REQUIREMENTS.md`
* `docs/ONUS_ACCEPTANCE_TESTS.md`
* `docs/ONUS_IMPLEMENTATION_ROADMAP.md`

`docs/ONUS_CURRENT_STATE.md` is descriptive rather than authoritative. It may be updated only after runtime evidence proves that the implementation state changed.

A request to change code does not authorize changes to the locked documents.

Locked documents may be changed only when the user explicitly writes:

`SPEC CHANGE APPROVED`

A specification change must also include an Architecture Decision Record explaining:

* requested change;
* reason;
* security effect;
* compatibility effect;
* acceptance-test changes;
* migration implications.

## Product hierarchy

When implementation and documentation conflict:

1. security requirements take priority;
2. acceptance requirements define completion;
3. target architecture defines intended boundaries;
4. product vision and whitepaper define product behavior;
5. implementation roadmap defines sequence;
6. current-state documentation describes what exists today;
7. existing code does not override the target specification merely because it already exists.

Do not silently reduce the target to match current code.

## Mandatory development behavior

Before changing code:

1. Read the relevant locked documents.
2. Inspect the existing implementation.
3. Compare the requested work against the current state.
4. Identify ambiguity, unsafe assumptions, duplicate functionality, and conflicts.
5. Correct an unsafe implementation request before acting.
6. State the exact milestone being implemented.
7. State the files expected to change.
8. Create or confirm a Git checkpoint.
9. Implement only the current milestone.
10. Run its acceptance tests.
11. Report runtime evidence.
12. Stop after the milestone.

Do not implement later roadmap phases unless explicitly requested.

## No fake completion

Never replace missing functionality with:

* mocks in production paths;
* placeholders;
* dummy data;
* hardcoded success;
* simulated runtime evidence;
* no-op functions;
* static dashboard records presented as live;
* fake approval results;
* fake rollback results;
* fake LLM responses presented as real;
* exception handling that silently returns allow.

Legitimate isolated test mocks may remain when clearly identified as test-only.

Demo and simulation paths must be labeled:

`DEMO_ONLY` or `SIMULATED`

## Security invariants

* Deterministic denial cannot be overridden by an LLM.
* Critical evaluator failure must not silently fail open.
* Secrets must not appear in logs, receipts, prompts, memory, dashboard responses, or exception traces.
* Approval must bind to the exact canonical action payload.
* Modified payloads require new approval.
* Production actions require verified environment identity.
* Completion requires evidence.
* Test deletion or weakening must not be accepted silently.
* Rollback must not be claimed without a tested restore or compensation path.
* Hash chaining alone must not be described as immutability.
* L1 cooperative hooks must be labeled BEST-EFFORT.
* L2 claims apply only to actions routed through Onus.
* L3 claims require real process, filesystem, network, and credential containment.
* L4 claims require Onus-controlled authority or credentials.

## Required validation

Run all relevant tests after modifications, including where applicable:

* `cargo test`
* `cargo build`
* `cargo clippy`
* `python -m pytest -q`
* reality demo;
* receipt-chain verification;
* affected integration tests;
* affected acceptance scenarios.

Do not describe a demo as a substitute for a missing test suite.

## Required completion report

Every implementation response must state:

* milestone;
* documents read;
* files changed;
* behavior implemented;
* runtime evidence;
* tests passed;
* tests failed;
* tests skipped;
* limitations;
* security findings;
* remaining work;
* exact current enforcement level.

Do not claim completion for behavior not proven by code and runtime evidence.

## Stop conditions

Stop and report instead of guessing when:

* requirements conflict;
* a destructive migration is required;
* credentials are unavailable;
* a required external service cannot be tested;
* production access would be needed;
* locked documents would need modification;
* the requested work would weaken a security invariant;
* the milestone cannot be proven in the current environment.

