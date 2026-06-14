# Security Bypass Audit

Date: 2026-06-14

## Executive Security Finding

Onus has useful pre-action enforcement when code voluntarily routes through the Python Guardian or CLI evaluator. It is not yet a reliable security boundary for arbitrary agents. The highest-risk gaps are fail-open behavior, raw secret/payload logging, missing exact approval binding, and MCP default-policy bypass.

## High-Severity Findings

| ID | Finding | Classification | Evidence | Impact |
| --- | --- | --- | --- | --- |
| SB-01 | No mandatory execution boundary | VERIFIED WITH LIMITATIONS | Guardian protects only calls routed through it; IDE/shell/MCP adapters are cooperative/prototype | Agents can bypass Onus by using direct tools, direct shell, direct DB/API clients, or direct MCP servers. |
| SB-02 | Secret leakage into audit/dashboard | BROKEN | Guardian payload stores `before_content`, `after_content`, headers/body previews; dashboard serves `payload`; secret-write probe returned `allow` | Secrets can be written, logged, and exposed in dashboard/API. |
| SB-03 | Critical fail-open paths | BROKEN | Python JSON parse failure returns allow; VS Code missing/eval failure returns allow; `evaluate_request` rules-load failure returns allow | Evaluator errors can silently permit dangerous actions. |
| SB-04 | Approval is not bound to exact payload | BROKEN / MISSING | MCP proxy checks existing approval by `session_id` and `tool_name`; no canonical payload hash comparison | An approval can potentially authorize a later different payload for the same tool/session. |
| SB-05 | MCP default-policy bypass | BROKEN | `tools/call` becomes `ActionType::MCP`; default rules have no `mcp` rules; dry-run `m_c_p` destructive payload returned `allow` | MCP proxy may forward destructive MCP actions by default. |
| SB-06 | MCP wire type inconsistency | BROKEN | External JSON action type `mcp` failed to parse; `m_c_p` parsed and allowed | Public wire format is inconsistent with rule config/display; integrations can misclassify or fail. |
| SB-07 | Local hash chain is not immutability | VERIFIED WITH LIMITATIONS | `onus verify` verifies SHA-256 chain, but DB is local writable and unsigned | An attacker with DB write access can alter rows and recompute hashes. |
| SB-08 | Approval/dashboard servers lack auth | PARTIAL / BROKEN FOR PRODUCTION | Local tiny_http servers; approval responses include `Access-Control-Allow-Origin: *` | Local malware/browser contexts may inspect or mutate approvals. Not enterprise-safe. |

## Medium-Severity Findings

| ID | Finding | Classification | Evidence | Impact |
| --- | --- | --- | --- | --- |
| SB-09 | VS Code terminal/task protection is best-effort | PARTIAL | Uses `onDidExecuteCommand` and `onDidStartTask`; evaluate failures allow | Not reliable pre-execution blocking. |
| SB-10 | Shell wrapper is cooperative and disable-able | PARTIAL | Wrapper script can be bypassed by not sourcing it or disabling environment | L1 only; should be labeled BEST-EFFORT. |
| SB-11 | DB rollback is not transactional | PARTIAL | Python SDK copies SQLite file before execution | Concurrent writers/WAL/stateful DBs may not restore cleanly. |
| SB-12 | API call inspection is shallow | PARTIAL | Guardian logs URL, headers, body preview; only URL-count heuristic observed | No method/path sensitivity, credential redaction, or production API approval model. |
| SB-13 | Hardcoded secret detection missing | MISSING | File write with `AWS_SECRET_ACCESS_KEY` returned `allow` | Fails acceptance scenario C. |
| SB-14 | Test deletion/weakening detection missing | MISSING | File delete of `tests/test_auth.py` returned `allow` | Fails acceptance scenario B. |
| SB-15 | Installer path bug risk | PARTIAL / BROKEN | Bash installer writes `/usr/local/bin/onus evaluate` even when install path is `$HOME/.local/bin` | Claude hook may point to wrong binary. |

## Security Invariant Compliance

| AGENTS.md invariant | Current status | Evidence |
| --- | --- | --- |
| Deterministic denial cannot be overridden by LLM | VERIFIED | No LLM override path exists. |
| Critical evaluator failure must not silently fail open | BROKEN | Python, VS Code, and library evaluator paths can return allow on failures. |
| Secrets must not appear in logs/receipts/prompts/dashboard | BROKEN | Raw payloads and file contents are stored and served. |
| Approval must bind exact canonical payload | BROKEN / MISSING | Approval lookup lacks payload hash. |
| Modified payloads require new approval | BROKEN / MISSING | No exact payload binding. |
| Production actions require verified environment identity | MISSING | No production env identity enforcement found. |
| Completion requires evidence | MISSING | No completion verifier found. |
| Test deletion/weakening must not be accepted silently | MISSING | Test delete probe allowed. |
| Rollback must not be claimed without tested restore | VERIFIED WITH LIMITATIONS | File rollback tested; general rollback not claimable. |
| Hash chaining alone must not be described as immutability | VERIFIED AS LIMITATION | Hash chain exists but must be described as tamper-evident only. |
| L1 hooks must be labeled BEST-EFFORT | PARTIAL | Some surfaces are best-effort; claims must be tightened. |
| L2 claims only for actions routed through Onus | VERIFIED WITH LIMITATIONS | Guardian path qualifies; external integrations not proven. |
| L3 claims require containment | MISSING | No containment found. |
| L4 claims require controlled authority/credentials | MISSING | No authority broker found. |

## Bypass Reproductions

Safe dry-run probes performed through `onus evaluate --rules D:\Onus\onus\rules\default.toml`:

- Hardcoded secret file write:
  - Input: file write with `AWS_SECRET_ACCESS_KEY="abc123"`.
  - Result: `{"decision":"allow"}`.
- Test deletion:
  - Input: file delete of `tests/test_auth.py`.
  - Result: `{"decision":"allow"}`.
- MCP destructive payload:
  - Input: action type `m_c_p`, payload containing `rm -rf /important`.
  - Result: `{"decision":"allow"}`.
- MCP public spelling:
  - Input: action type `mcp`.
  - Result: parse failure, not a normal decision.

Positive controls:

- Shell `rm -rf /important`: `block`, `SAFETY_001`.
- DB `DROP TABLE users;`: `escalate`, `SAFETY_009`.
- Shell `printenv`: `block`, `SAFETY_004`.

## Recommended Security Milestones

1. Make evaluator failures fail closed for production paths.
2. Add field-level redaction before audit persistence and dashboard responses.
3. Implement exact canonical payload hash approval binding.
4. Normalize action type serialization for `MCP` and add default MCP rules.
5. Add hardcoded-secret and test-delete/test-weaken acceptance tests.
6. Add authenticated local dashboard/approval APIs, or bind them to a protected localhost token.
7. Add live integration tests for Claude hook, VS Code, shell wrapper, and MCP proxy.
8. Add signed receipts or external anchoring before using "tamper-resistant" language.
