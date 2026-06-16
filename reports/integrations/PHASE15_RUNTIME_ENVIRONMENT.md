# Phase 15B — Runtime Environment Inventory

Date: 2026-06-17

## Current Branch

`codex/phase15-integrations` at commit `c349dd6`.

---

## Priority Surface Inventory

### 1. Claude Code CLI

| Field | Value |
|---|---|
| installed | yes — `npx @anthropic-ai/claude-code` |
| version | 2.1.177 |
| path | ephemeral via npx |
| authenticated | yes — `ANTHROPIC_API_KEY` present |
| executable | `npx -y @anthropic-ai/claude-code` |
| testable now | yes — local interactive agent |
| adapter exists | `onus/src/cli/claude_hook.rs` + install scripts |
| Phase 15 status | `VERIFIED WITH LIMITATIONS` |
| runtime proof needed | live hook delivery, denied-tool behavior, correction round-trip, approval binding |

### 2. Windsurf Editor / Cascade

| Field | Value |
|---|---|
| installed | no — `D:\Windsurf\bin\devin-desktop` exists but is a VS Code fork branded "Devin Desktop", not Windsurf/Cascade |
| version | N/A |
| authenticated | N/A |
| testable now | no |
| missing requirement | actual Windsurf Editor installation |
| safe installation | not available without download |
| adapter exists | `integrations/windsurf-cascade/README.md` (protocol only) |
| Phase 15 status | `PROTOCOL ONLY` |
| runtime proof possible | no — software not installed |

### 3. Visual Studio Code Agents

| Field | Value |
|---|---|
| installed | yes — VS Code `1.124.2`, `code` CLI available |
| version | 1.124.2 (commit 6928394f9) |
| authenticated | N/A (local) |
| executable | `code` at `C:\Users\A\AppData\Local\Programs\Microsoft VS Code\bin\code.cmd` |
| testable now | yes — local IDE surface |
| adapter exists | `onus/bindings/vscode/extension.js` + `package.json` |
| Phase 15 status | `VERIFIED WITH LIMITATIONS` |
| runtime proof needed | extension installation, activation, interception of terminal/task/shell operations |
| note | VS Code extensions probe returned empty list — no Onus extension installed. Windsurf/Devin/Antigravity/Kiro on PATH are VS Code forks sharing the VS Code CLI shim pattern. |

### 4. Cline

| Field | Value |
|---|---|
| installed | no — `npm list -g @cline/cline` returned empty |
| version | N/A |
| authenticated | N/A |
| testable now | no |
| missing requirement | VS Code extension `saoudrizwan.claude-dev` not installed |
| adapter exists | `integrations/cline/README.md` (protocol only) |
| Phase 15 status | `PROTOCOL ONLY` |
| runtime proof possible | no — not installed |

### 5. OpenAI Codex CLI

| Field | Value |
|---|---|
| installed | no — binary `codex`/`codex-cli` not in PATH; `C:\Program Files\Codex\` absent |
| version | N/A |
| authenticated | N/A |
| testable now | no |
| missing requirement | Windows app binary not installed or not executable |
| Phase 15 status | `PROTOCOL ONLY` |
| runtime proof possible | no — not installed |

### 6. Gemini CLI

| Field | Value |
|---|---|
| installed | no — `gemini` not in PATH; Google Cloud CLI absent |
| version | N/A |
| authenticated | N/A |
| testable now | no |
| missing requirement | `gemini-cli` npm package not installed |
| Phase 15 status | `PROTOCOL ONLY` |
| runtime proof possible | no — not installed |

### 7. LangChain Agents / LangGraph

| Field | Value |
|---|---|
| installed | no — `pip show langchain` / `langgraph` not found |
| version | N/A |
| authenticated | N/A |
| testable now | no — packages absent |
| API key available | yes — `OPENAI_API_KEY` present |
| safe installation | `pip install langchain langgraph langchain-openai` |
| adapter exists | `integrations/langchain-langgraph/README.md` (blocked) |
| Phase 15 status | `BLOCKED` |
| runtime proof possible | yes — after pip install (no subscription required; uses pay-per-token API) |

### 8. OpenAI Agents SDK

| Field | Value |
|---|---|
| installed | no — `pip show openai-agents` not found |
| version | N/A |
| authenticated | N/A |
| API key available | yes — `OPENAI_API_KEY` present |
| safe installation | `pip install openai-agents` |
| adapter exists | `integrations/openai-agents-sdk/README.md` (blocked) |
| Phase 15 status | `BLOCKED` |
| runtime proof possible | yes — after pip install (no subscription; pay-per-token API) |

---

## Additional Software Inventory

| Item | Path / Status |
|---|---|
| Rust toolchain | `cargo` available — onus builds pass |
| Python 3.12 | `python` available — 57/57 tests pass |
| Node.js | `node` available |
| npm | `npm` available |
| WSL 2 | installed (v2.7.8) |
| Docker | not installed |
| Onus debug binary | `onus/target/debug/onus.exe` (114 MB) |
| Onus release binary | `onus/target/release/onus.exe` (3.2 MB) |
| onus in PATH | no — must invoke via explicit path or `cargo run` |

---

## Inventory Summary

| Surface | Installed | Authenticated | Testable Now | Status |
|---|---|---|---|---|
| Claude Code CLI | yes | yes (ANTHROPIC_API_KEY) | yes | **SELECTED** |
| VS Code Agents | yes | N/A (local) | yes | candidate |
| LangChain/LangGraph | no | yes (OPENAI_API_KEY) | after pip install | candidate |
| OpenAI Agents SDK | no | yes (OPENAI_API_KEY) | after pip install | candidate |
| Windsurf/Cascade | no | N/A | no | skipped |
| Cline | no | N/A | no | skipped |
| OpenAI Codex CLI | no | N/A | no | skipped |
| Gemini CLI | no | N/A | no | skipped |

## Selected Surface for Stage 2

**Claude Code CLI** — installed, authenticated, adapter exists in Rust, Phase 15 report says `VERIFIED WITH LIMITATIONS` with the exact gap being "no live authenticated agent-loop run."

This is the highest-priority surface with the highest likelihood of successful runtime verification.
