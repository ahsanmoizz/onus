# Phase 15D Live Product Verification Matrix

**Date**: 2026-06-17
**Phase**: 15D
**Branch**: `codex/phase15-integrations`

## Legend

| Column | Description |
|--------|-------------|
| Installed | Product binary or runtime found on PATH or detected |
| Authenticated | Credentials present and functional |
| Version | Exact version string |
| Real product launched | Product process started with real agent |
| Real agent action tested | Agent made a real tool call through Onus |
| Deny proven | Onus blocked a destructive action and side effect did not occur |
| Correction proven | Agent received Onus correction and retried |
| Approval proven | Approval binding, payload integrity, and expiry verified |
| Bypass tested | Unwrapped call, direct filesystem, child process, alternate executable |
| L3 tested | L3 workspace containment proven for this surface |

## Statuses

- `LIVE PRODUCT VERIFIED`
- `LIVE PRODUCT VERIFIED WITH LIMITATIONS`
- `LIVE FRAMEWORK WRAPPER VERIFIED`
- `EXTENSION INSTALLED BUT NOT AGENT VERIFIED`
- `PROTOCOL VERIFIED ONLY`
- `IMPLEMENTED BUT NOT VERIFIED`
- `BLOCKED — USER INSTALLATION REQUIRED`
- `BLOCKED — USER AUTHENTICATION REQUIRED`
- `BLOCKED — SUBSCRIPTION REQUIRED`
- `BLOCKED — PLATFORM UNAVAILABLE`
- `FAILED`

---

## CLI and Terminal Agents

| Order | Exact surface | Installed | Authenticated | Version | Real product launched | Real agent action tested | Deny proven | Correction proven | Approval proven | Bypass tested | L3 tested | Final status | Blocker |
| ----: | ------------- | --------: | ------------: | ------- | --------------------: | -----------------------: | ----------: | ----------------: | --------------: | ------------: | --------: | ------------ | ------- |
| 1 | Claude Code CLI | Yes (npx) | No | — | No | No | No | No | No | No | No | `BLOCKED — USER AUTHENTICATION REQUIRED` | ANTHROPIC_API_KEY absent; requires user login |
| 2 | OpenAI Codex CLI | Binary present (locked) | No | — | No | No | No | No | No | No | No | `BLOCKED — USER INSTALLATION REQUIRED` | Binary locked by permissions |
| 3 | Gemini CLI | No | — | — | — | — | — | — | — | — | — | `BLOCKED — USER INSTALLATION REQUIRED` | `npm install -g @google-gemini/gemini-cli` |
| 4 | Cursor CLI | No | — | — | — | — | — | — | — | — | — | `BLOCKED — USER INSTALLATION REQUIRED` | Requires Cursor download from cursor.com |
| 5 | Continue CLI | No | — | — | — | — | — | — | — | — | — | `BLOCKED — USER INSTALLATION REQUIRED` | `npm install -g @continuedev/continue` |
| 6 | JetBrains Junie CLI | No | — | — | — | — | — | — | — | — | — | `BLOCKED — USER INSTALLATION REQUIRED` | Requires JetBrains Toolbox + Junie |
| 7 | Aider | No | — | — | — | — | — | — | — | — | — | `BLOCKED — USER INSTALLATION REQUIRED` | `pip install aider-chat` |

## IDE and Editor Agents

| Order | Exact surface | Installed | Authenticated | Version | Real product launched | Real agent action tested | Deny proven | Correction proven | Approval proven | Bypass tested | L3 tested | Final status | Blocker |
| ----: | ------------- | --------: | ------------: | ------- | --------------------: | -----------------------: | ----------: | ----------------: | --------------: | ------------: | --------: | ------------ | ------- |
| 8 | Windsurf Editor / Cascade | Windsurf binary absent. Devin Desktop found at `D:\Windsurf\bin\devin-desktop` — confirmed rebranded Windsurf (product.json: `oldNameShort: "Windsurf"`) | No | Devin Desktop v1.107.0 | No | No | No | No | No | No | No | `EXTENSION INSTALLED BUT NOT AGENT VERIFIED` | Onus extension deployed to user extensions dir |
| 9 | Visual Studio Code Agents | Yes | Yes | 1.124.2 | Yes (extension host via @vscode/test-electron) | No (terminal + task hooks are L1, cannot intercept Copilot agent tool calls) | No (L1: `onDidStartTask` fires after task starts) | No | No (extension has no approval binding) | No | `EXTENSION INSTALLED BUT NOT AGENT VERIFIED` | L1 cooperative hook only — can observe terminal/task events post-execution, cannot prevent Copilot agent tool calls |
| 10 | Cline for VS Code | No | — | — | — | — | — | — | — | — | — | `BLOCKED — USER INSTALLATION REQUIRED` | VS Code extension not installed |
| 11 | Google Antigravity | Installed at `/d/Antigravity/` | Yes (no auth needed) | VS Code fork v1.107.0 | Yes (extension listed, deployed at user extensions dir) | No (same L1 architecture as VS Code: `onDidStartTask` + `onDidChangeTerminalShellIntegration` post-execution hooks) | No (L1: post-execution hooks only) | No | No | No | `EXTENSION INSTALLED BUT NOT AGENT VERIFIED` | Onus extension registered and listed in Antigravity v1.107.0. Inherits same L1 limitations as VS Code extension (same source, same extension API). Can observe but cannot intercept Antigravity agent tool calls. |
| 12 | Cursor Agent in Cursor IDE | No | — | — | — | — | — | — | — | — | — | `BLOCKED — USER INSTALLATION REQUIRED` | Cursor IDE not installed |
| 13 | Continue Agent for VS Code | No | — | — | — | — | — | — | — | — | — | `BLOCKED — USER INSTALLATION REQUIRED` | Continue VS Code extension not installed |
| 14 | Continue Agent for JetBrains | No | — | — | — | — | — | — | — | — | — | `BLOCKED — PLATFORM UNAVAILABLE` | JetBrains IDE not installed |
| 15 | JetBrains Junie IDE Agent | No | — | — | — | — | — | — | — | — | — | `BLOCKED — PLATFORM UNAVAILABLE` | JetBrains IDE not installed |

## Remote and Background Surfaces

| Order | Exact surface | Installed | Authenticated | Version | Real product launched | Real agent action tested | Deny proven | Correction proven | Approval proven | Bypass tested | L3 tested | Final status | Blocker |
| ----: | ------------- | --------: | ------------: | ------- | --------------------: | -----------------------: | ----------: | ----------------: | --------------: | ------------: | --------: | ------------ | ------- |
| 16 | Cursor Background Agents | No | — | — | — | — | — | — | — | — | — | `BLOCKED — USER AUTHENTICATION REQUIRED` | Requires Cursor cloud subscription |

## SDK and Framework Surfaces

| Order | Exact surface | Installed | Authenticated | Version | Real product launched | Real agent action tested | Deny proven | Correction proven | Approval proven | Bypass tested | L3 tested | Final status | Blocker |
| ----: | ------------- | --------: | ------------: | ------- | --------------------: | -----------------------: | ----------: | ----------------: | --------------: | ------------: | --------: | ------------ | ------- |
| 17 | GitHub Copilot SDK | SDK registry-reachable (`@github/copilot-sdk@1.0.1`) | No | — | No | No | No | No | No | No | No | `BLOCKED — USER AUTHENTICATION REQUIRED` | No `gh` CLI; no GITHUB_TOKEN |
| 18 | OpenAI Agents SDK | Yes (pip) | Yes (OPENAI_API_KEY) | 0.17.5 | N/A (SDK, not product) | Yes (wrapper tests) | Yes (unit + live) | Yes (correction field proven) | Yes (action_id + payload hash) | Yes (raw func + invoker bypass proven) | No | `LIVE FRAMEWORK WRAPPER VERIFIED` | — |
| 19 | LangChain / LangGraph | Yes (pip) | Yes (OPENAI_API_KEY) | langchain-core 0.3.x, langgraph 0.4.x | N/A (SDK, not product) | Yes (wrapper tests) | Yes (unit + live) | Yes (correction field proven) | Yes (action_id + payload hash + approval_decision) | Yes (.func() + .invoke() bypass proven) | No | `LIVE FRAMEWORK WRAPPER VERIFIED` | — |
| 20 | CrewAI | No | — | — | — | — | — | — | — | — | — | `BLOCKED — USER INSTALLATION REQUIRED` | `pip install crewai` not run; model credentials absent |

---

## Summary

| Status | Count | Surfaces |
|--------|-------|----------|
| LIVE PRODUCT VERIFIED | 0 | — |
| LIVE FRAMEWORK WRAPPER VERIFIED | 2 | OpenAI Agents SDK, LangChain/LangGraph |
| EXTENSION INSTALLED BUT NOT AGENT VERIFIED | 3 | VS Code Agents (L1, 5 extension host tests pass), Google Antigravity (L1, extension deployed), Devin Desktop/Windsurf (extension deployed) |
| BLOCKED — USER INSTALLATION REQUIRED | 8 | Gemini CLI, Cursor CLI, Cursor IDE, Continue CLI, Continue VS Code, Junie CLI, Aider, CrewAI |
| BLOCKED — USER AUTHENTICATION REQUIRED | 3 | Claude Code CLI, GitHub Copilot SDK, Cursor Background Agents |
| BLOCKED — PLATFORM UNAVAILABLE | 2 | Continue JetBrains, Junie IDE Agent |
| BLOCKED — OTHER | 2 | OpenAI Codex CLI (binary locked), Cline for VS Code (not installed) |
| FAILED | 0 | — |


## Prioritization

Surfaces by next-action tier:

**Tier 1 — User action required for verification**:
1. Claude Code CLI — provide ANTHROPIC_API_KEY or login
2. GitHub Copilot SDK — provide GITHUB_TOKEN
3. Gemini CLI — `npm install -g @google-gemini/gemini-cli`
4. Cline for VS Code — install from VS Code marketplace
5. Cursor IDE/CLI — download from cursor.com
6. Continue for VS Code — install from marketplace

**Tier 2 — Currently automated, fully verified**:
7. OpenAI Agents SDK — 16 tests pass, bypass/approval/fail-closed verified
8. LangChain/LangGraph — 21 tests pass, bypass/approval/fail-closed verified

**Tier 3 — Extensions deployed but agent-testing blocked (all L1 only)**:
9. VS Code Agents — 5 extension host tests pass. L1 = cannot block Copilot agent tool calls
10. Google Antigravity — extension deployed, L1 only
11. Devin Desktop/Windsurf — extension deployed, L1 only
