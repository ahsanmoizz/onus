# INTEGRATION ACTIVATION MATRIX

Generated: 2026-06-18
Branch: final/activation-readiness

---

| Surface | Adapter exists | Installed | Version | Authenticated | Setup command | Login action | Enforcement | Continuity | Live-test command |
|---------|---------------|-----------|---------|---------------|--------------|-------------|-------------|------------|------------------|
| Claude Code CLI | Yes (`cli/claude_hook.rs`) | Detection via `doctor --claude` | Detected at runtime | GitHub/Anthropic SSO (external) | `onus setup --claude` | `claude login` (external) | Hook-based action interception | Via handoff + lease modules | `onus doctor --claude` |
| OpenAI Codex CLI | Yes (`cli/codex.rs`) | Detection via `doctor --codex` | Detected at runtime | OpenAI platform (external) | `onus setup --codex` | `codex login` (external) | MCP proxy | Via handoff + lease modules | `onus doctor --codex` |
| Cursor IDE | Yes (`cli/cursor.rs`) | Detection via `doctor --cursor` | Detected at runtime | Cursor account (external) | `onus setup --cursor` | Cursor IDE login (external) | Hook + MCP proxy | Not directly supported | `onus doctor --cursor` |
| Google Antigravity | Yes (`cli/antigravity.rs`) | Detection via `doctor --antigravity` | Detected at runtime | Google Cloud (external) | `onus setup --antigravity` | `gcloud auth login` (external) | Extension + MCP proxy | Not directly supported | `onus doctor --antigravity` |
| VS Code Agents | Registry present in `setup` | No installed adapter | N/A | N/A | `onus setup --vscode` | VS Code auth (external) | Extension-based (not yet implemented) | N/A | N/A |

## Notes

- **Adapter exists**: The Rust source code includes detection and setup logic for this integration.
- **Installed**: Onus cannot install the agent itself — the user must install the CLI/IDE separately.
- **Authenticated**: Authentication is handled by each agent's own mechanism. Onus does not store agent credentials.
- **Enforcement mechanism** varies by integration:
  - **Claude Code**: Git-based L1 hooks + action intake hook.
  - **Codex CLI**: MCP proxy interception.
  - **Cursor IDE**: Custom hooks + MCP proxy.
  - **Antigravity**: Extension + MCP proxy.
- **Continuity** (handoff between agents) is supported via `handoff` and `lease` Rust API modules, usable by Claude Code ↔ Codex CLI workflows.

## Required external authentication

Each agent requires its own authentication before Onus can govern it:

| Agent | Authentication command |
|-------|----------------------|
| Claude Code | `claude login` (interactive browser) |
| OpenAI Codex CLI | `codex login` (interactive browser) |
| Cursor IDE | Login via Cursor application |
| Google Antigravity | `gcloud auth login` (interactive browser) |
| VS Code | VS Code built-in authentication |
