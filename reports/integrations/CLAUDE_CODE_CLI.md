# Claude Code CLI Integration Report

Milestone surface: 1 of 20, Claude Code CLI.

Branch: `integration/claude-code-cli`.

Date: 2026-06-16.

## Claim

Onus provides a Claude Code CLI `PreToolUse` hook adapter through:

- `onus claude-hook`
- `onus/src/cli/claude_hook.rs`
- `onus/src/cli/evaluate.rs`
- `onus/install/install.ps1`
- `onus/install/install.sh`

This integration is L1 cooperative and must remain labeled:

```text
BEST-EFFORT
```

Onus only participates when Claude Code is configured to invoke the hook.
Disabled hooks, missing hook settings, direct tool execution, direct MCP server
access, unsupported Claude tool events, and authenticated/cloud behavior not
observed locally remain outside this adapter's proven boundary.

## Official Control Surface

Official Claude Code documentation exposes hooks, including `PreToolUse`, as the
native control surface for reviewing tool calls before execution:

- <https://code.claude.com/docs/en/hooks>
- <https://code.claude.com/docs/en/hooks-guide>

The Onus adapter uses that hook shape and returns `permissionDecision` values
that map Onus verdicts to Claude Code hook behavior:

| Onus result | Claude Code hook result |
| --- | --- |
| `allow`, `warn` | `allow` |
| `escalate` | `ask` |
| `block` or evaluator failure | `deny` |
| unsupported tool | `ask` |

## Version-Pinned Runtime Probe

Pinned command:

```text
npx -y @anthropic-ai/claude-code@2.1.177
```

Version probe:

```text
2.1.177 (Claude Code)
```

Authentication probe:

```json
{
  "loggedIn": false,
  "authMethod": "none",
  "apiProvider": "firstParty"
}
```

No `ONUS_CLAUDE_CODE_LIVE`, `ANTHROPIC_API_KEY`, or
`CLAUDE_CODE_OAUTH_TOKEN` environment value was present. Therefore a live,
authenticated Claude Code agent-loop run was not certified in this environment.

## Runtime Evidence

Focused Rust hook tests:

```text
cargo test claude_hook -- --nocapture
3 passed; 0 failed
```

Focused Python hook/runtime tests:

```text
python -m pytest -q -rs onus\bindings\python\tests\test_onus.py -k claude_hook
4 passed, 55 deselected
```

Spec lock:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

Manual hook-process probe using the real local binary
`D:\Onus\onus\target\debug\onus.exe`:

Denied destructive shell command:

```json
{
  "permissionDecision": "deny",
  "permissionDecisionReason": "Onus blocked this Claude Code action: SAFETY_001 - This shell command would destroy filesystem data. If this is intentional (cleaning build artifacts), add the path to your allowlist via `onus rules edit`."
}
```

Allowed harmless shell command:

```json
{
  "permissionDecision": "allow",
  "permissionDecisionReason": "Onus allowed this Claude Code tool call (allow)."
}
```

Unsupported tool:

```json
{
  "permissionDecision": "ask",
  "permissionDecisionReason": "Onus Claude Code hook does not yet support tool 'ImaginaryTool'. Ask before proceeding."
}
```

Malformed stdin through a Windows PowerShell text pipe produced a fail-closed
hook response:

```json
{
  "permissionDecision": "deny",
  "permissionDecisionReason": "Onus Claude Code hook rejected malformed input: expected value at line 1 column 1"
}
```

The byte-exact UTF-8 probe was rerun through Node.js `spawnSync`, and produced
the allow, deny, and ask decisions listed above.

## Tested Behaviors

| Required behavior | Result | Evidence |
| --- | --- | --- |
| allow | VERIFIED WITH LIMITATIONS | local `onus claude-hook` process allowed `echo phase15-ok` |
| deny | VERIFIED WITH LIMITATIONS | local `onus claude-hook` process denied `rm -rf /important` |
| ask | VERIFIED WITH LIMITATIONS | unsupported tool returned `ask`; approval path covered by focused tests |
| correction delivery | VERIFIED WITH LIMITATIONS | deny reason includes rule and correction |
| agent retry | VERIFIED WITH LIMITATIONS | covered by focused Python hook test |
| malformed output | VERIFIED WITH LIMITATIONS | covered by focused Python hook test |
| hook process failure | VERIFIED WITH LIMITATIONS | covered by focused Python hook test |
| timeout | VERIFIED WITH LIMITATIONS | covered by focused Python hook test |
| unavailable evaluator | VERIFIED WITH LIMITATIONS | covered by focused Python hook test |
| disabled hook | VERIFIED WITH LIMITATIONS | covered by focused Python hook test |
| nested/subagent metadata | VERIFIED WITH LIMITATIONS | covered by focused Python hook test |
| unsupported tool | VERIFIED WITH LIMITATIONS | local process returned `ask` |
| bypass behavior | VERIFIED WITH LIMITATIONS | report documents direct/misconfigured hook bypass |
| strict and guardian modes | VERIFIED WITH LIMITATIONS | covered by focused Python hook tests |
| authenticated Claude Code agent loop | BLOCKED | `loggedIn: false`; no credentials present |

## Security Notes

- This is not L2, L3, or L4 enforcement.
- The adapter cannot protect Claude Code if Claude does not load or run the hook.
- The adapter cannot prove Claude Code subagent behavior without an authenticated
  live Claude Code session.
- Deterministic Onus denial is preserved in the hook process evidence.
- Malformed hook input fails closed to `deny`.

## Final Classification

```text
VERIFIED WITH LIMITATIONS
```

The local hook adapter is runtime-proven. The authenticated Claude Code agent
loop remains `BLOCKED` in this environment.
