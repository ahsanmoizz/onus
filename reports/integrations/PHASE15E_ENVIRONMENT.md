# Phase 15E — Environment Inventory

**Date**: 2026-06-17
**Branch**: `codex/phase15-integrations`

---

## Runtime environment

| Component | Status | Version | Path |
|-----------|--------|---------|------|
| OS | Windows 11 Pro | 10.0.22631 | MINGW64_NT-10.0-22631 |
| Python | Installed | 3.12.5 | C:\Users\A\AppData\Local\Programs\Python\Python312\python.exe |
| Node | Installed | 24.15.0 | C:\Program Files\nodejs\node.exe |
| npm | Installed | 11.12.1 | C:\Program Files\nodejs\npm |
| Rust / cargo | Installed | 1.96.0 | cargo 1.96.0 |
| rustc | Installed | 1.96.0 | rustc 1.96.0 |
| WSL2 | Available | Default Version 2 | C:\WINDOWS\system32\wsl.exe |
| WSL distros | **None installed** | — | — |
| Docker | **Not installed** | — | — |
| gh (GitHub CLI) | **Not installed** | — | — |
| npx | Installed | — | C:\Program Files\nodejs\npx |
| Onus binary (debug) | Built | debug | D:\Onus\onus\target\debug\onus.exe |
| Onus binary (release) | **Not built** | — | — |

## Authentication state

| Provider | Key status | Length | Source |
|----------|-----------|--------|--------|
| OPENAI_API_KEY | **SET** | 35 chars | Environment |
| ANTHROPIC_API_KEY | **NOT SET** | — | — |
| GITHUB_TOKEN | **NOT SET** | — | — |

## Python SDK packages

| Package | Status | Version |
|---------|--------|---------|
| openai | Installed | 1.89.0 |
| langchain-core | Installed | 0.3.49 |
| langgraph | Installed | 0.4.29 |
| pydantic | Installed | 2.11.1 |
| pytest | Installed | 9.1.0 |
| crewai | **NOT INSTALLED** | — |

## Installed IDEs and editors

| IDE | Status | Version | Path |
|----|--------|---------|------|
| VS Code | **Installed** | 1.124.2 | Via `@vscode/test-cli` |
| VS Code (user) | **Installed** | — | C:\Users\A\.vscode\extensions\ |
| Google Antigravity | **Installed** | 1.107.0 | D:\Antigravity\bin\antigravity |
| Devin Desktop/Windsurf | **Installed** | 1.110.1 | D:\Windsurf\bin\devin-desktop |
| Cursor IDE | **Not installed** | — | — |
| JetBrains (any) | **Not installed** | — | — |
| Windsurf Editor | **Not installed separately** | — | Devin Desktop appears to be rebranded Windsurf |

## CLI agents

| CLI | Status | Notes |
|-----|--------|-------|
| Claude Code CLI | npx available, **not authenticated** | No ANTHROPIC_API_KEY |
| OpenAI Codex CLI | **Not found on PATH** | — |
| Gemini CLI | **Not installed** | — |
| Cursor CLI | **Not installed** | — |
| Continue CLI | **Not installed** | — |
| Junie CLI | **Not installed** | — |
| Aider | **Not installed** | — |

## VS Code extensions

| Extension | Installed location | Status |
|-----------|-------------------|--------|
| onus.onus-firewall (VS Code) | C:\Users\A\.vscode\extensions\onus.onus-firewall-0.1.0 | Deployed, has src/extension.js |
| onus.onus-firewall (Antigravity) | C:\Users\A\.antigravity\extensions\ | Added to extensions.json |
| onus.onus-firewall (Devin Desktop) | C:\Users\A\.devin-desktop\extensions\ | Registered in extensions.json |
| onus.agents-vscode | C:\Users\A\.vscode\extensions\onus.agents-vscode-0.1.0 | Has package.json + src/ |

## Onus project structure

| Module | Exists | Lines (approx) | Notes |
|--------|--------|----------------|-------|
| approval_broker.rs | Yes | — | Approval decision broker |
| daemon.rs | Yes | — | Background process lifecycle |
| mcp/mod.rs | Yes | — | MCP module root |
| mcp/proxy.rs | Yes | — | MCP gateway/proxy |
| ipc/mod.rs | Yes | — | IPC module root |
| ipc/protocol.rs | Yes | — | IPC protocol definitions |
| ipc/server.rs | Yes | — | IPC server |
| ipc/client.rs | Yes | — | IPC client |
| scope/mod.rs | Yes | — | Scope module |
| scope/tracker.rs | Yes | — | Scope tracker (L3 containment) |
| workspace.rs | Yes | — | L3 workspace management (Linux only via bubblewrap) |
| policy | Yes | — | Policy engine |
| security.rs | Yes | — | Security / redaction |
| semantic.rs | Yes | — | Semantic analysis |
| task_contract.rs | Yes | — | Task contract verification |
| memory.rs | Yes | — | Memory system |
| quality.rs | Yes | — | Quality checks |
| authority.rs | Yes | — | Authority broker |
| prompt_intake.rs | Yes | — | Prompt intake guardian |

## L3 containment

| Capability | Status |
|------------|--------|
| L3 workspace.rs | **Implemented** — Linux-only via bubblewrap |
| L3 scope/tracker.rs | **Implemented** — filesystem scope tracking, drift detection |
| Docker | Not available |
| WSL distros | None installed |
| Linux containers | Not available on this host |
| L3 on Windows | **Not implemented** — workspace.rs is Linux-only |

## Runtime verification workspaces

| Surface | Workspace exists |
|---------|-----------------|
| claude-code-cli | Yes (from Phase 15C) |
| All others | **Not created** |

---

*Created 2026-06-17. Environment re-detected fresh for Phase 15E.*
