# V1 Integration Scope — Honest Support Classification

**Date**: 2026-06-17
**Branch**: `codex/phase15-integrations`
**Phase**: 15E delivery — Pre-Phase 16 gate closure

---

## Classification tiers

| Tier | Label | Meaning |
|------|-------|---------|
| **A** | SHIPS | Working, tested, documented. Ready for v1. |
| **B** | ADVISORY | Works but bypassable. Must be documented as advisory. |
| **C** | PROTOCOL | Protocol-level implementation exists. No real-agent test. |
| **D** | INSTALL | Requires user install/auth/subscription. Not testable here. |
| **E** | PLATFORM | Requires platform not available (Linux L3, JetBrains). |
| **F** | UNSUPPORTED | Cannot achieve enforcement. Architectural barrier. |

---

## Integration surface classification

### Python SDK

| Surface | Tier | Notes |
|---------|------|-------|
| **OpenAI Agents SDK wrapper** | A — SHIPS | 16/16 passing tests. Bypass via `.func()` documented. Approval binding proven. |
| **LangChain / LangGraph wrapper** | A — SHIPS | 21/21 passing tests. Bypass via `.func()`/`.invoke()` documented. Approval binding proven. |
| **CrewAI adapter** | A — SHIPS | 7/7 passing tests. `Tool(name=..., func=onus_wrapper)` constructor. Bypass via original Python function proven. |

### Onus binary (evaluate path)

| Surface | Tier | Notes |
|---------|------|-------|
| **CLI evaluate** | A — SHIPS | Block/allow proven. Deterministic denial proven. Start-contract protocol proven. |
| **Receipt chain** | A — SHIPS | Action IDs unique. Hashes deterministic. Approval expiry tested. |

### MCP proxy

| Surface | Tier | Notes |
|---------|------|-------|
| **MCP proxy** | C — PROTOCOL | Unit tests pass. No live agent test. Requires an MCP-supporting agent to configure proxy. |

### IDE / Editor extensions

| Surface | Tier | Notes |
|---------|------|-------|
| **VS Code extension** | C — PROTOCOL | Extension deploys, activates, registers commands. All hooks are L1 (post-hoc). MCP routing possible. 5 passing extension host tests. |
| **Antigravity** | D — INSTALL | Extension files deployed. Never agent-loaded. No verification. |
| **Devin Desktop** | D — INSTALL | Extension files deployed. Never agent-loaded. No verification. |
| **Cursor IDE** | D — INSTALL | Not installed. VS Code fork. MCP routing possible. |
| **Windsurf** | D — INSTALL | VS Code fork. MCP routing template exists. |
| **Continue (VS Code)** | D — INSTALL | Not installed. MCP routing possible for MCP tools. |
| **Continue (JetBrains)** | E — PLATFORM | JetBrains not installed. No plugin code exists. |
| **JetBrains Junie IDE** | E — PLATFORM | JetBrains not installed. No plugin code exists. |

### CLI agents

| Surface | Tier | Notes |
|---------|------|-------|
| **Claude Code CLI** | D — INSTALL | Hook protocol implemented. Requires ANTHROPIC_API_KEY + interactive login. |
| **Aider** | D — INSTALL | Not installed. No analysis performed. |
| **Gemini CLI** | D — INSTALL | Not installed. No analysis performed. |

### SDK / Framework

| Surface | Tier | Notes |
|---------|------|-------|
| **CrewAI (via Python adapter)** | A — SHIPS | Same as CrewAI adapter above. |
| **GitHub Copilot SDK** | F — UNSUPPORTED | No adapter exists. No credentials available. No enforcement path identified. |

### L3 containment

| Surface | Tier | Notes |
|---------|------|-------|
| **Linux L3 (bubblewrap)** | E — PLATFORM | Implemented, tested, but Windows-only environment. Cannot verify on this system. |
| **Windows L3** | F — UNSUPPORTED | No equivalent of bubblewrap. WSL2 not installed. |

---

## What ships in v1 (Tier A)

1. **Onus binary** — `cargo build`, `cargo test`, `onus evaluate` with block/allow/correction
2. **Python SDK** — `OnusClient`, `OnusBlockError`, `OnusEvaluationError`, `TaskContract`, `OnusReceiptData`
3. **OpenAI Agents SDK wrapper** — `OnusToolWrapper` + tests (16)
4. **LangChain / LangGraph wrapper** — `OnusToolWrapper` + tests (21)
5. **CrewAI adapter** — `crewai_onus_tool` decorator + tests (7)

---

## What ships as protocol-only (Tier C)

1. **MCP proxy** — The proxy binary exists and passes unit tests but has never been tested with a real agent. Deploying it requires: installing an MCP-supporting agent, configuring its MCP server to `onus mcp-proxy`, and verifying interception in a live loop.
2. **VS Code extension** — Deploys, activates, shows status. All hooks are L1. No agent tool-call interception proven.

---

## What is explicitly NOT in v1

| Surface | Reason |
|---------|--------|
| JetBrains (any) | No installed IDE, no plugin code, no analysis |
| Cursor IDE L2 | Not installed, needs investigation |
| Windsurf L2 | Not installed, needs investigation |
| Windows L3 | No OS support for bubblewrap |
| GitHub Copilot SDK | No adapter, no credentials |
| Aider | Not installed, no analysis |
| Gemini CLI | Not installed, no analysis |
| Cline native tool interception | Architectural barrier — VS Code extension API cannot intercept another extension's tool calls |
| Continue native tool interception | Same architectural barrier as Cline |
| VS Code Copilot tool interception | Same architectural barrier |

---

## What requires user action to ship

See `PHASE15E_USER_ACTIONS.md` for full grouped checklist. Summary:

- **Group A** (auth/credentials): ANTHROPIC_API_KEY, GITHUB_TOKEN
- **Group B** (install): Cursor, Windsurf, JetBrains, Aider, Gemini CLI
- **Group C** (platform): Linux for L3 testing
- **Group D** (subscription): Devin, Antigravity agent workspaces, Cursor Pro

---

## v1 honest statement

> Onus v1 provides **deterministic enforcement** for:
> 1. Shell commands and file operations evaluated through the Onus binary
> 2. Python agent framework tool calls routed through the SDK wrappers (OpenAI Agents SDK, LangChain/LangGraph, CrewAI) — advisory/bypassable
>
> Onus provides **protocol-level readiness** for MCP proxy interception but has not been tested with a live agent.
>
> Onus provides **L1 observation only** for VS Code, Antigravity, and Windsurf agent actions through native extension hooks.
>
> Onus does **NOT enforce** against JetBrains agents, Cursor native agents, Cline/Continue internal tools, Aider, Gemini CLI, Claude Code CLI, or GitHub Copilot SDK — these require installation, authentication, platform changes, or new adapter code not present in v1.
>
> Onus does **NOT provide L3 workspace containment** on Windows.

---

*Classification produced: 2026-06-17*
*Sources: Test results from Phase 15E engineering closures, environment inventory, gate matrix, agent analysis per surface.*
