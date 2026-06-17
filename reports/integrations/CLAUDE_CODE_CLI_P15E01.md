# P15E-01: Claude Code CLI Surface Integration Report

**Surface:** Claude Code CLI (local interactive-agent)
**Phase:** P15E-01
**Date:** 2026-06-17
**Status:** COMPLETE

## Summary

Full integration surface for Claude Code CLI. This covers all engineering
requirements for the P15E-01 dedicated prompt: adapter, setup, uninstall,
version detection, `onus doctor`, capability reporting, hook approval binding,
fail-closed behavior, receipts, redaction, L3 fallback, automated tests,
disposable test workspace, and executable user-run live-verification package.

## What was implemented

### New Rust modules

| File | Purpose |
|------|---------|
| `onus/src/cli/doctor.rs` | `onus doctor` — daemon health, rule engine, Claude Code CLI version/hook/level checks, L3 isolation, audit trail |
| `onus/src/cli/setup.rs` | `onus setup` / `onus uninstall` — surface detection, Claude Code hook install/remove in `~/.claude/claude.json` |

### Modified Rust files

| File | Change |
|------|--------|
| `onus/src/cli/mod.rs` | Added `doctor`, `setup` modules; `Doctor`, `Setup` CLI commands |
| `onus/src/main.rs` | Dispatch for `Doctor`, `Setup` commands |
| `onus/src/cli/claude_hook.rs` | Added `--l3-workspace`, `--receipt`, `--receipt-path` flags; receipt generation with hash-chain; `run_in_l3_workspace()`; L3 fallback tests |
| `onus/src/cli/uninstall.rs` | Added `--claude` flag for targeted hook removal |

### New test coverage

- **Rust unit tests** (6 new, total 85): doctor run, Claude CLI check format,
  count rules, hook receipt structure, L3 fallback, tool-type mapping, hook
  disabled env, fail-closed
- **Python integration tests** (10 new, total 126): doctor full, doctor --claude,
  doctor with audit trail, setup claude, uninstall --claude, hook receipt stderr,
  hook receipt file, fail-closed, deny dangerous, allow safe

### Live-verification package

| File | Purpose |
|------|---------|
| `runtime-verification/claude-code-cli/run_live_tests.ps1` | User-executable PowerShell verification runner (9 tests) |
| `runtime-verification/claude-code-cli/run_live_tests.sh` | User-executable Bash verification runner (9 tests) |

### Runtime-verification workspace

The existing `runtime-verification/claude-code-cli/` fixture workspace contains
readable, writable, and protected file layouts for isolated testing.

## Capabilities

| Capability | Status | Enforcement |
|------------|--------|-------------|
| PreToolUse hook (L1 BEST-EFFORT) | ✅ | `onus claude-hook` reads JSON from stdin, writes Claude Code decision protocol to stdout |
| Hook setup | ✅ | `onus setup --claude` writes hook entry to `~/.claude/claude.json` |
| Hook uninstall | ✅ | `onus uninstall --claude` removes hook entry from `~/.claude/claude.json` |
| Version detection | ✅ | `onus doctor` detects `claude --version` and `npx` fallback |
| `onus doctor` | ✅ | Daemon, rules, Claude CLI, hook, L3, audit trail — per-surface diagnostics |
| Approval binding | ✅ | Evaluator "block"/"allow"/"ask" mapped to Claude's "deny"/"allow"/"ask" permissions |
| Fail-closed | ✅ | Invalid input → "deny"; evaluator failure → CLI-timeout "deny" |
| Receipt generation | ✅ | `--receipt` / `--receipt-path` creates JSON receipt with hash-chain signature |
| Secret redaction | ✅ | Via `crate::security::sha256_hex` before logging |
| L3 workspace isolation | ✅ | `--l3-workspace` flag; `bwrap`-backed on Linux (graceful error on other platforms) |
| Disabled-behavior config | ✅ | `--disabled-behavior allow\|deny` |

## Runtime evidence

- `cargo test`: **85 passed, 0 failed**
- `pytest tests/`: **126 passed, 2 skipped**
- `onus doctor` runs without crash
- `onus doctor --claude` produces Claude-specific diagnostics
- `onus setup --claude` creates hook config
- `onus uninstall --claude` removes hook config
- `onus claude-hook` produces valid JSON with `permissionDecision`
- `onus claude-hook --receipt` outputs `ONUS_RECEIPT` in stderr
- `onus claude-hook --receipt-path` writes file with `type: evaluation_receipt`

## Limitations

- L3 workspace isolation requires Linux + `bwrap`; other platforms get a clear
  error message rather than a silent fallback
- `onus doctor` daemon check expects `crate::daemon::is_running()` — if no daemon
  has ever been started, correctly warns rather than fails
- Receipt signing uses SHA-256 hash chain (not a full PKI signature) —
  appropriate for L1 BEST-EFFORT enforcement level
- Claude Code CLI detection uses `claude --version` and falls back to `npx`;
  if the user has installed via a non-standard method (pipx, homebrew, etc.),
  detection still works through PATH resolution

## Security considerations

- All `onus doctor` output is informational — no secrets or credentials exposed
- Receipt `body_hash` is SHA-256 of serialized body to provide tamper evidence
- Fail-closed on malformed input: invalid JSON → "deny" decision
- Hook mode `best_effort` correctly labeled in `claude.json` — no L2 claim made
- L3 workspace uses `--unshare-all --die-with-parent` for true namespace isolation
