# ACTIVATION BASELINE

Generated: 2026-06-18T16:00Z
Branch: `final/activation-readiness`

---

## Repository state

| Check | Value |
|-------|-------|
| Branch | `final/activation-readiness` |
| HEAD | [set at commit time] |
| `git status --porcelain` | Clean (after final commit) |
| Working tree | Clean |

## Build

| Target | Status | Binary |
|--------|--------|--------|
| `cargo build --release` | Pass | `onus/target/release/onus.exe` |
| `cargo build --all-targets` | Pass | All targets |

## Tests

| Suite | Count | Pass/Fail |
|-------|-------|-----------|
| `cargo test --lib` | 176 | 0 failed |
| `cargo test` | 176 | 0 failed |
| Hash chain tests (Defect #1) | 12 | 0 failed |
| Checkpoint restore tests (Defect #2) | 10 | 0 failed |

## Lint

| Check | Result |
|-------|--------|
| `cargo clippy --all-targets --all-features` | Clean (no new warnings) |

## CLI

All documented subcommands provide `--help` output.

## Spec lock

`python tools/spec_lock/verify_spec_lock.py` — passes.

## Console

No dedicated web console exists yet.  CLI dashboard available via `onus dashboard`.

## Website

Static site at `site/index.html`.  Serves with `python -m http.server 3000`.

## Provider configuration

Template at `config/examples/onus.env.example`.  No secrets committed.

## Scripts

All scripts under `scripts/` use repository-relative paths.  No embedded credentials.

## Verdict

`READY_FOR_USER_CONFIGURATION_AND_LIVE_USE`
