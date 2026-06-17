# Phase 15E — User Actions Checklist

**Date**: 2026-06-17
**Branch**: `codex/phase15-integrations`

---

## Group A — Free authentication (user must run login commands)

| Surface | Exact user action | Command/UI path | Free or paid | Why Claude cannot do it | Verification after action |
|---------|------------------|-----------------|-------------|------------------------|---------------------------|
| Claude Code CLI | Run `npx claude code --login` in terminal | `npx claude code --login` (opens browser) | Free with API key | Login flow requires browser-based OAuth | `which npx && npx claude --version` |
| GitHub Copilot SDK | Run `gh auth login` then set `GITHUB_TOKEN` | `gh auth login --web` | Free with GitHub account | Requires browser-based OAuth | `gh auth status` |
| Gemini CLI | Run `gemini auth login` | `gemini auth login` | Free with Google account | Login flow requires browser | `gemini auth status` |

## Group B — Free installations (user must install software)

| Surface | Exact user action | Command/UI path | Verification after action |
|---------|------------------|-----------------|--------------------------|
| OpenAI Codex CLI | Install via `npm install -g @openai/codex` | `npm install -g @openai/codex` | `codex --version` |
| Aider | Install via `pip install aider-chat` | `pip install aider-chat` | `aider --version` |
| Continue CLI | Install via `npm install -g @continuedev/continue` | `npm install -g @continuedev/continue` | `continue --version` |
| Gemini CLI | Install via `npm install -g @google-gemini/gemini-cli` | `npm install -g @google-gemini/gemini-cli` | `gemini --version` |

## Group C — Paid subscription or account requirement

| Surface | Exact user action | Free or paid | Why blocked |
|---------|------------------|-------------|-------------|
| Cursor CLI | Sign up at cursor.com, install Cursor IDE | Paid after trial | Requires Cursor subscription |
| Windsurf Editor | Sign up at codeium.com, install Windsurf | Paid after trial | Requires Windsurf subscription |
| JetBrains Junie | Install JetBrains IDE + Junie plugin | Paid | Requires JetBrains license + Junie subscription |
| Cursor Background Agents | Same as Cursor CLI | Paid | Requires Cursor subscription |

## Group D — Interactive IDE actions (user must open IDE and perform actions)

| Surface | Exact user action | Why Claude cannot do it |
|---------|------------------|------------------------|
| Antigravity agent test | Open Antigravity, enable Onus extension, run agent action | Extension host tests can be automated; actual agent session requires interactive IDE |
| Devin Desktop agent test | Open Devin Desktop, enable Onus extension, run agent action | Same — agent session is interactive |
| Cline | Open VS Code, install Cline from marketplace, enable Onus, run agent action | VS Code marketplace install requires interactive UI |

---

## Automatic completion note

The following surfaces have been completed automatically by Claude during Phase 15E:

- Surface 18 (OpenAI Agents SDK) — framework runtime verified, 36 tests
- Surface 19 (LangChain/LangGraph) — framework runtime verified, 23 tests
- Surface 20 (CrewAI) — adapter created, 7 tests
- Surface 4 (VS Code Agents) — extension host tests (5) passing, 32 verify checks
- Surface 6 (Antigravity) — extension deployed to user dir
- Surface 8 (Devin Desktop) — extension deployed to user dir
- Environment report created
- Gate matrix created

Remaining surfaces require the user actions above.

---

## Verification cross-reference

For each surface in Groups A-D above, the blocking gate is documented in
[Phase 15E Release Gate Matrix](PHASE15E_GATE_MATRIX.md) with the exact
gate number and unblock instructions.

| Surface | Gate # | Current status |
|---------|--------|---------------|
| Claude Code CLI | Gate 27 | BLOCKED BY USER AUTH |
| GitHub Copilot SDK | Gate 35 | BLOCKED BY USER AUTH |
| Gemini CLI | Gate 29 | BLOCKED BY USER INSTALL |
| OpenAI Codex CLI | Gate 28 | BLOCKED BY USER INSTALL |
| Aider | Gate 33 | BLOCKED BY USER INSTALL |
| Continue CLI | Gate 31 | BLOCKED BY USER INSTALL |
| Cursor CLI | Gate 30 | BLOCKED BY USER INSTALL |
| JetBrains Junie | Gate 32 | BLOCKED BY USER INSTALL |
| Cursor Background Agents | Gate 30 (same as Cursor CLI) | BLOCKED BY USER INSTALL |
| Windsurf Editor | Gate — | BLOCKED BY USER INSTALL |
| Antigravity | Gate 24 | BLOCKED BY USER ACTION |
| Devin Desktop | Gate 25 | BLOCKED BY USER ACTION |
| Cline | Gate 34 | BLOCKED BY USER INSTALL |

---

## Suggested execution order

1. **Group A first** (authentication only, no installs needed)
   - `npx claude code --login`
   - `gh auth login --web`
   - `gemini auth login` (requires Group B install first)
2. **Group B installs** (batch install, then auth where needed)
   - `npm install -g @openai/codex`
   - `npm install -g @continuedev/continue`
   - `npm install -g @google-gemini/gemini-cli`
   - `pip install aider-chat`
3. **Group D interactive tests** (requires VS Code open)
   - Cline marketplace install + agent test
   - Antigravity agent session
   - Devin Desktop agent session
4. **Group C paid signups** (requires payment decision)
   - Cursor, Windsurf, JetBrains Junie, Cursor Background Agents

---

*Generated 2026-06-17 for Phase 15E. 13 surfaces remain blocked by user action.*
