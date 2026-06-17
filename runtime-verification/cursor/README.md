# Onus — Cursor IDE Live Verification

This directory contains user-executable verification scripts for the Cursor IDE
integration surfaces (P15E-04/12/16).

## Prerequisites

- Onus built: `cargo build` in the repo root
- Cursor IDE installed (optional for testing CLI commands)
- PowerShell 5+ (Windows) or Bash 4+ (Linux/macOS)

## Running

### Windows (PowerShell)

```powershell
.\runtime-verification\cursor\run_live_tests.ps1
```

### Linux/macOS (Bash)

```bash
chmod +x runtime-verification/cursor/run_live_tests.sh
./runtime-verification/cursor/run_live_tests.sh
```

## Test Fixtures

| Directory | Purpose |
|-----------|---------|
| `allowed/` | Files the hook should permit |
| `protected/` | Files L3 workspace should protect |
| `secrets/` | Credential-like files for redaction verification |
| `tests/` | Test fixture files |

## What's Verified

1. `onus doctor --cursor` produces Cursor-specific output
2. Full `onus doctor` includes a Cursor section
3. `onus setup --cursor` succeeds
4. `cursor --version` (optional, if Cursor is installed)
5. `onus uninstall --cursor` succeeds
6. L3 workspace advice present
7. `onus cursor-hook` reads stdin JSON and returns a verdict
8. cursor-hook output is valid JSON
