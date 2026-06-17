# L3 Release Gate — Containment Verification

**Date**: 2026-06-17
**Branch**: `codex/phase15-integrations`
**Phase**: 15E gate closure

---

## L3 definition

> Same as L2 (pre-execution interception and deterministic block/allow) **plus** OS-level containment of the agent process: filesystem isolation, network restrictions, and credential protection enforced by the operating system.

Onus L3 implementation uses **bubblewrap** (`bwrap`) on Linux. No equivalent exists on Windows (no WSL2 distro available).

---

## Environment assessment

| Requirement | Status |
|------------|--------|
| Linux OS | NOT MET — Windows 11 build 22631 |
| bubblewrap (`bwrap`) | NOT INSTALLED |
| WSL2 distro | NOT AVAILABLE — no distros installed |
| Docker | NOT INSTALLED |
| L3 containment test suite | NOT RUNNABLE — platform-dependent |

---

## Implementation status

The Rust codebase contains L3 workspace isolation logic in `src/workspace.rs`. This code:
1. Creates a bubblewrap sandbox profile per action
2. Mounts only the required paths
3. Drops network access by default
4. Runs the action inside the sandbox
5. Returns stdout/stderr/exit code

This was verified in earlier phases on Linux. On the current Windows environment, the tests are **compile-checked only** — `cargo build` and `cargo test` pass because the Linux-specific code is behind `#[cfg(target_os = "linux")]` or feature-gated.

---

## Containment tests (18 required — all BLOCKED BY PLATFORM)

| # | Test | Status | Notes |
|---|------|--------|-------|
| 1 | `bwrap --version` succeeds | BLOCKED | No bwrap on PATH |
| 2 | `bwrap` can find `onus` binary | BLOCKED | Cannot test without L3 env |
| 3 | Workspace root is writable inside sandbox | BLOCKED | Cannot test without L3 env |
| 4 | Outside paths are NOT writable | BLOCKED | Cannot test without L3 env |
| 5 | Network is blocked by default | BLOCKED | Cannot test without L3 env |
| 6 | Network can be explicitly allowed | BLOCKED | Cannot test without L3 env |
| 7 | Allowed command executes inside sandbox | BLOCKED | Cannot test without L3 env |
| 8 | Blocked command fails with OnusBlockError | BLOCKED | Cannot test without L3 env |
| 9 | Sandboxed process cannot write to /etc | BLOCKED | Cannot test without L3 env |
| 10 | Sandboxed process cannot read ~/.ssh | BLOCKED | Cannot test without L3 env |
| 11 | Onus binary crash does not break containment | BLOCKED | Cannot test without L3 env |
| 12 | Container cleanup after process exit | BLOCKED | Cannot test without L3 env |
| 13 | Container cleanup after SIGKILL | BLOCKED | Cannot test without L3 env |
| 14 | Deterministic denial via shell action | BLOCKED | Cannot test without L3 env |
| 15 | Nested shell process stays inside sandbox | BLOCKED | Cannot test without L3 env |
| 16 | Environment variable isolation | BLOCKED | Cannot test without L3 env |
| 17 | /tmp isolation per action | BLOCKED | Cannot test without L3 env |
| 18 | Resource limit enforcement (cpu/mem) | BLOCKED | Cannot test without L3 env |

---

## What would need to change for L3 on Windows

1. Install WSL2 with a Linux distro (Ubuntu recommended) — OR —
2. Switch to a Linux/macOS development machine — OR —
3. Implement Windows-native containment:
   - **AppContainer / Windows Sandbox**: Programmatic API exists but requires deep integration
   - **Job Objects**: Can limit CPU/memory but do not isolate filesystem
   - **Windows Defender Application Guard**: Enterprise-only, not suitable
   - **Integrity levels (MIC)**: Can prevent writes to high-integrity paths but no filesystem virtualization

Recommendation: **Install WSL2 with Ubuntu** is the lowest-cost path to unblock L3 testing on this machine.

---

## Patches since L2 reached

L2 enforcement was reached during Phase 15D (live LLM verification + Python SDK wrappers). Since then:

- **CrewAI adapter** (Phase 15E) — Proven block/allow for Python agent framework tools
- **Approval binding** — `action_id` uniqueness and `canonical_payload_hash` determinism hardened
- **Fail-closed** — Missing binary, missing contract, missing rules all raise errors, not silent pass

All of these are L2-compatible. No L3 work was attempted because the platform doesn't support it.

---

## L3 gate verdict

| Criterion | Verdict |
|-----------|---------|
| L3 code exists? | YES — bubblewrap workspace isolation in `src/workspace.rs` |
| L3 code compiles? | YES — behind `#[cfg(target_os = "linux")]` |
| L3 tests pass on Linux? | ASSUMED — verified in earlier phases |
| L3 tests pass here? | BLOCKED BY PLATFORM — not runnable on Windows |
| Can L3 be shipped in v1? | **NO** — not without Linux build or WSL2 |

**L3 is NOT RELEASABLE** in v1 from the current development environment.

---

*Verification date: 2026-06-17*
*Environment: Windows 11 build 22631, no WSL, no Docker, no bwrap*
