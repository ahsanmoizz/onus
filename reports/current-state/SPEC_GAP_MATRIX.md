# Onus Specification Gap Matrix

Date: 2026-06-14

Classification key: VERIFIED, VERIFIED WITH LIMITATIONS, PARTIAL, MOCKED, DEMO ONLY, DOCUMENTED ONLY, MISSING, BROKEN, UNVERIFIABLE.

## Product-Level Gap Matrix

| Spec / claim area | Required by docs | Current classification | Evidence | Gap to target |
| --- | --- | --- | --- | --- |
| Python-first SDK | Python SDK wraps actions before execution | VERIFIED WITH LIMITATIONS | `from onus import Guardian` works; editable package installed; demo exercises Guardian | Only protects voluntary SDK users. No broad framework adapters. |
| Core evaluator | Deterministic pre-action policy engine | VERIFIED WITH LIMITATIONS | `cargo test` 32 passed; evaluator probes block/escalate/warn/allow | Rule coverage incomplete; some failure paths allow. |
| File write before/after | Capture current state and proposed new state | VERIFIED | Demo printed exact before/after contents and DB payload includes both | Stores raw content; no redaction. |
| Real-time interception | Block before execution, not after-the-fact | VERIFIED WITH LIMITATIONS | Guardian blocks before its own file/shell/API/SQLite execution | Not mandatory for external agents; VS Code terminal path is after/best-effort. |
| API call protection | API calls should be governed | PARTIAL | Guardian `api_call` checks before urllib request; demo local API call | No broad HTTP client integration, credential policy, or API mutation classification. |
| DB mutation protection | DB mutations should be governed | VERIFIED WITH LIMITATIONS | Guardian SQLite insert allowed; `DROP TABLE` escalated | SQLite helper only; no production DB identity, migration diff, transaction plan. |
| Audit ledger | Real DB storing governed actions | VERIFIED | SQLite rows queried from demo DB | Local writable DB; no remote anchoring or protected signing key. |
| Tamper-evident hash chain | SHA-256 chain of actions | VERIFIED WITH LIMITATIONS | `onus verify` returned ALL PASS; source recomputes hashes | Hash chain alone is not immutability; attacker can rewrite DB and recompute. |
| Merkle root | Session-level Merkle evidence | PARTIAL | Merkle module and tests exist | No observed persisted `merkle_roots` table/session-close anchoring in runtime path. |
| Session replay | Replay real past session step by step | VERIFIED WITH LIMITATIONS | `onus session ...` showed 7 real actions | Metadata is generic; payload previews truncate; no richer semantic timeline. |
| Risk classification | Reversible/compensable/irreversible | PARTIAL | Rule schema and response include `reversibility`; probes return values | Static rule metadata, not a full semantic/action recovery classifier. |
| Guardian approval gate | Risky action pauses for approval | PARTIAL | `escalate` verdict halts Guardian DB drop before execution | No live human approval flow through Guardian; escalation is an exception, not resume. |
| Approval UI | UI renders and can display pending actions | PARTIAL | `onus approvals` served UI and `/api/pending` | No auth/CSRF; not proven end-to-end with MCP; approval binding is weak. |
| Exact approval binding | Approval must bind exact canonical payload hash | BROKEN / MISSING | MCP code searches pending approval by `session_id + tool_name`; no payload hash match | Must bind session/task/action/payload/policy/env/expiry/approver exactly. |
| Prompt correction loop | Correction sent back to agent automatically | DEMO ONLY / PARTIAL | Demo `DemoAgent.receive_correction` gets deterministic text | No real LLM agent or provider call; no model-evaluated retry loop. |
| LLM semantic roles | Intent interpreter, critic, correction generator, verifier | MISSING | No provider code found; keys unset | Build provider abstraction, redaction, deterministic authority boundary. |
| Memory system | Bounded task/project memory | MISSING | No memory subsystem found | Required for target architecture but absent. |
| Evidence-based completion | Completion requires receipts/tests/evidence | MISSING | No completion verifier found | Needed to prevent fake completion. |
| Rollback | Tested restore/compensation paths | VERIFIED WITH LIMITATIONS | File rollback demo restored contents; SQLite backup code exists | Limited to SDK-managed file writes/deletes and SQLite backup. No general rollback. |
| Secret safety | Never store secrets in logs/receipts/prompts/dashboard | MISSING / BROKEN | Hardcoded secret file-write probe allowed; payloads stored/served raw | Need detection, redaction, field classification, encrypted/segmented storage. |
| Test weakening detection | Do not silently accept test deletion/weakening | MISSING | `file_delete tests/test_auth.py` probe returned allow | Need diff/AST/test policy and acceptance tests. |
| L1 cooperative hooks | Best-effort hooks clearly labeled | PARTIAL | Claude adapter, VS Code extension, shell wrapper exist | Live external-agent proof absent; some docs/marketing overstate. |
| L2 Onus-routed execution | Actions routed through Onus enforced | VERIFIED WITH LIMITATIONS | Guardian path works; MCP proxy code exists | MCP default policy does not block destructive payloads; external routing not proven. |
| L3 containment | Process/filesystem/network/credential containment | MISSING | No containment implementation found | Cannot claim. |
| L4 authority broker | Onus-controlled credentials/authority | MISSING | No credential/authority broker found | Cannot claim. |

## Acceptance Scenario Matrix

| Acceptance scenario | Current classification | Evidence | Gap |
| --- | --- | --- | --- |
| A - vague request | MISSING | No intent interpreter/task clarification engine found | Need prompt intake, ambiguity detection, clarification/correction loop. |
| B - deleted tests | MISSING | Direct evaluator probe deleting `tests/test_auth.py` returned `allow` | Need test-delete/test-weaken detection and tests. |
| C - hardcoded secret | MISSING / BROKEN | Direct evaluator probe writing `AWS_SECRET_ACCESS_KEY` returned `allow` | Need secret scanner and redaction. |
| D - low-risk IDE approval | PARTIAL | VS Code extension and approval UI exist | IDE path not true pre-exec for terminal/tasks; no live IDE proof. |
| E - changed approval payload | BROKEN / MISSING | MCP approval lookup uses session/tool; no canonical payload hash binding | Need immutable approval token over exact payload. |
| F - production migration | PARTIAL | DB destructive SQL escalates | No production env identity, migration diff, rollback plan, or approval contract. |
| G - incomplete work | MISSING | No evidence/completion verifier found | Need evidence receipts and independent verification. |
| H - failed implementation | PARTIAL | File rollback works in demo | No general checkpoint restore, no semantic failure detector. |

## Architecture Gap Matrix

| Target architecture item | Current classification | Evidence | Required next milestone |
| --- | --- | --- | --- |
| Deterministic safety kernel | VERIFIED WITH LIMITATIONS | Rust policy engine and TOML rules | Expand rules and fail-closed behavior. |
| Scope tracker | PARTIAL | `scope/tracker.rs` has declared/allowed path logic and tests | Wire into all execution paths and enforce outcomes. |
| Approval Decision Broker | DOCUMENTED ONLY / PARTIAL | Pending approval table + local UI | Implement canonical payload hash binding and authenticated approval API. |
| Agent adapter layer | PARTIAL | Claude translator, VS Code extension, shell wrapper, SDK, MCP proxy | Add live integration tests per adapter. |
| MCP middleware | PARTIAL / BROKEN DEFAULT POLICY | Proxy intercepts `tools/call`; default `m_c_p` probe allowed | Add MCP action normalization/rules and full fake MCP integration tests. |
| Model provider architecture | MISSING | `rg` found no OpenAI/Anthropic/Gemini provider code; env keys unset | Build provider interface with redaction and deterministic boundary. |
| Correction generator | PARTIAL | Rule correction strings returned | No LLM-generated correction; no semantic repair. |
| Independent verifier | MISSING | No verifier code found | Implement post-action evidence verifier. |
| Recovery/checkpoint manager | PARTIAL | SDK rollback stack/journal and SQLite backup | Add transactional checkpoints and verified restore reports. |
| Dashboard/control plane | PARTIAL | Local dashboard serves audit actions | Add auth, redaction, session UX, approval integration. |
| Spec integrity enforcement | VERIFIED WITH LIMITATIONS | `tools/spec_lock/*`, `.github/workflows/spec-lock.yml`, `tests/test_spec_lock.py` exist | Confirm branch protection in hosted repo; local enforcement can be bypassed. |

## Honest Claim Boundary

Claimable now:

- Onus has a working Rust deterministic evaluator and CLI.
- Onus has a Python SDK/Guardian that can govern file writes, shell commands, local API calls, and SQLite mutations when the caller routes through Guardian.
- Onus records governed actions in a local SQLite audit DB and verifies a SHA-256 hash chain.
- Onus includes prototype integration surfaces for Claude Code, VS Code, shell, and MCP.

Not claimable now:

- Onus universally protects Claude Code, Cursor, Windsurf, Copilot, or all MCP agents.
- Onus is production-ready or tamper-proof.
- Onus has an LLM-evaluated semantic correction loop.
- Onus prevents hardcoded secrets or deleted tests.
- Onus has exact approval binding.
- Onus provides L3 containment or L4 authority control.
