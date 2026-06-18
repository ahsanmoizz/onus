# Phase 16 — Enforcement Coverage

**Date:** 2026-06-18
**Auditor:** OpenClaude (deepseek-chat)
**Branch:** `codex/phase15-integrations`
**Commit:** `aa38749`

---

## Enforcement Layer Definition

| Layer | Name | Description | Scope |
|---|---|---|---|
| L1 | BEST-EFFORT | Cooperative hooks that the agent can ignore | IDE extensions, cursor_hook.rs |
| L2 | Agent-firewall | Onus core: prompt intake, policy engine, approval broker, semantic review, quality | All actions routed through Onus |
| L3 | Containment | Process/filesystem/network isolation via workspace | Linux bubblewrap only |
| L4 | Authority | Credential broker, capability binding, tamper-evident receipts | Not wired to any surface yet |

---

## Per-Surface Enforcement

| Surface | L1 | L2 | L3 | L4 | Effective |
|---|---|---|---|---|---|
| Claude Code CLI | Hook (doctor fixed) | Through Onus binary | Linux only | — | L2 native, L3 on Linux |
| OpenAI Codex CLI | — | Through Onus binary | Linux only | — | L2 native, L3 on Linux |
| Gemini CLI | — | Through Onus binary | Linux only | — | L2 native, L3 on Linux |
| Cursor CLI | — | Through Onus binary | Linux only | — | L2 native, L3 on Linux |
| Continue CLI | — | Through Onus binary | Linux only | — | L2 native, L3 on Linux |
| JetBrains Junie CLI | — | Through Onus binary | Linux only | — | L2 native, L3 on Linux |
| Aider | — | Through Onus binary | Linux only | — | L2 native, L3 on Linux |
| VS Code Agents | Post-hoc events | — | Linux only | — | L1 only (platform limit) |
| Cline for VS Code | Post-hoc events | — | Linux only | — | L1 only (platform limit) |
| Antigravity | Post-hoc events | — | Linux only | — | L1 only (platform limit) |
| Cursor IDE | MCP/approval API | Through Onus binary | Linux only | — | L2 via MCP, L3 on Linux |
| Continue VS Code | Post-hoc events | Through Onus binary | Linux only | — | L1+L2, L3 on Linux |
| Continue JetBrains | Post-hoc events | Through Onus binary | Linux only | — | L1+L2, L3 on Linux |
| Junie IDE | ACP/approval API | Through Onus binary | Linux only | — | L2 via ACP, L3 on Linux |
| Windsurf Editor | Post-hoc events | — | Linux only | — | L1 only (platform limit) |
| Cursor Background Agents | — | — | — | Boundary defined | L4 design ready |
| GitHub Copilot SDK | — | Onus-owned executor | Linux only | — | L2, L3 on Linux |
| OpenAI Agents SDK | — | Tool guardrail | Linux only | — | L2, L3 on Linux |
| LangChain/LangGraph | — | StructuredTool/callback | Linux only | — | L2, L3 on Linux |
| CrewAI | — | Before-tool interceptor | Linux only | — | L2, L3 on Linux |

---

## Component Enforcement

| Component | Layer | Enforcement Type | Status |
|---|---|---|---|
| Prompt Intake Guardian | L2 | Find → classify → reject/contract | IMPLEMENTED |
| Task Contract | L2 | Canonical hash, evaluation, completion | IMPLEMENTED |
| Approval Broker | L2 | 5 decisions, 3 modes, deterministic supremacy | IMPLEMENTED |
| Semantic Reviewer | L2 | 5 roles, deterministic fallback, fail-closed | IMPLEMENTED |
| Security & Redaction | L2 | 17+12 patterns, environment identity | IMPLEMENTED |
| Quality Verifier | L2 | 10 evidence items, test integrity | IMPLEMENTED |
| Memory Isolation | L2 | AES-256-GCM, provenance, session isolation | IMPLEMENTED |
| L3 Workspace | L3 | Bubblewrap, env filter, resource limits | Linux only |
| L4 Authority | L4 | Disposable DB, capabilities, receipt chain | Design ready, not wired |
| Policy Engine | L1+L2 | Rules loading → Verdict | IMPLEMENTED |
| MCP Proxy | L2 | Tool mediation | IMPLEMENTED |
| IPC | L2 | Client/server protocol | IMPLEMENTED |
| Scope Tracker | L2→L3 | Filesystem scope, drift detection | IMPLEMENTED |

---

## Coverage Map

```
                    ┌─────────────────────────────────────┐
                    │          User / Agent Action          │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
             ┌─────│    L1: Cooperative Hooks (BEST-EFFORT)│
             │     │  Cursor_hook, IDE extensions, etc.    │
             │     └──────────────┬──────────────────────┘
             │                    │ (may bypass)
             │     ┌──────────────▼──────────────────────┐
             │     │   L2: Agent-firewall (Onus Core)     │
             │     │  ┌──────────┐  ┌────────────────┐   │
             │     │  │ Prompt   │  │ Task Contract  │   │
             │     │  │ Intake   │──│ Hash binding   │   │
             │     │  │ Guardian │  │ Evaluation     │   │
             │     │  └──────────┘  └────────────────┘   │
             │     │  ┌──────────┐  ┌────────────────┐   │
             │     │  │ Approval │  │ Semantic       │   │
             │     │  │ Broker   │  │ Reviewer (5    │   │
             │     │  │ 5 decis. │  │ roles + fallbk)│   │
             │     │  └──────────┘  └────────────────┘   │
             │     │  ┌──────────┐  ┌────────────────┐   │
             │     │  │ Security │  │ Quality        │   │
             │     │  │ & Redact │  │ Verifier       │   │
             │     │  └──────────┘  └────────────────┘   │
             │     └──────────────┬──────────────────────┘
             │                    │ (action allowed/denied)
             │     ┌──────────────▼──────────────────────┐
             │     │   L3: Containment (Linux only)      │
             │     │  Bubblewrap, resource limits,       │
             │     │  filtered environment, checkpoints   │
             │     └──────────────┬──────────────────────┘
             │                    │ (execution within boundary)
             │     ┌──────────────▼──────────────────────┐
             │     │   L4: Authority (design ready)      │
             │     │  Credential broker, capability       │
             │     │  binding, receipt chain              │
             │     └──────────────┬──────────────────────┘
             │                    │
             │     ┌──────────────▼──────────────────────┐
             └─────│        Action Executed               │
                   └─────────────────────────────────────┘
```

---

## Coverage Summary

| Layer | Implementation Status | Surfaces Covered | Gap |
|---|---|---|---|
| L1 | 5 extensions deployed | IDE agents (5) | All L1 only — post-hoc events |
| L2 | Full Rust core + Python SDK | All CLI + SDK surfaces | Requires agent to route through Onus |
| L3 | Bubblewrap (Linux) | All surfaces via workspace | Not available on Windows |
| L4 | Disposable DB + capabilities | Not surface-wired | No L4 claim made for any surface |

**Effective enforcement for a Linux deployment:** L2 (all surfaces through Onus binary) + L3 (containment for bypass paths).

**Effective enforcement for a Windows deployment:** L2 only. L1 is BEST-EFFORT. L3 is not available (fail-closed).
