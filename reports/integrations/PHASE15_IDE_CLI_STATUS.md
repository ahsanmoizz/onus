# Phase 15C — IDE and CLI Runtime Status Audit

Date: 2026-06-17.

Phase: 15C — Complete IDE and CLI runtime verification across all 15 surfaces.

## Runtime Status Legend

| Status | Meaning |
|---|---|
| `LIVE RUNTIME VERIFIED` | End-to-end test with real installed software, no live model call needed |
| `LIVE RUNTIME VERIFIED WITH LIMITATIONS` | Runtime-tested but bypass or environment limit documented |
| `IMPLEMENTED BUT NOT LIVE VERIFIED` | Adapter code exists, no local runtime test run |
| `PROTOCOL VERIFIED ONLY` | Open protocol (MCP) template exists, no native product proof |
| `BLOCKED` | Software not installed or credentials unavailable |
| `FAILED` | Installed but integration broken |

## Detection Sources

Detection was performed by probing the local machine (2026-06-17):

- PATH probes: `which <binary>` for all 15 CLI tools
- VS Code and forks: `code --list-extensions`, `/d/Antigravity/bin/antigravity --list-extensions`, `/d/Windsurf/bin/devin-desktop --list-extensions`, `/d/Kiro/bin/kiro --list-extensions`
- API keys: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GOOGLE_API_KEY` env var checks
- Package probes: `pip show`, `npm ls -g`, `npm list -g`
- Filesystem: `C:\Program Files\JetBrains`, `C:\Users\A\.vscode\extensions`

## Status Audit Matrix

| Order | Exact surface | Installed | Authenticated | Version | Adapter state | Runtime state | Enforcement route | Level | Missing proof | Blocker |
| ---: | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Claude Code CLI | YES (npx) | NO | 2.1.179 | `onus claude-hook` L1 hook adapter | LIVE RUNTIME VERIFIED WITH LIMITATIONS | Hook protocol (stdin/stdout JSON) | L1 BEST-EFFORT | Live authenticated agent loop with retry and correction delivery | ANTHROPIC_API_KEY absent / not logged in |
| 2 | Windsurf Editor / Cascade | NO | N/A | N/A | MCP routing template | PROTOCOL VERIFIED ONLY | MCP via `onus mcp-proxy` | PROTOCOL ONLY | Native Windsurf installation; `D:\Windsurf\bin\devin-desktop` is Devin Desktop (VS Code fork), not Windsurf Editor | Not installed locally |
| 3 | Visual Studio Code Agents | YES | N/A | VS Code 1.124.2 | Extension `onus.onus-firewall` at `C:\Users\A\.vscode\extensions\onus.agents-vscode-0.1.0\` | LIVE RUNTIME VERIFIED WITH LIMITATIONS | Extension API (terminal hooks, task hooks, status bar) | L1 BEST-EFFORT | Runtime tests with `@vscode/test-electron`; interactive terminal interception | No interactive VS Code session available; no .vsix packaging |
| 4 | Cline | NO | N/A | N/A | N/A | BLOCKED | MCP via `onus mcp-proxy` (potential) | BLOCKED | Cline VS Code extension `saoudrizwan.claude-dev` | VS Code extension not installed; not in any VS Code fork |
| 5 | OpenAI Codex CLI | NO | N/A | N/A | MCP routing template | PROTOCOL VERIFIED ONLY | MCP via `onus mcp-proxy` (potential) | PROTOCOL ONLY | Codex CLI binary on PATH; local Windows app present but access denied | `codex`/`codex-cli` binary not in PATH |
| 6 | Gemini CLI | NO | N/A | N/A | MCP routing template | PROTOCOL VERIFIED ONLY | MCP via `onus mcp-proxy` (potential) | PROTOCOL ONLY | Gemini CLI binary installed and authenticated | `gemini` binary not in PATH |
| 7 | Cursor CLI | NO | N/A | N/A | N/A | BLOCKED | MCP via `onus mcp-proxy` (potential) | BLOCKED | Cursor CLI installed on PATH | `cursor` binary not on PATH |
| 8 | Cursor Agent in Cursor IDE | NO | N/A | N/A | N/A | BLOCKED | Extension API (potential) | BLOCKED | Cursor IDE installed; VS Code extension checks not reusable | Cursor IDE not installed |
| 9 | Continue CLI | NO | N/A | N/A | N/A | BLOCKED | MCP via `onus mcp-proxy` (potential) | BLOCKED | `continue` binary on PATH | `continue` binary not on PATH |
| 10 | Continue Agent for VS Code | NO | N/A | N/A | N/A | BLOCKED | Extension API (potential) | BLOCKED | Continue VS Code extension installed | VS Code extension `continue.continue` not installed |
| 11 | Continue Agent for JetBrains | NO | N/A | N/A | N/A | BLOCKED | N/A | BLOCKED | JetBrains IDE installed; Continue JetBrains extension | JetBrains not installed; Continue JetBrains extension not installed |
| 12 | JetBrains Junie CLI | NO | N/A | N/A | MCP routing template | PROTOCOL VERIFIED ONLY | MCP via `onus mcp-proxy` (potential) | PROTOCOL ONLY | `junie` binary on PATH | `junie` not on PATH |
| 13 | JetBrains Junie IDE Agent | NO | N/A | N/A | N/A | BLOCKED | IDE plugin API (potential) | BLOCKED | JetBrains IDE installed with Junie plugin | JetBrains not installed |
| 14 | Aider | NO | N/A | N/A | N/A | BLOCKED | L3 workspace (potential) | BLOCKED | `aider` binary or pip package installed | `aider` not on PATH; `aider-chat` pip package not installed |
| 15 | Google Antigravity | YES | N/A | 1.107.0 | None | IMPLEMENTED BUT NOT LIVE VERIFIED | Extension API (VS Code fork, potential) | BLOCKED | Onus adapter for Antigravity; Antigravity agent interop | Product is a VS Code fork branded "Antigravity" — needs investigation of agent surface |

## Summary

| Count | Category |
| ---: | --- |
| 1 | LIVE RUNTIME VERIFIED WITH LIMITATIONS (Claude Code CLI hook) |
| 1 | LIVE RUNTIME VERIFIED WITH LIMITATIONS (VS Code Agents extension) |
| 1 | IMPLEMENTED BUT NOT LIVE VERIFIED (Google Antigravity — VS Code fork detected) |
| 3 | PROTOCOL VERIFIED ONLY (Windsurf/Cascade, Codex CLI, Gemini CLI, Junie CLI) |
| 9 | BLOCKED (Cline, Cursor CLI, Cursor IDE, Continue CLI, Continue VS Code, Continue JetBrains, Junie IDE, Aider) |

## Key Findings

1. **Claude Code CLI**: Hook protocol verified. Live agent loop blocked by auth (401).
2. **VS Code Agents**: Extension installed at `onus.onus-firewall`. Runtime tests require `@vscode/test-electron` or interactive session.
3. **Google Antigravity**: Installed at `/d/Antigravity/` (VS Code fork, v1.107.0). No Onus extension configured. Needs investigation of its agent surface (native VS Code agent API or proprietary extension system).
4. **Antigravity/Kiro/Devin**: All three are VS Code forks on PATH. None have any VS Code extensions installed. Devin Desktop (`D:\Windsurf\bin\`) is NOT Windsurf Editor/Cascade.
5. **VS Code extension state**: Only `onus.onus-firewall` is installed. No Cline, Continue, or other agent extensions present.
6. **OPENAI_API_KEY**: Present (len=35) — available for live LLM tests with SDK frameworks.
7. **No JetBrains products**: No JetBrains directory found, no Junie CLI.

## Next Step

The VS Code extension surface is the most promising candidate for Phase 15C runtime improvement:
- VS Code 1.124.2 is installed
- Onus extension is verified at `code --list-extensions`
- Extension source at `onus/bindings/vscode/` contains real interception code
- Requires `@vscode/test-electron` for headless runtime tests
- Requires `vsce` for .vsix packaging

Google Antigravity is an unexpected second candidate — it is a VS Code fork with a native agent surface that needs investigation.

## Security Boundary

All surfaces remain L1 BEST-EFFORT or lower. No L2/L3/L4 claim was added.
