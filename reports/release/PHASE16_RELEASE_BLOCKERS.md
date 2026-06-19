# Phase 16 — Release Blockers

**Date:** 2026-06-18
**Auditor:** OpenClaude (deepseek-chat)
**Branch:** `codex/phase15-integrations`
**Commit:** `aa38749`

---

## Blocker Summary

| Severity | Count |
|---|---|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 1 |
| LOW | 2 |
| USER-ACTION | 15 |

**No critical or high blockers. No engineering defects remain.**

---

## Medium: 1

### B-01: Windows L3 containment unimplemented

| Field | Value |
|---|---|
| Severity | MEDIUM |
| Surface | All CLI agents on Windows |
| Description | L3 workspace containment requires bubblewrap, which is Linux-only. Windows hosts cannot run `run_isolated()`. The `test_run_isolate_fails_closed_without_linux_boundary` test proves fail-closed behavior, but there is no containment option for Windows. |
| Impact | Windows users cannot execute agent actions inside a contained workspace. Onus can still deny/block via L2, but cannot prevent bypass paths (direct shell, direct filesystem). |
| Workaround | Deploy on Linux with bubblewrap installed; or use WSL2 with a Linux distro + bubblewrap |
| Fix target | Phase 16+: Docker-based L3 for Windows, or Windows Sandbox API integration |

---

## Low: 2

### B-02: 8 non-security Clippy warnings

| Field | Value |
|---|---|
| Severity | LOW |
| Warnings | cast_lossless, needless_pass_by_value, option_map_unit_fn, etc. |
| Impact | None on runtime behavior |
| Fix | Code cleanup (no behavioral change) |

### B-03: 2 pytest UnknownMarkWarning for `live_llm` mark

| Field | Value |
|---|---|
| Severity | LOW |
| Description | `pytest.mark.live_llm` used in test files but not registered in `pyproject.toml` |
| Impact | Warning only — does not affect test execution |
| Fix | Add `markers = {"live_llm": "Live LLM integration tests"}` to pytest config |

---

## User-Action: 15

### Authentication Required

| ID | Blocker | Surface | Action | Gate |
|---|---|---|---|---|
| B-04 | No Claude Code auth | Claude Code CLI | `npx claude code --login` | Gate 27 |
| B-05 | No GitHub token | GitHub Copilot SDK | `gh auth login --web` | Gate 35 |
| B-06 | No Gemini auth | Gemini CLI | `gemini auth login` | Gate 29 |

### Installation Required

| ID | Blocker | Surface | Action | Gate |
|---|---|---|---|---|
| B-07 | Not on PATH | OpenAI Codex CLI | `npm install -g @openai/codex` | Gate 28 |
| B-08 | Not installed | Aider | `pip install aider-chat` | Gate 33 |
| B-09 | Not installed | Continue CLI | `npm install -g @continuedev/continue` | Gate 31 |
| B-10 | Not installed | Gemini CLI | `npm install -g @google-gemini/gemini-cli` | Gate 29 |
| B-11 | Not installed | Cursor CLI | Install Cursor IDE | Gate 30 |
| B-12 | Not installed | JetBrains Junie | Install JetBrains IDE + plugin | Gate 32 |
| B-13 | Not installed | Cline | VS Code marketplace install | Gate 34 |
| B-14 | Not installed | Cursor IDE Agent | Install Cursor IDE (paid) | Gate 30 |
| B-15 | Not installed | Cursor Background Agents | Install Cursor IDE (paid) | Gate 30 |
| B-16 | Not installed | Windsurf Editor | Install Windsurf (paid) | Gate — |
| B-17 | Not installed | Continue JetBrains | Install JetBrains IDE (paid) | Gate — |

### Interactive GUI Required

| ID | Blocker | Surface | Action | Gate |
|---|---|---|---|---|
| B-18 | No agent session | Antigravity | Run agent in Antigravity | Gate 24 |
| B-19 | No agent session | Devin Desktop | Run agent in Devin Desktop | Gate 25 |

---

## Per-Phase Blocker Attribution

**Blockers requiring Phase 16+:**
- B-01 (Windows L3 containment) — Docker or Windows Sandbox

**Blockers for Phase 15E closure:**
- None — all Phase 15E engineering is complete. Remaining are user actions.

**Blocker count trend:**

| Phase | Critical | High | Medium | Low | User-action |
|---|---|---|---|---|---|
| Phase 15E audit | 0 | 0 | 1 (VS Code GPU) | 2 (clippy, pytest) | 11 |
| **Phase 16** | **0** | **0** | **1** (L3 on Windows) | **2** (clippy, pytest) | **15** |

Note: The user-action count increased because Phase 16 audited all 20 surfaces in detail, identifying 4 more surfaces that need user action (Cursor IDE, Cursor Background, Windsurf, Continue JetBrains).

---

## Verdict

**0 release-blocking engineering defects.** The sole MEDIUM blocker (Windows L3) is a platform limitation requiring Phase 16+ work. All 15 user-action blockers are documented with exact unblock steps.

The codebase is releasable for Linux targets with bubblewrap. Windows release is CONDITIONAL — L2 enforcement works, L3 containment is not available.
