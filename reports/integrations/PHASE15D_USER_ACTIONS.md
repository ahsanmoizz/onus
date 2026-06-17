# Phase 15D — User Actions Checklist

**Date**: 2026-06-17

Surfaces that can be automated (no user action needed):
1. VS Code Agents — extension host test
2. Google Antigravity — extension load + agent test
3. OpenAI Agents SDK — add bypass/approval/fail-closed tests
4. LangChain/LangGraph — add bypass/approval/fail-closed tests

## Required User Actions

| Order | Surface | Required action | Why required | Cost | Account needed | Installation size | Safe to continue afterward |
| ----: | ------- | --------------- | ------------ | ---: | -------------: | ----------------: | -------------------------: |
| 1 | Claude Code CLI | Authenticate via `npx @anthropic-ai/claude-code --login` | ANTHROPIC_API_KEY absent; hook tests exist but need live authenticated agent loop | Free (existing account) | Anthropic account | ~200 MB (npx cached) | Yes |
| 2 | Cline for VS Code | Install Cline VS Code extension | Cline runtime not detected | Free | None | ~50 MB | Yes |
| 3 | Gemini CLI | Run `npm install -g @google-gemini/gemini-cli` then `gemini login` | Not installed; no auth | Free tier | Google account | ~100 MB | Yes |
| 4 | Cursor CLI + IDE | Download from cursor.com and install | Not installed locally | Free tier | Cursor account | ~300 MB | Yes |
| 5 | Continue CLI | Run `npm install -g @continuedev/continue` | Not installed | Free | None | ~50 MB | Yes |
| 6 | Continue VS Code extension | Install from VS Code marketplace | Not installed | Free | None | ~30 MB | Yes |
| 7 | JetBrains Junie CLI | Install JetBrains Toolbox + Junie plugin | JetBrains runtime absent | Free tier | JetBrains account | ~1 GB | Yes |
| 8 | JetBrains Junie IDE | Same as above — requires full JetBrains IDE | JetBrains runtime absent | Free tier | JetBrains account | ~1.5 GB | Yes |
| 9 | Aider | `pip install aider-chat` then configure API key | Not installed; needs model credentials | API usage costs | OpenAI/Anthropic account | ~50 MB | Yes |
| 10 | Windsurf Editor | Download from codeium.com/windsurf and install | Only Devin Desktop found (Windsurf rebrand, not the real product) | Free tier | Codeium account | ~300 MB | Yes |
| 11 | OpenAI Codex CLI | Unlock Windows app binary or run via installer | Binary present but access denied | Free tier | OpenAI account | ~200 MB | Yes |
| 12 | GitHub Copilot SDK | Install `gh` CLI and authenticate with GITHUB_TOKEN | No `gh` CLI, no token | Free tier | GitHub account | ~50 MB | Yes |
| 13 | Cursor Background Agents | Enable Cursor cloud subscription | Cloud service, not local | Paid | Cursor account | N/A | Yes |
| 14 | CrewAI | `pip install crewai` and configure model credentials | Neither package nor credentials present | API usage costs | OpenAI/Anthropic account | ~100 MB | Yes |
| 15 | Continue JetBrains | Install JetBrains IDE + Continue plugin | JetBrains runtime absent | Free tier | JetBrains account | ~1.5 GB | Yes |

## Grouped Execution Plan

**Group A — Automated (this session, no user action needed):**
1. VS Code Agents — extension host test
2. Google Antigravity — extension load + agent test
3. OpenAI Agents SDK — add bypass/approval/fail-closed tests
4. LangChain/LangGraph — add bypass/approval/fail-closed tests

**Group B — Requires user authentication (one `!` command each):**
5. `npx @anthropic-ai/claude-code --login`

**Group C — Requires user installation (batch when user is ready):**
6. Install Cline, Continue CLI, Continue VS Code, Aider, Gemini CLI

**Group D — Requires signup or payment:**
7. Cursor, Windsurf, JetBrains, Codex CLI, GitHub Copilot, Cursor Background, CrewAI

When user approves Group A, I will proceed automatically. When user reaches Group B, a single `npx @anthropic-ai/claude-code --login` interactive session is needed.
