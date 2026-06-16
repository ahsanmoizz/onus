# Phase 15B — Runtime Continuation

Date: 2026-06-17

## Completed in Phase 15B

| Surface | Classification | Tests | Commit |
|---|---|---|---|
| OpenAI Agents SDK | BLOCKED → IMPLEMENTED AND RUNTIME VERIFIED | 10/10 | `81858e7` |
| LangChain Agents / LangGraph | BLOCKED → IMPLEMENTED AND RUNTIME VERIFIED | 15/15 | `6c065ed` |
| Environment inventory | Done | N/A | `81858e7` |

## Remaining Priority Surfaces

### 1. Claude Code CLI — VERIFIED WITH LIMITATIONS (Phase 15)

**Blocker**: `npx @anthropic-ai/claude-code` runs but not authenticated.
`ANTHROPIC_API_KEY` env var present but returns 401.

**Next exact command**: `npx @anthropic-ai/claude-code login`
or authenticate via browser-based auth flow.

**Test to run**: Uncomment `ONUS_CLAUDE_CODE_LIVE=1` tests in
`test_onus.py::TestClaudeCodeAdapterRuntime`.

**Notes**: Hook protocol already verified in Phase 15 (8 claude_hook Rust tests +
8 Python hook protocol tests). Only the live agent loop with retry and
correction delivery is gated behind authentication.

### 2. Visual Studio Code Agents — VERIFIED WITH LIMITATIONS (Phase 15)

**Blocker**: Extension installed at
`C:\Users\A\.vscode\extensions\onus.agents-vscode-0.1.0\` and confirmed via
`code --list-extensions`, but runtime verification requires VS Code with an
interactive session to test terminal interception, task hooks, and status bar.

**Next exact command**: `code --install-extension onus.agents-vscode-0.1.0`
(already done). Then run VS Code with `--enable-proposed-api onus.onus-firewall`
and manually verify terminal interception, status bar, and blocked actions.

**Test to write**: VS Code extension test using `@vscode/test-electron` with
`vscode-test` runner.

**Notes**: The extension source at `onus/bindings/vscode/` contains real
interception code (terminal shell hooks, task evaluation, status bar).

### 3. Windsurf Editor / Cascade — BLOCKED

**Blocker**: Product not installed. `D:\Windsurf\bin\devin-desktop` exists but
is a VS Code fork branded "Devin Desktop", not Windsurf.

### 4. Cline — BLOCKED

**Blocker**: VS Code extension `saoudrizwan.claude-dev` not installed.

### 5. OpenAI Codex CLI — BLOCKED

**Blocker**: `codex`/`codex-cli` binary not in PATH.

### 6. Gemini CLI — BLOCKED

**Blocker**: `gemini` binary not in PATH.

## How to Continue

1. Authenticate Claude Code CLI via browser login or valid ANTHROPIC_API_KEY
2. Uncomment and run the live Claude Code tests
3. Set up VS Code extension test environment with @vscode/test-electron
4. Test terminal interception, blocked actions, status bar in VS Code

## Repository State

| Item | Status |
|---|---|
| Branch | `codex/phase15-integrations` |
| Uncommitted changes | None — working tree clean |
| Python tests | 82 passed, 2 skipped |
| Rust tests | 75 passed, 0 failed |
| Spec lock | PASSED |
| Phase 15B progress | 2 surfaces upgraded to RUNTIME VERIFIED |
