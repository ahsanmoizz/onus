# Phase 17 — Baseline Report

**Date:** 2026-06-18
**Starting commit:** `aa3874972b196826d8c338d6d9e17cf84c6ec0a7`
**Starting branch:** `codex/phase15-integrations`
**Phase 17 branch:** `phase17/whitepaper-reality-closure`

---

## Environment

| Property | Value |
|---|---|
| Operating system | Windows 11 Pro 10.0.22631 |
| WSL version | WSL 2.7.8.0, Kernel 6.1.83.1-1 |
| Linux availability | WSL2 available |
| Node.js | v24.15.0 |
| npm | 11.12.1 |
| Rust | 1.96.0 (ac68faa20 2026-05-25) |
| Cargo | 1.96.0 |
| Python | 3.12.5 |
| Git | 2.54.0.windows.1 |
| Shell | MINGW64/MSYS2 (Git Bash) |

---

## Baseline test results

| Suite | Count | Result |
|---|---|---|
| Rust lib tests (`cargo test --lib`) | 120 | 120 pass, 0 fail |
| Python spec lock tests | 6 | 6 pass, 0 fail |
| Python SDK tests | TBD | TBD |
| Rust release build | — | PASS |
| Rust debug build | — | PASS |
| Clippy warnings | 8 | All pre-existing, non-security |
| Spec lock verification | — | PASS |

---

## Known whitepaper gaps (from Phase 16 matrix)

| Gap | Phase to close |
|---|---|
| Rollback commands not fully wired to CLI (checkpoint restore, group/session rollback) | 17.1 |
| Human approval workflow lacks real interactive surface | 17.2 |
| Dashboard unauthenticated | 17.3 |
| Signed policies not implemented | 17.4 |
| L4 broker not wired to real production operations | 17.5 |
| MCP enforcement not tested with real client/server | 17.6 |
| Windows L3 not implemented | 17.7 |
| Secret detection uses pattern matching only — no content inspection | 17.8 |
| Quality completion evidence not enforced via controlled runner | 17.9 |
| Memory lifecycle commands missing | 17.10 |
| Packaging/setup incomplete | 17.11 |
| No product frontend | 17.12 |
| No official website | 17.13 |
| Provider integration templates and live-test runbooks missing | 17.14 |
| No single executable acceptance harness for whitepaper scenarios | 17.15 |

---

## Installed IDEs and CLIs

| Tool | Status |
|---|---|
| VS Code | Available |
| Cargo/Rust | Available |
| Python/pytest | Available |
| Node/npm | Available |
| Git | Available |
| Docker | TBD |
| bubblewrap | Linux-only (WSL candidate) |
