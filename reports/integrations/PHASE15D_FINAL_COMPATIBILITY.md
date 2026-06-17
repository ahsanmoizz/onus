# Phase 15D — Final Compatibility Report

**Date**: 2026-06-17
**Branch**: `codex/phase15-integrations`
**Phase**: 15D — Live Product Verification (20-surface audit)

---

## Executive Summary

Phase 15D performed live product verification across all 20 surfaces in the Onus integration matrix. Zero (`LIVE PRODUCT VERIFIED`) surfaces were achieved — no single surface had a real agent launched through Onus with deny/correction/approval/bypass proven end-to-end.

### By the numbers

| Metric | Value |
|--------|-------|
| Total surfaces | 20 |
| LIVE PRODUCT VERIFIED | 0 |
| LIVE FRAMEWORK WRAPPER VERIFIED | 2 (OpenAI Agents SDK, LangChain/LangGraph) |
| EXTENSION INSTALLED BUT NOT AGENT VERIFIED | 3 (VS Code, Antigravity, Devin Desktop) |
| BLOCKED — INSTALLATION REQUIRED | 8 |
| BLOCKED — AUTHENTICATION REQUIRED | 3 |
| BLOCKED — PLATFORM UNAVAILABLE | 2 |
| BLOCKED — OTHER | 2 |
| Tests added | 12 (6 OpenAI SDK + 6 LangChain) |
| Extension verification tests added | 5 (VS Code extension host) |
| Full regression | 103 passed, 2 skipped, 0 failed |

---

## Results by category

### CLI and Terminal Agents (Surfaces 1-7)

| Surface | Status | Blocker |
|---------|--------|---------|
| Claude Code CLI | `BLOCKED — AUTH REQUIRED` | `ANTHROPIC_API_KEY` not configured |
| Gemini CLI | `BLOCKED — INSTALL REQUIRED` | `npm install -g @google-gemini/gemini-cli` |
| Cursor CLI | `BLOCKED — INSTALL REQUIRED` | Download from cursor.com |
| Junie CLI | `BLOCKED — INSTALL REQUIRED` | `npm install -g @jetbrains/junie` |
| OpenAI Codex CLI | `BLOCKED — OTHER` | Binary locked, requires OpenAI account |
| Aider | `BLOCKED — INSTALL REQUIRED` | `pip install aider-chat` |
| Continue CLI | `BLOCKED — INSTALL REQUIRED` | `npm install -g continue` |

**Key finding**: All CLI surfaces require user action (install or auth). None are testable in current environment.

### IDE Agents (Surfaces 8-15)

| Surface | Status | Evidence |
|---------|--------|----------|
| Windsurf/Devin Desktop | `EXTENSION INSTALLED BUT NOT AGENT VERIFIED` | Extension deployed to user extensions dir |
| VS Code Agents | `EXTENSION INSTALLED BUT NOT AGENT VERIFIED` | 5 extension host tests pass. **L1 only** — cannot intercept Copilot agent tool calls |
| Google Antigravity | `EXTENSION INSTALLED BUT NOT AGENT VERIFIED` | Extension deployed. **L1 only** — VS Code fork, same architectural limits |
| Cursor IDE | `BLOCKED — INSTALL REQUIRED` | Not installed |
| Continue VS Code | `BLOCKED — INSTALL REQUIRED` | Not installed |
| Junie IDE Agent | `BLOCKED — PLATFORM UNAVAILABLE` | JetBrains-only |
| Continue JetBrains | `BLOCKED — PLATFORM UNAVAILABLE` | JetBrains plugin |
| Cline for VS Code | `BLOCKED — OTHER` | Not installed |

**Key finding**: All IDE extension surfaces are L1 best-effort cooperative hooks. The VS Code extension uses `onDidStartTask` (fires after task starts) and `onDidChangeTerminalShellIntegration` (fires after shell starts). None can pre-emptively intercept VS Code Agent / Copilot tool calls. Deny-at-source (L2+) would require a VS Code API that does not exist — Onus can only post-hoc observe and report.

### Remote / Background Agents (Surfaces 16-17)

| Surface | Status | Blocker |
|---------|--------|---------|
| GitHub Copilot SDK | `BLOCKED — AUTH REQUIRED` | `GITHUB_TOKEN` not configured |
| Cursor Background Agents | `BLOCKED — AUTH REQUIRED` | Cursor account required |

### SDK / Framework Wrappers (Surfaces 18-20)

| Surface | Status | Tests | Bypass | Approval | Fail-closed |
|---------|--------|-------|--------|----------|-------------|
| OpenAI Agents SDK v0.17.5 | `LIVE FRAMEWORK WRAPPER VERIFIED` | 16/16 | Proven (raw func + invoker) | action_id + canonical_payload_hash | binary crash → OnusEvaluationError |
| LangChain/LangGraph | `LIVE FRAMEWORK WRAPPER VERIFIED` | 21/21 | Proven (.func() + .invoke()) | action_id + canonical_payload_hash + approval_decision | binary crash → OnusEvaluationError |
| CrewAI | `BLOCKED — INSTALL REQUIRED` | — | — | — | — |

**Key finding**: Only the SDK wrapper surfaces can be tested autonomously. Both now have comprehensive bypass, approval binding, and fail-closed tests. But the wrappers are **advisory** — the agent framework can call the underlying function directly, bypassing Onus entirely.

---

## Critical security findings

### 1. All IDE extensions are L1 only (cannot block)

Every VS Code-compatible extension (VS Code, Antigravity, Devin Desktop) uses `onDidStartTask` and `onDidChangeTerminalShellIntegration` — both post-execution events. The VS Code extension API does not expose a pre-execution interception hook for agent tool calls. **Onus cannot block destructive actions from VS Code Copilot/Chat agents.**

Proof from `src/extension.js:248`:
```javascript
context.subscriptions.push(
    vscode.tasks.onDidStartTask(...) // fires AFTER task starts
);
```

### 2. SDK wrappers are advisory

The OpenAI Agents SDK and LangChain wrappers intercept `evaluate()` calls, but the underlying tool functions can be invoked directly without going through Onus. Bypass tests confirm this:
- `function_tool.on_invoke_tool()` calls the raw Python function
- `StructuredTool.func()` and `StructuredTool.invoke()` call the raw function

### 3. Only the Rust binary enforces L2+ (cannot be tested without product integration)

The Onus Rust binary (`cargo run -- evaluate`) correctly denies destructive actions. But no CLI agent or IDE agent routes through it at L2+ enforcement:

| Expected L2+ | Actual | Gap |
|-------------|--------|-----|
| IDE extensions pre-intercept | `onDidStartTask` = post-exec | Cannot block |
| SDK wrappers enforce inline | `.func()` bypass O | Advisory only |
| CLI agent routes through proxy | No CLI agent tested | Not verified |

---

## Tests added during Phase 15D

| File | Tests added | Purpose |
|------|------------|---------|
| `test_openai_agents_sdk.py` | 6 | Bypass, approval binding, fail-closed for SDK wrapper |
| `test_langchain_langgraph.py` | 6 | Bypass, approval binding, fail-closed for SDK wrapper |
| `test/extension.test.js` | 5 | VS Code extension host activation, commands, config |
| `test/verify.js` | 32 checks | Standalone VS Code extension structural verification |

### Test results

| Suite | Passed | Failed | Skipped |
|-------|--------|--------|---------|
| OpenAI SDK tests | 16 | 0 | 0 |
| LangChain/LangGraph tests | 21 | 0 | 0 |
| VS Code extension host tests | 5 | 0 | 0 |
| VS Code verify script | 32 | 0 | 0 |
| Full Python regression | 103 | 0 | 2 |
| Rust lib tests (`cargo test --lib`) | 75 | 0 | 0 |
| **Total** | **252** | **0** | **2** |

---

## Pre-Phase-16 requirements

These must be addressed before Phase 16 (product release) begins:

1. **ANTHROPIC_API_KEY / GITHUB_TOKEN** — needed to test Claude Code CLI and GitHub Copilot SDK (Surfaces 1, 16)
2. **L2+ enforcement for VS Code agents** — onus-firewall extension cannot block Copilot tool calls. Requires `onus-agent-proxy` MCP server or VS Code API change
3. **Real agent launch test** — no surface has proven: agent → Onus → deny → side-effect-did-not-occur. This is the core security promise
4. **All 20 surfaces** — 13/20 remain blocked. Install 8 products, configure 3 auth tokens
5. **L3 containment** — zero surfaces tested with L3 workspace isolation

---

## Enforcement level audit

| Surface | Current level | Required level | Gap |
|---------|-------------|----------------|-----|
| VS Code extension | L1 (best-effort hook) | L2+ | Post-exec only; cannot pre-intercept |
| Antigravity | L1 (best-effort hook) | L2+ | VS Code fork, same limit |
| Devin Desktop | L1 (best-effort hook) | L2+ | Same extension API |
| OpenAI SDK wrapper | L0 (advisory) | L2+ | Bypassable via raw function call |
| LangChain wrapper | L0 (advisory) | L2+ | Bypassable via `.func()`/`.invoke()` |
| Onus Rust binary | L4 (own binary) | L4 | Correct — blocks destructive actions |

---

## Verification artifacts

| Artifact | Location |
|----------|----------|
| Phase 15C independent verification | `reports/integrations/PHASE15C_INDEPENDENT_VERIFICATION.md` |
| Phase 15D live product matrix | `reports/integrations/PHASE15D_LIVE_PRODUCT_MATRIX.md` |
| Phase 15D user actions | `reports/integrations/PHASE15D_USER_ACTIONS.md` |
| Phase 15D completion report | `reports/integrations/PHASE15D_FINAL_COMPATIBILITY.md` |
| Runtime verification workspace | `runtime-verification/` |
| VS Code extension verify | `onus/bindings/vscode/test/verify.js` |
| Extension host tests | `onus/bindings/vscode/test/extension.test.js` |
| Python SDK bypass tests | `onus/bindings/python/tests/test_openai_agents_sdk.py` |
| Python SDK bypass tests | `onus/bindings/python/tests/test_langchain_langgraph.py` |

---

*Phase 15D complete. 0/20 surfaces reached LIVE PRODUCT VERIFIED. 13/20 blocked by install/auth/platform. All IDE extensions L1 only. SDK wrappers advisory. Phase 16 cannot begin until requirements above are met.*
