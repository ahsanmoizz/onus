# Phase 16 — Threat Model Update

**Date:** 2026-06-18
**Auditor:** OpenClaude (deepseek-chat)
**Branch:** `codex/phase15-integrations`
**Commit:** `aa38749`

---

## Scope

This document updates the Onus threat model based on Phase 16 audit evidence. It records which threats are mitigated, which remain unmitigated, and which were newly identified during this audit.

---

## 1. Threat: Malicious Prompt (Vibe Coding)

| Attribute | Value |
|---|---|
| ID | T-PROMPT-01 |
| Description | User submits ambiguous/destructive prompt that would cause agent harm |
| Mitigation | Prompt Intake Guardian: 9 finding categories, 4 IntakeStatus values |
| Test evidence | test_dangerously_broad, test_destructive_wording, test_delete_tests, test_expose_secrets, test_disable_security, test_direct_production |
| Residual risk | Attacker crafts prompt outside keyword patterns but semantically dangerous — mitigated by semantic reviewer (if configured) |
| Status | **MITIGATED (L2)** |

## 2. Threat: Test Weakening

| Attribute | Value |
|---|---|
| ID | T-QUALITY-01 |
| Description | Agent silently weakens tests (skip, .only(), xfail) or deletes tests |
| Mitigation | quality.rs: 12 skip patterns, 9 assertion patterns, deletion detection |
| Test evidence | test_skip_count, test_assertion_count, test_detect_test_deletion |
| Status | **MITIGATED (L2)** |

## 3. Threat: Secret Exposure

| Attribute | Value |
|---|---|
| ID | T-SECRET-01 |
| Description | Hardcoded secrets committed to repository |
| Mitigation | prompt_intake.rs: ExposeSecrets; security.rs: 17+12 redaction patterns; quality.rs: secret_scan |
| Test evidence | test_redact_secrets, test_classify_payload, test_secret_exposure_in_prompt |
| Residual risk | Zero-day secret pattern not in keyword list |
| Status | **MITIGATED (L2)** |

## 4. Threat: Approval Reuse

| Attribute | Value |
|---|---|
| ID | T-APPROVAL-01 |
| Description | Previously approved action replayed with modified payload |
| Mitigation | task_contract.rs: canonical_payload_hash; authority.rs: capability binding + one-time use + TTL |
| Test evidence | test_approval_binding, test_changed_payload_rejection, test_replay_prevention |
| Status | **MITIGATED (L2+L4)** |

## 5. Threat: LLM Override of Policy

| Attribute | Value |
|---|---|
| ID | T-POLICY-01 |
| Description | Attacker uses prompt injection to make semantic reviewer override deterministic denial |
| Mitigation | approval_broker.rs: Verdict::Block → DenyWithCorrection (non-overridable); semantic.rs: deterministic fallback |
| Test evidence | test_deterministic_policy_supremacy, test_fail_closed_on_critical_evaluator_failure |
| Status | **MITIGATED (L2)** |

## 6. Threat: Semantic Reviewer Failure (Silent Open)

| Attribute | Value |
|---|---|
| ID | T-SEMANTIC-01 |
| Description | Semantic reviewer crashes/timeouts and action is allowed by default |
| Mitigation | semantic.rs: critical roles FailClosed; non-critical → deterministic fallback |
| Test evidence | test_fail_closed_on_binary_crash, test_fail_closed_on_binary_timeout |
| Status | **MITIGATED (L2)** |

## 7. Threat: L3 Bypass (Shell Escape)

| Attribute | Value |
|---|---|
| ID | T-L3-01 |
| Description | Agent escapes controlled environment via shell, network, or filesystem |
| Mitigation | workspace.rs: bubblewrap isolation (Linux), setrlimit resource limits, 7-variable filtered environment |
| Test evidence | test_run_isolate (Linux only), test_fail_closed_without_linux_boundary (Windows) |
| Status | **MITIGATED (Linux L3) / UNMITIGATED (Windows)** |

## 8. Threat: Credential Theft from Memory Store

| Attribute | Value |
|---|---|
| ID | T-MEMORY-01 |
| Description | Attacker reads sensitive values from SQLite memory database |
| Mitigation | memory.rs: AES-256-GCM encryption for value_ciphertext; nonce+encrypted format |
| Test evidence | test_memory_encryption |
| Status | **MITIGATED (L2)** |

## 9. Threat: Authority Replay / Forgery

| Attribute | Value |
|---|---|
| ID | T-AUTH-01 |
| Description | Attacker reuses captured capability token or forges receipts |
| Mitigation | authority.rs: one-time use, status="used", TTL, previous_receipt_hash chain, environment identity binding |
| Test evidence | test_replay_prevention, test_capability_expiry, test_compensation_chain |
| Status | **MITIGATED (L4)** |

## 10. Threat: Config File Tampering (NEW — found this audit)

| Attribute | Value |
|---|---|
| ID | T-CONFIG-01 |
| Description | Attacker with filesystem write access modifies policy rules or config files |
| Mitigation | None — rules loaded from filesystem without integrity verification |
| Residual risk | Requires existing filesystem access; attacker at that level can already cause damage |
| Status | **UNMITIGATED** — requires config signing feature |

## 11. Threat: Environment Identity Spoofing (NEW — found this audit)

| Attribute | Value |
|---|---|
| ID | T-ENV-01 |
| Description | Attacker sets ONUS_ENVIRONMENT_IDENTITY to impersonate another environment |
| Mitigation | Fallback hash includes user+cwd; env var is primary, override by host-level access |
| Status | **PARTIALLY MITIGATED** — relies on host trust boundary |

## 12. Threat: DoS via Memory Flood (NEW — found this audit)

| Attribute | Value |
|---|---|
| ID | T-DOS-01 |
| Description | Attacker floods memory store to cause performance degradation or storage exhaustion |
| Mitigation | SQLite constraints provide backpressure; no explicit rate limiting or quota enforcement |
| Status | **PARTIALLY MITIGATED** — SQLite write throughput limits natural rate |

## 13. Threat: L1 Cooperative Hook Bypass

| Attribute | Value |
|---|---|
| ID | T-L1-01 |
| Description | Agent ignores cooperative hook or runs without hook installed |
| Mitigation | Explicitly labeled BEST-EFFORT; L3 containment for Linux; no containment on Windows |
| Test evidence | cursor_hook.rs: L1 label; test_direct_bypass documents uncontained paths |
| Status | **DOCUMENTED LIMITATION** — L1 is cooperative; Linux L3 available for containment |

## 14. Threat: SDK Wrapper Bypass (Direct Function Call)

| Attribute | Value |
|---|---|
| ID | T-SDK-01 |
| Description | Code calls underlying SDK function directly, bypassing Onus wrapper |
| Mitigation | Documented as bypass; no prevention outside L3 containment |
| Test evidence | test_func_bypass, test_invoke_bypass, test_raw_python_function_bypass |
| Status | **DOCUMENTED LIMITATION** — SDK wrappers are L2 cooperative |

---

## Threat Summary

| Status | Count |
|---|---|
| MITIGATED (L2/L3/L4) | 9 |
| PARTIALLY MITIGATED | 3 (env identity, memory flood, semantic prompt injection) |
| UNMITIGATED | 1 (config file integrity) |
| DOCUMENTED LIMITATION | 2 (L1 bypass, SDK wrapper bypass) |

**New threats found during Phase 16 audit:** T-CONFIG-01, T-ENV-01, T-DOS-01 — all are medium-severity.

**No critical unmitigated threats.** Primary deployment requirement for full security posture: **Linux host with bubblewrap** (for L3 containment).
