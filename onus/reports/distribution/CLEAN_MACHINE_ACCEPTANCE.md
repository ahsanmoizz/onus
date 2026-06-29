# Onus Clean-Machine Acceptance Report

**Version:** 0.1.0
**Date:** 2026-06-19
**Commit:** 8ffad03

---

## Status: NOT EXECUTED

Clean-machine acceptance requires a disposable environment (VM, container, or fresh OS install)
that has never had Onus installed. This test cannot be performed from the development machine
where Onus was built and tested.

## Test Specification

The following test sequence should be executed on a clean machine:

### Phase 1: Download

```bash
# Windows
Invoke-WebRequest -Uri "https://github.com/ahsanmoizz/onus/releases/download/v0.1.0/install-onus.ps1" -OutFile "install-onus.ps1"

# Linux
curl -fsSL https://github.com/ahsanmoizz/onus/releases/main/install/install-onus.sh -o install-onus.sh
```

### Phase 2: Install

```bash
# Windows
powershell -ExecutionPolicy Bypass -File install-onus.ps1

# Linux
bash install-onus.sh
```

**Expected:** Binary installed to PATH, config directory created, next steps printed.

### Phase 3: First Run

```bash
onus doctor
```

**Expected:** System health check runs. Reports missing API key (expected for new install).
Detects Git availability. Reports platform info.

### Phase 4: Daemon

```bash
onus daemon start
onus daemon status
```

**Expected:** Daemon starts and reports running.

### Phase 5: Setup

```bash
onus setup --non-interactive --provider openai --api-key "sk-test..."
```

**Expected:** Provider configured, default guardian mode set.

### Phase 6: Console

```bash
onus console
```

**Expected:** Console web UI launches in browser.

### Phase 7: Activation

Open console at `http://localhost:9090/activate` and run through wizard.

**Expected:** Wizard completes with green checkmark.

### Phase 8: Evaluate

```bash
onus evaluate --prompt "list files in current directory"
```

**Expected:** Evaluation runs, returns allow or deny verdict.

### Phase 9: Uninstall

```bash
onus uninstall
```

**Expected:** Binary removed, config preserved.

```bash
onus uninstall --purge
```

**Expected:** Binary and config both removed.

---

## Preconditions for Clean-Machine Test

| Requirement | Status |
|---|---|
| GitHub release v0.1.0 published | NOT YET CREATED |
| Windows VM or test machine | NOT CONFIGURED |
| Linux VM or test machine | NOT CONFIGURED |
| Test API key (OpenAI or Anthropic) | NOT CONFIGURED |
| Timer for daemon start verification | NOT SET UP |

## Test Prerequisites (from user's local machine)

Already validated on the development machine:
- `cargo build --release` produces working binary at `target/release/onus.exe`
- 176 lib tests pass
- All CLI commands respond with help text
- Both frontend apps build successfully
- Installer scripts are syntactically valid

## Verdict

**CLEAN-MACHINE ACCEPTANCE: INCOMPLETE** — This test must be run on a disposable
environment before the v0.1.0 release is published. The installers, CLI, and frontend
have been verified on the development machine but full clean-machine flow requires
external testing infrastructure.
