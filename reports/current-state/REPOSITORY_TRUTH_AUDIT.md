# Onus Repository Truth Audit

Date: 2026-06-14
Workspace: `D:\Onus`
Mode: evidence-backed audit only. No application code or locked documents were modified.

## Scope

Documents read and compared against implementation:

- `AGENTS.md`
- `MANIFESTO.md`
- `SPEC.md`
- `docs/Onus_Whitepaper.txt`
- `docs/ONUS_PRODUCT_VISION.md`
- `docs/ONUS_TARGET_ARCHITECTURE.md`
- `docs/ONUS_SECURITY_REQUIREMENTS.md`
- `docs/ONUS_ACCEPTANCE_TESTS.md`
- `docs/ONUS_IMPLEMENTATION_ROADMAP.md`
- `docs/Onus_current_state.md`

Canonical path mismatch found:

- `AGENTS.md` names `docs/Onus_Whitepaper.md`; the repository contains `docs/Onus_Whitepaper.txt`.
- `AGENTS.md` names `docs/ONUS_CURRENT_STATE.md`; the repository contains `docs/Onus_current_state.md`.

This audit treats the existing files as the available canonical/current-state documents, but the mismatch itself is a repository integrity issue.

## Executive Truth

Onus is a working technical alpha around a Rust evaluator plus a Python SDK/Guardian. The most functional path today is:

Python `Guardian` -> Rust `onus evaluate` -> deterministic rule engine -> SQLite audit DB with hash-chain verification -> CLI replay/dashboard.

That path can show real pre-action checks, real file before/after capture, real local API and SQLite side effects, a blocked destructive shell command, a destructive DB mutation escalated before execution, and a simple rollback for file writes/SQLite backups.

What cannot honestly be claimed yet:

- universal agent firewall;
- production-grade execution boundary;
- LLM semantic evaluation loop;
- exact approval binding to canonical payload hash;
- secret-safe audit/dashboard storage;
- real end-to-end Claude Code, Cursor, Windsurf, Copilot, LangChain, CrewAI, OpenAI Agents SDK, or MCP-agent proof;
- L3 containment or L4 credential/authority control.

Rough implementation reality against the locked target product: about 30-35 percent has executable code, and a smaller subset is runtime-proven. The production/security guarantee layer is still mostly missing.

## Runtime Evidence Collected

Repository inventory:

- `rg --files --hidden -g '!onus/target/**' -g '!**/__pycache__/**' -g '!**/.git/**'` listed the repository, including hidden CI/spec-lock files.
- `rg --files -g '!onus/target/**' -g '!**/__pycache__/**'` counted 98 non-target files before these reports were added.
- Root `git status --short` showed a dirty tree and deleted/recreated generated artifacts. Audit proceeded without reverting any user/generated work.

Build and test evidence:

- `cargo test` in `D:\Onus\onus`: PASSED, 32 tests.
- `cargo build` in `D:\Onus\onus`: PASSED.
- `cargo clippy` in `D:\Onus\onus`: PASSED with 24 warnings.
- `python -m pytest -q` in `D:\Onus\onus`: BROKEN in this environment, `No module named pytest`.

SDK/import evidence:

- `python -c "from onus import Guardian; print('import=ok', Guardian)"`: PASSED.
- `python -m pip show onus`: editable install points to `D:\Onus\onus\bindings\python`.

Safe demo evidence:

- `python onus\examples\reality_demo.py`: PASSED.
- Demo output included:
  - `import_guardian=ok`
  - `file_write_verdict=allow`
  - `file_before="original file contents\n"`
  - `file_after="new proposed file contents\n"`
  - `rollback_action=file_write`
  - `file_restored="original file contents\n"`
  - `api_call_verdict=allow`
  - `db_insert_verdict=allow`
  - `db_drop_blocked=escalate`
  - `db_drop_rule=SAFETY_009`
  - `correction_loop_received=[...]`

Audit DB evidence:

- `onus verify --db D:\Onus\reality_demo_workspace\reality_demo_audit.db`: `Hash chain integrity verified: ALL PASS`.
- Real rows queried from SQLite: 7 actions.
- Latest session: `guardian-6347a238-e905-4d85-91e1-21a309656a90`.
- Real action sequence:
  - 1 `shell` / `guardian_session_start` / `allow`
  - 2 `file_write` / `DemoAgent.Write` / `allow`
  - 3 `api_call` / `DemoAgent.ApiCall` / `allow`
  - 4 `db_mutation` / `DemoAgent.SQLite` / `allow`
  - 5 `db_mutation` / `DemoAgent.SQLite` / `escalate` / `SAFETY_009`
  - 6 `shell` / `Bash` / `block` / `SAFETY_001`
  - 7 `file_write` / `DemoAgent.Write` / `allow`

Replay/dashboard evidence:

- `onus session guardian-6347a238-e905-4d85-91e1-21a309656a90 --db ...`: replay showed all 7 steps with payload previews and corrections.
- `onus dashboard --db ... --port 8787`, then `GET /api/actions`: status 200 with real JSON rows from the audit DB.
- `onus approvals --db ... --port 9191`, then `GET /` and `GET /api/pending`: status 200, UI rendered, pending response `[]`.

Policy probes:

- Shell `rm -rf /important`: `block`, `SAFETY_001`, exit code 2.
- DB `DROP TABLE users;`: `escalate`, `SAFETY_009`, exit code 3.
- Shell `printenv`: `block`, `SAFETY_004`, exit code 2.
- Large file write: `warn`, `MAGNITUDE_001`, exit code 1.
- Multi-URL API payload: `warn`, `MAGNITUDE_001`, exit code 1.
- Hardcoded secret file write: `allow`, exit code 0.
- Test file delete: `allow`, exit code 0.
- MCP action using wire value `m_c_p` with destructive payload: `allow`, exit code 0.
- MCP action using wire value `mcp`: parse failure, because external JSON serde expects `m_c_p` even though rule config/display use `mcp`.

## Feature Classification

| Feature | Classification | Evidence | Limitations |
| --- | --- | --- | --- |
| Rust core crate and CLI binary | VERIFIED | `cargo build`, `cargo test`, `onus evaluate` probes | Clippy warnings remain. |
| Python SDK package | VERIFIED WITH LIMITATIONS | `from onus import Guardian` works; editable install points to repo | Installed in this environment; portability depends on packaging/install. |
| `Guardian.file_write` interception | VERIFIED | Demo captured real before/after and wrote file after allow | Only protects code voluntarily using Guardian. Payload stores full contents. |
| File before/after capture | VERIFIED WITH LIMITATIONS | Audit payload contains `before_content` and `after_content` | Secret redaction is missing. |
| `Guardian.shell` pre-execution block | VERIFIED WITH LIMITATIONS | Demo/probe blocked `rm -rf` before execution | Only for Guardian-owned execution path. |
| API call interception | VERIFIED WITH LIMITATIONS | Demo called local HTTP API after allow | No strong API risk rules beyond URL-count heuristic; headers/body previews may leak data. |
| SQLite DB mutation interception | VERIFIED WITH LIMITATIONS | Demo allowed insert and escalated `DROP TABLE` before execution | Only SQLite helper path; no general DB adapter or transaction policy. |
| Simple rollback for file writes | VERIFIED WITH LIMITATIONS | Demo restored file contents | Not generalized; no cryptographic receipt; secret leaks cannot be undone by rollback. |
| SQLite backup rollback support | PARTIAL | Python SDK copies DB before mutation | No runtime rollback probe in current demo for DB mutation; not transaction-aware. |
| Deterministic rule engine | VERIFIED WITH LIMITATIONS | `cargo test`, rule probes, `rules list` | Missing hardcoded-secret and test-weakening/deletion rules. |
| Reversibility classification | PARTIAL | Rule config supports reversible/compensable/irreversible | Mostly static metadata; no full recovery planner. |
| Audit SQLite DB | VERIFIED | Real DB rows queried; dashboard served rows | Local writable DB; full payload storage. |
| Hash-chain verification | VERIFIED WITH LIMITATIONS | `onus verify` passed; source recomputes SHA-256 chain | Tamper-evident only if attacker cannot recompute/replace DB; no signing/anchoring. |
| Session replay | VERIFIED WITH LIMITATIONS | `onus session` replayed real demo session | Metadata generic: agent `onus-cli`, task direct eval, workspace unknown. |
| Dashboard | VERIFIED WITH LIMITATIONS | `/api/actions` returned real DB JSON | No auth; serves raw payloads. |
| Approval UI | PARTIAL | UI rendered and `/api/pending` returned JSON | Not proven with live MCP flow; no auth/CSRF; approval not exact-payload bound. |
| MCP proxy | PARTIAL / BROKEN DEFAULT POLICY | Source intercepts `tools/call` | Default rules do not apply to `MCP`; destructive MCP probe allowed. No full MCP runtime test. |
| Claude Code hook adapter | PARTIAL | Translator tests pass; installer writes hook config | No live Claude Code run. Bash installer hardcodes `/usr/local/bin/onus evaluate`. |
| VS Code extension | PARTIAL | Extension source exists | Fail-open on missing/eval failure; terminal/task hooks are best-effort/after-the-fact. |
| Shell wrapper | PARTIAL | Bash wrapper source exists | Not runtime-tested; cooperative and disable-able. |
| LLM semantic roles | MISSING | No OpenAI/Anthropic/Gemini provider code found | API keys unset; no provider interface. |
| Prompt correction loop to real LLM agent | DEMO ONLY / PARTIAL | Demo agent receives deterministic correction string | No real LLM call; no automatic LLM re-plan loop. |
| Intent/task contract | DOCUMENTED ONLY | Target docs specify it | No implemented prompt intake/task contract hash. |
| Evidence-based completion verifier | MISSING | Acceptance docs require it | No completion verifier or receipt evidence gate. |
| Secret detection/redaction | MISSING | Secret write probe allowed; dashboard shows payload | Violates security requirement if claimed. |
| Test deletion/weakening detection | MISSING | Test delete probe allowed | Acceptance scenario B not implemented. |
| L3 containment | MISSING | No sandbox/process/network/credential containment found | Cannot claim L3. |
| L4 authority/credential broker | MISSING | No authority/credential broker found | Cannot claim L4. |

## Functional Logic Files

Files with actual Onus logic, not just stubs/comments:

- `onus/src/lib.rs`
- `onus/src/main.rs`
- `onus/src/policy/engine.rs`
- `onus/src/policy/rule.rs`
- `onus/rules/default.toml`
- `onus/src/audit/db.rs`
- `onus/src/audit/merkle.rs`
- `onus/src/cli/evaluate.rs`
- `onus/src/cli/log_cmd.rs`
- `onus/src/cli/session.rs`
- `onus/src/cli/verify.rs`
- `onus/src/cli/dashboard.rs`
- `onus/src/cli/approvals.rs`
- `onus/src/cli/mcp_proxy.rs`
- `onus/src/cli/rules.rs`
- `onus/src/cli/shell.rs`
- `onus/src/cli/status.rs`
- `onus/src/cli/uninstall.rs`
- `onus/src/cli/upgrade.rs`
- `onus/src/cli/daemon_cmd.rs`
- `onus/src/ipc/mod.rs`
- `onus/src/ipc/protocol.rs`
- `onus/src/ipc/client.rs`
- `onus/src/ipc/server.rs`
- `onus/src/daemon.rs`
- `onus/src/scope/tracker.rs`
- `onus/src/mcp/proxy.rs`
- `onus/src/mcp/mod.rs`
- `onus/src/approval/mod.rs`
- `onus/src/approval/index.html`
- `onus/bindings/python/src/onus/__init__.py`
- `onus/bindings/python/tests/test_onus.py`
- `onus/examples/reality_demo.py`
- `onus/bindings/vscode/src/extension.js`
- `onus/bindings/vscode/package.json`
- `onus/scripts/onus-shell-wrapper.sh`
- `onus/install/install.sh`
- `onus/install/install.ps1`
- `onus/install/npx-package/install.js`
- `tools/spec_lock/spec_lock.py`
- `tools/spec_lock/generate_manifest.py`
- `tools/spec_lock/verify_spec_lock.py`
- `tests/test_spec_lock.py`

Static or primarily presentation files:

- `site/index.html`
- `onus/site/index.html`
- `onus/bindings/python/README.md`

Generated/runtime artifacts found:

- `reality_demo_workspace/*`
- `fresh_test.db*`
- `test_audit.db`
- Python `*.egg-info` / `__pycache__` artifacts
- A suspicious root entry named `and normalize repository`

## Single Best Demo Today

The best honest 10-minute demo:

1. Show `from onus import Guardian` importing from the editable SDK.
2. Run `python onus\examples\reality_demo.py`.
3. Point to the output proving:
   - file before/after capture;
   - rollback restoring original content;
   - local API call gated by Onus;
   - SQLite insert gated by Onus;
   - `DROP TABLE` escalated;
   - `rm -rf` blocked;
   - deterministic correction surfaced to a demo agent.
4. Run `onus verify --db D:\Onus\reality_demo_workspace\reality_demo_audit.db`.
5. Run `onus session <session_id> --db ...`.
6. Start `onus dashboard --db ... --port 8787` and open `/api/actions`.

Claim for that demo: "Onus has a working local Python Guardian and Rust policy/audit core for governed actions routed through it."

Do not claim: "universal agent firewall", "production-ready", "LLM-evaluated", "secret-safe", or "protects all MCP agents".
