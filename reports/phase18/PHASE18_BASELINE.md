# Phase 18 Baseline Report

## Timestamp
2026-06-18T00:00:00Z

## Branch
`phase18/existing-agent-continuity` (based on `phase17/whitepaper-reality-closure` commit 96f733b)

## Repository State
- Working tree: **clean**
- Binary build: **PASS** (0 warnings, 0 errors on `cargo build`)
- Tests: not yet run (baseline capture only)
- Spec lock: **PASS** (python tools/spec_lock/verify_spec_lock.py)

## All Registered CLI Commands (from main.rs)
1. approvals      — Approval workflow (list, show, approve, deny, cancel, serve)
2. authority      — Credential authority management
3. claude-hook    — Claude Code CLI hook handler
4. evaluate       — Evaluate an action against policy
5. daemon         — Daemon mode
6. contract       — Task contract management
7. dashboard      — Dashboard server
8. intake         — Intake Guardian
9. status         — System status
10. log           — View audit log
11. run           — Run a command under Onus
12. session       — Session management
13. rules         — Policy rules management (install, verify, sign, revoke, generate-keys)
14. upgrade       — Self-upgrade
15. doctor        — System diagnostics
16. setup         — Surface integration setup
17. uninstall     — Remove integrations
18. mcp-proxy     — MCP proxy server
19. shell         — Shell subcommand
20. cursor-hook   — Cursor IDE hook handler
21. verify        — Verify audit receipts
22. checkpoint    — Create/list/inspect/restore checkpoints
23. rollback      — Rollback by action/group/session
24. compensation  — Compensation inspect/execute
25. workspace     — L3 workspace management
26. memory        — Memory lifecycle (list, inspect, export, delete, archive, retention, incidents)

## Phase 17 Claims (from PHASE17_COMPLETION_REPORT)

### Claim 1: Rollback & Recovery (R0-R4)
- checkpoint create/list/inspect/restore CLI
- rollback by action ID, group ID, or session ID
- compensation inspection and execution

### Claim 2: Human Approval Workflow
- 6 subcommands: list, show, approve, deny, cancel, serve
- Approval binding validation (pending status, expiry)
- Local approval UI server with security headers

### Claim 3: Dashboard Security
- CSRF (Origin/Referer header validation)
- Rate limiting (60 req/min/IP)
- Security headers (CSP, X-Content-Type-Options, X-Frame-Options, Referrer-Policy)

### Claim 4: Signed Policies
- Ed25519 key generation
- Policy signing and verification
- Policy installation with backup

### Claim 5: L4 Authority
- Credential authority management
- Controlled operation execution
- Sovereign mode

### Claim 6: MCP L2 Enforcement
- MCP proxy intercept/filter/forward

### Claim 7: L3 Workspace
- Linux: bubblewrap sandbox
- Windows: base directory/network isolation
- Create, exec, stop workspace commands

### Claim 8: Content-Aware Secret Detection
- JWT, connection string, PEM, high-entropy string detection

### Claim 9: Quality Obligations
- Quality runner with verification checks

### Claim 10: Memory Lifecycle
- List, inspect, export, delete, archive, retention, incidents

### Claim 11: Packaging, Setup, Doctor
- Install scripts, surface auto-detection, targeted doctor

### Claim 12: Product Frontend (onus-console)
- Next.js dashboard with Tailwind CSS

### Claim 13: Official Website (onus/site)
- Dark-themed product landing page

### Claim 14: Provider Configs & Runbooks
- 12 integration runbooks

### Claim 15: Whitepaper Acceptance Suite
- Python runner: 11/15 pass, 4 skip
- Bash runner: mirrors Python

## Phase 17 Test Results (claimed)
- `cargo test`: 140 passed, 0 failed
- `cargo build`: clean compile, 0 errors
- `cargo clippy`: 4 warnings
- Python acceptance: 11 passed, 0 failed, 4 skipped
- Next.js build: clean, 0 errors

## Current Enforcement Levels (claimed)
- L0: dashboard, audit viewer, session replay — VERIFIED
- L1: Claude Code, Codex CLI, Antigravity, Cursor, VS Code hooks — VERIFIED
- L2: MCP proxy, evaluate CLI, approval workflow — VERIFIED
- L3: Linux bubblewrap sandbox, Windows base isolation — IMPLEMENTED
- L4: Credential authority, controlled execution — VERIFIED

## Environment
- OS: Windows 11 Pro (10.0.22631), MINGW64
- Rust: 1.96.0
- Python: 3.12.5
- Node: 24.15.0, npm: 11.12.1
- WSL: NOT available
- L3 sandbox: BLOCKED (requires Linux + bubblewrap)

## Existing Adapter Surfaces (production adapters, not skeletons)
1. **Claude Code CLI** — L1 BEST-EFFORT hook via `onus claude-hook`
2. **OpenAI Codex CLI** — MCP proxy via `~/.codex/config.toml`
3. **Cursor IDE** — hooks + MCP via `~/.cursor/hooks.json` + `~/.cursor/mcp.json`
4. **Google Antigravity** — extension + MCP via VSIX + `--add-mcp`

## Verification Plan

### Phase 17 Verification Order
1. Rollback/Recovery (R0-R4) — test via disposable git repo
2. Human Approval Workflow — test serve/approve/deny via HTTP
3. Dashboard and Console — test /api/actions endpoint
4. Signed Policies — test key generation + signing
5. L4 Authority — test credential authority commands
6. MCP L2 Proxy — verify proxy code compiles and routes
7. L3 Workspace — attempt Windows isolation (note: L3 blocked)
8. Secret Detection — verify redact pipeline
9. Quality Obligations — verify quality runner
10. Memory Lifecycle — verify memory CLI commands
11. Packaging and Doctor — verify doctor diagnostics
12. Whitepaper Acceptance — run acceptance suite

### Repair Loop
- Any Phase 17 engineering defect found will be repaired in production code
- Re-verify after repair

### Cross-Agent Continuity (after verification)
- 6 sources of truth: task contract, repo state, session memory, project memory, policy/incident context, audit/receipt
- Adapters: Claude Code CLI → Codex CLI (preferred proof)
- Session leases with crash recovery
- Handoff manifests with schema v1
