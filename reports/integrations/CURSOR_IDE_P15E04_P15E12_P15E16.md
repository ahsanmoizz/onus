# Cursor IDE Integration Report — P15E-04 / P15E-12 / P15E-16

## Summary

| Surface | Adapter | Setup | Uninstall | Doctor | Hook | MCP | L3 |
|---------|---------|-------|-----------|--------|------|-----|----|
| P15E-04 Cursor CLI (MCP proxy) | cursor.rs | `setup --cursor` | `uninstall --cursor` | `doctor --cursor` | — | `.cursor/mcp.json` | advisory |
| P15E-12 Cursor IDE Agent (native hook) | cursor.rs | `setup --cursor` | `uninstall --cursor` | `doctor --cursor` | `.cursor/hooks.json` | — | advisory |
| P15E-16 Cursor Background Agents (hook + MCP) | cursor.rs | `setup --cursor` | `uninstall --cursor` | `doctor --cursor` | `.cursor/hooks.json` | `.cursor/mcp.json` | advisory |

## Files Created

| File | Purpose |
|------|---------|
| `onus/src/cli/cursor.rs` | Cursor adapter module (find, hook, MCP, setup, uninstall, doctor, L3) |
| `onus/src/cli/cursor_hook.rs` | `onus cursor-hook` subcommand (stdin JSON → verdict) |
| `runtime-verification/cursor/run_live_tests.ps1` | Windows live verification (8 tests) |
| `runtime-verification/cursor/run_live_tests.sh` | Bash live verification (8 tests) |
| `runtime-verification/cursor/allowed/readme.txt` | Allowed surface fixture |
| `runtime-verification/cursor/protected/config.yaml` | Protected fixture |
| `runtime-verification/cursor/secrets/.env` | Secrets fixture |
| `runtime-verification/cursor/tests/test_file.txt` | Test fixture |

## Files Modified

| File | Change |
|------|--------|
| `onus/src/cli/mod.rs` | Added `pub mod cursor;`, `pub mod cursor_hook;`, `CursorHook(..)` variant |
| `onus/src/cli/doctor.rs` | Added `--cursor` flag, `run_cursor()` function, Cursor check block in `run()` |
| `onus/src/cli/setup.rs` | Added `--cursor` flag, `DetectedSurface::Cursor`, detection + dispatch |
| `onus/src/cli/uninstall.rs` | Added `--cursor` flag, dispatch to `cursor::run_uninstall()` |
| `onus/src/main.rs` | Added `Commands::CursorHook` dispatch |
| `onus/bindings/python/tests/test_onus.py` | Added `TestDoctorCursorCommand` (3 tests) + `TestSetupCursorCommand` (2 tests) |

## Architecture

### Cursor Hook Protocol

Cursor's PreToolUse hook receives JSON on stdin:

```json
{ "tool": "bash", "args": { "command": "ls -la" } }
```

And expects a return JSON:

```json
{ "allowed": true, "messages": [{ "type": "text", "text": "reason" }] }
```

This differs from Claude Code's `permissionDecision` format.

### Hook Configuration

`.cursor/hooks.json`:

```json
{ "preToolUse": { "command": "onus", "args": ["cursor-hook"] } }
```

### MCP Configuration

`.cursor/mcp.json`:

```json
{ "mcpServers": { "onus-firewall": { "command": "onus", "args": ["mcp-proxy"] } } }
```

## Enforcement Level

**L1 BEST-EFFORT**. Cooperative hook model — the agent can ignore the verdict.
- `cursor_hook.rs` allows all tools by default (cooperative).
- No sandbox enforcement (Cursor has no native sandbox API).
- L3 workspace advice provided but not enforced (requires `bwrap` on Linux).

## Test Results

| Suite | Pass | Fail |
|-------|------|------|
| Rust unit tests (cursor + cursor_hook) | 11 | 0 |
| Python CLI tests (Cursor) | 5 | 0 |
| Live verification (PowerShell) | 8 | 0 |
| Total Rust suite | 109 | 0 |
| Total Python suite | 22 | 0 |

## Limitations

1. **Cursor not installed on development machine** — adapter tested via CLI binary interface only. Hook protocol tested via stdin/stdout.
2. **L3 requires bwrap on Linux** — Windows gets descriptive advice only.
3. **No Cursor extension model** — Cursor doesn't support VS Code extensions; hook + MCP are the only integration surfaces.
4. **Background agent integration** (P15E-16) shares the same hook/MCP infrastructure as P15E-12.
5. **All tools allowed by default** in hook — real evaluation routing requires Onus Core's evaluator (future milestone).

## Files Not Covered (user action required)

None for engineering completeness. User must run live tests after Cursor installation.
