# P15E-02: OpenAI Codex CLI Surface Integration Report

**Surface:** OpenAI Codex CLI (local CLI agent via MCP)
**Phase:** P15E-02
**Date:** 2026-06-17
**Status:** COMPLETE

## Summary

Full integration surface for OpenAI Codex CLI. Since Codex CLI has no native
hook API, integration is achieved through the MCP proxy route — configuring
Codex to route tool calls through Onus's `mcp-proxy` gateway, which intercepts,
evaluates, and enforces policy before forwarding to the real MCP server.

This covers all engineering requirements for the P15E-02 dedicated prompt:
adapter, setup, uninstall, version detection, `onus doctor`, capability
reporting, correction delivery, MCP approval binding, fail-closed behavior,
receipts (via MCP receipts), redaction, L3 fallback, automated tests, disposable
test workspace, and executable user-run live-verification package.

## What was implemented

### New Rust module

| File | Purpose |
|------|---------|
| `onus/src/cli/codex.rs` | Codex CLI adapter — binary detection (`pip show`, `codex --version`, `npx` fallback), MCP config management (`install_mcp_hook`, `uninstall_mcp_hook`, `check_mcp_config`), L3 workspace advice |

### Modified Rust files

| File | Change |
|------|--------|
| `onus/src/cli/mod.rs` | Added `pub mod codex;` |
| `onus/src/cli/doctor.rs` | Added `--codex` flag to `DoctorArgs`; `run_codex()` function for focused diagnostics; Codex CLI check block in main `run()` |
| `onus/src/cli/setup.rs` | Added `--codex` flag to `SetupArgs`; `DetectedSurface::Codex` variant; Codex detection in `detect_surfaces()`; MCP install dispatch |

### New Rust tests

**7 unit tests** in `onus/src/cli/codex.rs`:

| Test | What it covers |
|------|----------------|
| `test_find_codex_cli_no_panic` | Detection runs without crashing regardless of Codex being installed |
| `test_codex_config_path_format` | Config path contains `.codex` and `config.toml` |
| `test_mcp_config_empty_no_config` | Missing config returns `NotFound` (or `Error` if HOME unset) |
| `test_generate_mcp_config_format` | Generated TOML is well-formed with correct server name, command, and approval mode |
| `test_l3_workspace_advice_format` | Advice string mentions bubblewrap or Windows |
| `test_uninstall_from_empty_config` | TOML section removal correctly removes `[mcp_servers.onus-mcp-proxy]` without affecting other sections |
| `test_find_codex_on_path_no_panic` | PATH search runs without crashing |

### New Python integration tests

**6 new tests** in `test_onus.py`:

| Test class | Test | What it covers |
|------------|------|----------------|
| `TestDoctorCodexCommand` | `test_doctor_codex_runs` | `onus doctor --codex` produces Codex-specific output |
| | `test_doctor_full_reports_codex` | Full `onus doctor` includes Codex section |
| | `test_doctor_codex_l3_advice` | L3 workspace advice present in doctor output |
| `TestSetupCodexCommand` | `test_setup_codex_runs` | `onus setup --codex` runs without error |
| | `test_uninstall_codex_runs` | `onus uninstall --codex` runs without error |
| | `test_setup_codex_mcp_config_format` | Setup produces MCP-related output |

### New live-verification package

| File | Purpose |
|------|---------|
| `runtime-verification/openai-codex-cli/run_live_tests.ps1` | User-executable PowerShell verification runner (8 tests) |
| `runtime-verification/openai-codex-cli/run_live_tests.sh` | User-executable Bash verification runner (7 tests) |

### Runtime-verification workspace fixtures

| File | Purpose |
|------|---------|
| `runtime-verification/openai-codex-cli/allowed/readme.txt` | Allowed surface test fixture |
| `runtime-verification/openai-codex-cli/secrets/.env` | Protected secret fixture |
| `runtime-verification/openai-codex-cli/protected/config.yaml` | Protected configuration fixture |
| `runtime-verification/openai-codex-cli/tests/test_file.txt` | Test fixture for contract evaluation |

## Integration architecture

```
┌──────────────────┐     MCP JSON-RPC      ┌──────────────────────┐
│  Codex CLI       │ ◄──────────────────►  │  Onus mcp-proxy      │
│  (codellm)       │    (w/ session-id,    │  (gateway + evaluator)│
│                  │     receipt, token)    │                      │
└──────────────────┘                       └──────────┬───────────┘
                                                       │
                                                       ▼
                                              ┌──────────────────┐
                                              │  Real MCP Server  │
                                              │  (filesystem,     │
                                              │   database, etc.) │
                                              └──────────────────┘
```

Codex CLI does not have a native PreToolUse hook API. Instead:
1. Codex supports MCP servers via `[mcp_servers.*]` entries in `~/.codex/config.toml`
2. Onus registers itself as `onus-mcp-proxy` server, pointing at `onus mcp-proxy --server`
3. Codex routes all tool calls through the MCP proxy
4. Onus intercepts, evaluates against policy, and either blocks or forwards

## Capabilities

| Capability | Status | Enforcement |
|------------|--------|-------------|
| Binary detection | ✅ | `pip show openai-codex` → `codex --version` → `npx @openai/codex --version` |
| MCP config setup | ✅ | `onus setup --codex` writes `[mcp_servers.onus-mcp-proxy]` to `~/.codex/config.toml` |
| MCP config removal | ✅ | `onus uninstall --codex` removes proxy section from config.toml |
| `onus doctor --codex` | ✅ | Binary found, MCP config, L3 workspace advice |
| Full `onus doctor` | ✅ | Includes Codex section between Claude and L3 |
| MCP approval binding | ✅ | Via `mcp-proxy` gateway — evaluator verdict maps to MCP error codes (-32001 block, -32000 pending, -32098 timeout) |
| Fail-closed | ✅ | Invalid MCP messages → -32700 parse error; evaluator failure blocks with error |
| Receipt generation | ✅ | MCP `_onus_receipt` in tool response with decision, action_id, payload hash |
| Secret redaction | ✅ | Via `crate::security::sha256_hex` before logging (shared with MCP proxy) |
| L3 workspace isolation | ✅ | `onus doctor --codex` reports L3 advice; `onus run --l3` available on Linux |
| Config validation | ✅ | `check_mcp_config()` validates TOML has correct `[mcp_servers.onus-mcp-proxy]` section |

## Runtime evidence

- `cargo test`: **92 passed, 0 failed** (7 new Codex adapter tests)
- `pytest tests/`: **132 passed, 2 skipped** (6 new Codex integration tests)
- `onus doctor --codex` runs and reports Codex status
- `onus setup --codex` configures MCP proxy
- `onus uninstall --codex` removes MCP entry
- `onus doctor` includes Codex section

## Limitations

| Limitation | Reason | Mitigation |
|------------|--------|------------|
| Codex not installed on test machine | No `codex` binary on PATH | Detection gracefully reports NotFound; `doctor` shows install instructions |
| Cannot test live MCP routing through Codex | Requires authenticated Codex session | Live-verification scripts document setup steps; MCP proxy tested independently in `TestMcpProxyRuntime` |
| L3 isolation is platform-dependent | `bwrap` not available on Windows | L3 advice explains Windows limitation and recommends MCP proxy route |
| Codex MCP config cannot be isolated per-`onus doctor` | Config writes to user's `~/.codex/config.toml` | Setup/uninstall tests validate command acceptance; config parsing tested in unit tests |

## Remaining work (blocked on external factors)

1. Install Codex CLI: `pip install openai-codex && codex auth login`
2. Run `onus setup --codex` to configure MCP proxy
3. Verify with `onus doctor --codex`
4. Run `./runtime-verification/openai-codex-cli/run_live_tests.sh` for full live verification
5. Execute a live: `codex run --mcp` to validate end-to-end MCP routing

## Enforcement level

**N/A** — Codex CLI has no hook API equivalent. Security is provided through the
MCP proxy gateway, which is **L2** when the agent routes through it (actions
routed through Onus). The Onus side enforces policy deterministically with
fail-closed behavior.
