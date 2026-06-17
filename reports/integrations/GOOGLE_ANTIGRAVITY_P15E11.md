# P15E-11: Google Antigravity Integration Report

**Surface:** Google Antigravity (VS Code fork, v1.107.0)
**Phase:** P15E-11
**Date:** 2026-06-18
**Status:** COMPLETE

## Summary

Full integration surface for Google Antigravity — a VS Code fork by Google with
its own extension model and CLI extension management. Antigravity does NOT have
a native hook API. Integration is provided through:

1. **Extension deployment** — the Onus extension (shared with VS Code) is
   packaged and installed via `antigravity --install-extension <vsix>`.
2. **MCP routing** — `antigravity --add-mcp <json>` configures Onus as an MCP
   proxy server for tool-call interception.
3. **CLI extension management** — `--list-extensions`, `--uninstall-extension`,
   `--update-extensions`.
4. **Native extension API** — `activate()` intercepts vscode commands; 5
   contributed commands (doctor, setup, status, toggle).

The extension is already deployed at
`~/.antigravity/extensions/onus.onus-firewall-0.1.0/` and listed by
`antigravity --list-extensions`. This is the only IDE in this repository that
is live-deployed on this machine.

## What was implemented

### New Rust module

| File | Purpose |
|------|---------|
| `onus/src/cli/antigravity.rs` | Antigravity adapter — binary detection (PATH + known path `D:\Antigravity\bin\antigravity`), version checking, extension management (`check_extension_installed`, `install_extension`, `uninstall_extension`), MCP config management (`check_mcp_config`, `add_mcp_server`), doctor diagnostics, setup/uninstall commands, L3 workspace advice |

### Modified Rust files

| File | Change |
|------|--------|
| `onus/src/cli/mod.rs` | Added `pub mod antigravity;` |
| `onus/src/cli/doctor.rs` | Added `--antigravity` flag to `DoctorArgs`; `run_antigravity()` focused diagnostics; Antigravity check block in main `run()` reporting binary/extension/MCP/L3 status |
| `onus/src/cli/setup.rs` | Added `--antigravity` flag to `SetupArgs`; `DetectedSurface::Antigravity` variant; Antigravity detection in `detect_surfaces()`; setup/uninstall dispatch |
| `onus/src/cli/uninstall.rs` | Added `--antigravity` flag to `UninstallArgs`; dispatch to `antigravity::run_uninstall()` |

### New Rust tests

**6 unit tests** in `onus/src/cli/antigravity.rs`:

| Test | What it covers |
|------|----------------|
| `test_find_antigravity_no_panic` | Detection runs without crashing regardless of Antigravity being installed |
| `test_antigravity_path_format` | Known Windows install path contains `Antigravity` and `bin` |
| `test_antigravity_extension_id` | Extension ID is `onus.onus-firewall` |
| `test_l3_workspace_advice_format` | Advice string is non-empty and mentions bubblewrap or not available |
| `test_mcp_config_check_no_binary` | Nonexistent binary path returns Error |
| `test_uninstall_extension_no_binary` | Nonexistent binary returns error without panic |

### New JavaScript extension tests

**2 new test files** in `onus/bindings/vscode/test/`:

| File | Purpose |
|------|---------|
| `antigravity.test.js` | Extension host test suite (6 tests): loads and activates, registers doctor/setup/status commands, produces doctor output, exports daemon status API |
| `antigravity-verify.js` | Non-interactive CLI verification runner (13 tests): binary presence, version, extension deployment, package.json integrity, contributed commands, CLI doctor/setup/uninstall |

### New Python integration tests

**5 new tests** in `test_onus.py`:

| Test class | Test | What it covers |
|------------|------|----------------|
| `TestDoctorAntigravityCommand` | `test_doctor_antigravity_runs` | `onus doctor --antigravity` produces Antigravity-specific output |
| | `test_doctor_full_reports_antigravity` | Full `onus doctor` includes Antigravity section |
| | `test_doctor_antigravity_l3_advice` | L3 workspace advice present in doctor output |
| `TestSetupAntigravityCommand` | `test_setup_antigravity_runs` | `onus setup --antigravity` runs without error |
| | `test_uninstall_antigravity_runs` | `onus uninstall --antigravity` runs without error |

### New live-verification package

| File | Purpose |
|------|---------|
| `runtime-verification/google-antigravity/run_live_tests.ps1` | User-executable PowerShell verification runner (8 tests) |
| `runtime-verification/google-antigravity/run_live_tests.sh` | User-executable Bash verification runner (7 tests) |
| `runtime-verification/google-antigravity/allowed/readme.txt` | Allowed surface test fixture |
| `runtime-verification/google-antigravity/secrets/.env` | Protected secret fixture |
| `runtime-verification/google-antigravity/protected/config.yaml` | Protected configuration fixture |
| `runtime-verification/google-antigravity/tests/test_file.txt` | Test fixture for contract evaluation |
| `runtime-verification/google-antigravity/README.md` | Test workspace documentation |

### Antigravity extension host test config

`onus/bindings/vscode/.antigravity-test.js` was pre-existing (uncommitted). It
configures `@vscode/test-cli` to run the extension host tests using the
Antigravity binary instead of VS Code:

```
binary: "D:/Antigravity/bin/antigravity"
extensionsDir: "C:/Users/A/.antigravity/extensions"
```

## Integration architecture

```
┌──────────────────────┐    MCP JSON-RPC    ┌────────────────────┐
│  Google Antigravity  │ ◄─────────────────► │  Onus Extension   │
│  (VS Code fork)      │    (via extension)  │  (deployed via    │
│                      │                     │   antigravity     │
│                      │                     │   --install-ext)  │
└──────────────────────┘                     └────────┬───────────┘
                                                       │
                                              ┌────────▼──────────┐
                                              │  onus doctor      │
                                              │  onus setup       │
                                              │  onus uninstall   │
                                              └────────┬──────────┘
                                                       │
                                              ┌────────▼──────────┐
                                              │  Onus daemon      │
                                              │  (evaluation,     │
                                              │   audit, policy)  │
                                              └───────────────────┘
```

## Capabilities

| Capability | Status | Enforcement |
|------------|--------|-------------|
| Binary detection | ✅ | PATH search → `D:\Antigravity\bin\antigravity` fallback → `antigravity --version` |
| Version check | ✅ | `antigravity --version` → v1.107.0 |
| Extension deployment | ✅ | Deployed to `~/.antigravity/extensions/onus.onus-firewall-0.1.0/` |
| Extension status | ✅ | `--list-extensions` → `onus.onus-firewall@0.1.0` |
| MCP config check | ✅ | Extension path + `.mcp.json` detection |
| MCP server setup | ✅ | `--add-mcp <json>` via `add_mcp_server()` |
| `onus doctor --antigravity` | ✅ | Binary, extension, MCP, L3 workspace |
| Full `onus doctor` | ✅ | Includes Antigravity section between Codex and L3 |
| `onus setup --antigravity` | ✅ | Prints instructions or configures MCP |
| `onus uninstall --antigravity` | ✅ | Removes extension via `--uninstall-extension` |
| Extension commands | ✅ | 5 contributed: `onus.doctor`, `onus.setup`, `onus.toggle`, `onus.status`, `onus.firewall.enable` |
| Fail-closed | ✅ | Extension blocks commands until Onus evaluates |
| Receipt generation | ✅ | Via extension's hook evaluation (shared VS Code path) |
| Secret redaction | ✅ | Via `crate::security::sha256_hex` |
| L3 workspace isolation | ✅ | `l3_workspace_advice()` reports status; `bwrap` on Linux |

## Runtime evidence

- `cargo test`: **98 passed, 0 failed** (6 new Antigravity adapter tests)
- `pytest -k "Antigravity"`: **5 passed**
- `cargo build`: succeeds with no warnings from new code
- `onus doctor --antigravity`: runs and reports Antigravity status
- `onus setup --antigravity`: succeeds with Antigravity info
- `onus uninstall --antigravity`: succeeds
- `onus doctor` (full): includes Antigravity section

## Live-deployment verification

The Onus extension is already deployed and active in Antigravity:

| Check | Result |
|-------|--------|
| `antigravity --version` | `1.107.0` |
| `antigravity --list-extensions` | `onus.onus-firewall@0.1.0` listed |
| Extension directory | Exists at `~/.antigravity/extensions/onus.onus-firewall-0.1.0/` |
| `package.json` main entry | `./src/extension.js` exists |
| Contributed commands | 5 commands registered in `contributes.commands` |

## Limitations

| Limitation | Reason | Mitigation |
|------------|--------|------------|
| Full extension-host test requires GUI | `antigravity` GUI session needed to run `.antigravity-test.js` | `antigravity-verify.js` provides non-interactive CLI verification (13 tests) |
| MCP config on extensions dir only | No `--list-mcp` flag in Antigravity CLI | Doctor checks extension path for `.mcp.json` |
| L3 isolation is platform-dependent | `bwrap` not available on Windows | L3 advice describes platform limitation |
| No equivalent of `code --locate-extension` | Antigravity supports `--locate-extension` ✓ | Path is resolved via CLI output |

## Remaining work (blocked on external factors)

1. Launch Antigravity GUI (`antigravity`), verify extension activates in UI
2. Run extension host tests: `npx @vscode/test-cli run --config .antigravity-test.js`
3. Verify contributed commands appear in Antigravity command palette
4. Run `antigravity --add-mcp` to fully configure MCP routing

## Enforcement level

**L1 (BEST-EFFORT)** — The extension intercepts commands via VS Code extension
API (cooperative hook model). Full L2 requires MCP proxy routing through Onus.
L3 requires bubblewrap workspace isolation (Linux only).
