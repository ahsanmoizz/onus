# CONTINUITY LIVE TEST

Generated: 2026-06-18

## Purpose

Validate Claude Code → Codex CLI handoff continuity via Onus.

## Pre-requisites

Both agents must be installed, authenticated, and registered:

| Agent | Install | Auth | Register | Verify |
|-------|---------|------|----------|--------|
| Claude Code CLI | `npm i -g @anthropic/claude-code` | `claude login` | `onus setup --claude` | `onus doctor --claude` |
| OpenAI Codex CLI | `npm i -g @openai/codex` | `codex login` | `onus setup --codex` | `onus doctor --codex` |

## Script

```
.\scripts\test-continuity-claude-codex.ps1
```

The script:

1. Creates a disposable Git project under `runtime/continuity-test/`
2. Starts Claude Code with a governed task
3. Claude creates `src/main.py`, stops before completing
4. Captures a checkpoint via `onus checkpoint create`
5. Starts Codex CLI to continue
6. Codex creates `src/test_main.py` and runs tests
7. Prompts to verify receipts and session continuity

## Expected chain-of-custody

```
Claude Code action 1  (create task contract)
Claude Code action 2  (create src/main.py)
Claude Code action 3  (stop, incomplete)
        ↓ checkpoint + handoff
Codex CLI action 1    (continue contract)
Codex CLI action 2    (create src/test_main.py)
Codex CLI action 3    (run tests)
Codex CLI action 4    (report completion)
```

All actions share one session_id → one hash chain → `onus verify` passes.

## Current status

The script is **semi-automated** because both Claude Code and Codex CLI are
interactive agents requiring user prompts.  The script launches each agent,
waits for the user to complete the interaction, and then proceeds.

When both agents are unavailable, the script reports the installation steps
and exits without failure — it does NOT fake agent execution.
