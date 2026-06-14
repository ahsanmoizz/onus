1. Executive Current-State Verdict
Onus today is a technical alpha / prototype: a Rust policy/audit core plus a Python Guardian SDK executor, local SQLite audit trail, CLI replay, dashboard, approval UI, MCP proxy code, shell wrapper, and VS Code extension skeleton.
Strongest working capability: Python Guardian -> Rust evaluator -> SQLite audit rows -> hash-chain verification -> replay/dashboard proof.
Largest missing security boundary: no L3/L4 containment. Direct shell, Python, SQLite, filesystem, network, and MCP-server access can bypass Onus unless the developer voluntarily routes execution through Guardian/proxy/hooks.
The repository does not yet support the full public “universal AI agent firewall” story. It supports a narrower safe claim: “local technical alpha that can govern actions routed through its SDK/CLI/proxy surfaces.”
2. Verified Working Capabilities
from onus import Guardian, OnusClient works.
cargo build passes.
cargo test passes: 32 Rust tests.
cargo clippy runs successfully but reports warnings.
Reality demo runs: [reality_demo.py](D:/Onus/onus/examples/reality_demo.py).
Guardian-governed shell action blocks rm -rf /important before execution.
Guardian-governed file write records real before_content and after_content.
rollback_last() restores original file contents.
Guardian-governed local API call records an api_call.
Guardian-governed SQLite insert records a db_mutation.
DROP TABLE items; escalates through SAFETY_009.
Correction reaches the simulated DemoAgent.
SQLite audit DB contains shell, file_write, api_call, db_mutation.
Blocked and escalated outcomes are stored.
Receipt/hash-chain verification passes.
Session replay displays ordered action sequence.
Dashboard /api/actions returns real DB rows.
Approval UI runs; approve endpoint transitions a pending approval to approved.
3. Partially Working Capabilities
Pre-execution interception: VERIFIED WITH LIMITATIONS. Strong in Python Guardian and potential MCP proxy; weak/best-effort in VS Code/shell hooks.
Approval gates: PARTIALLY IMPLEMENTED. UI/API works; MCP proxy can create pending approvals; Guardian escalation raises an exception, not a reusable approval workflow.
Task contracts/scope: PARTIALLY IMPLEMENTED. ScopeTracker exists, but Guardian demo does not start rich scoped sessions with declared files.
Rollback/checkpoints: PARTIALLY IMPLEMENTED. File rollback and SQLite backup restore exist in Guardian; no general checkpoint graph or multi-action transaction rollback.
Audit integrity: VERIFIED WITH LIMITATIONS. Hash-chained SQLite rows verify, but DB is writable and unsigned.
IDE hooks: PARTIALLY IMPLEMENTED. VS Code extension exists but can fail open and does not reliably prevent execution.
MCP: IMPLEMENTED BUT UNTESTED. Proxy code exists; no runtime MCP server/client validation was performed.
Correction loop: VERIFIED WITH LIMITATIONS. Rule-driven correction reaches a simulated agent; no real coding agent or LLM evaluator.
API governance: VERIFIED WITH LIMITATIONS. Guardian urllib path is governed; direct requests, httpx, raw sockets bypass.
DB governance: VERIFIED WITH LIMITATIONS. Guardian SQLite path is governed; direct sqlite3 bypasses.
4. Missing Capabilities
LLM/semantic evaluation with a configured model.
Real Claude Code, Cursor, Windsurf, Copilot, Codex CLI, Gemini CLI, etc. runtime proof.
L3 filesystem/process/network sandboxing.
L4 production credential/authority control.
Static hardcoded credential detection.
Test deletion/weakening detection.
Task completion evidence verification.
Signed receipts, key protection, external anchoring.
Production auth/authz for dashboard or approval UI.
Broad language/framework adapters for LangChain, LangGraph, CrewAI, OpenAI Agents SDK.
Real MCP integration test against an MCP server.
General rollback beyond simple file/SQLite support.
5. Claim Comparison Matrix
Claim	Evidence in code	Runtime evidence	Status	Limitation	Safe wording
Python SDK exists	[__init__.py](D:/Onus/onus/bindings/python/src/onus/__init__.py)	Import passed	VERIFIED	Thin wrapper over Rust	“Python SDK for local Guardian execution.”
Guardian imports	SDK __all__	Import passed	VERIFIED	Needs PYTHONPATH/install	“Guardian is available in SDK.”
Shell block before execution	Guardian.shell()	Demo blocked rm -rf	VERIFIED	Only through Guardian	“Blocks SDK-routed shell actions.”
File before/after capture	Guardian.file_write()	DB row has both contents	VERIFIED	Stores full content, possible secrets	“Captures file-write before/after when using Guardian.”
API governance	Guardian.api_call()	Demo local API action	VERIFIED WITH LIMITATIONS	Only this SDK path	“Can govern SDK-routed API calls.”
DB mutation governance	Guardian.db_execute() and DbMutation	Demo insert/drop	VERIFIED WITH LIMITATIONS	SQLite SDK path only	“Can govern SQLite mutations routed through Guardian.”
Approval UI	[approvals.rs](D:/Onus/onus/src/cli/approvals.rs)	HTTP approve smoke passed	VERIFIED WITH LIMITATIONS	No auth; not tied to all surfaces	“Local approval UI/API exists.”
Hash chain	[db.rs](D:/Onus/onus/src/audit/db.rs)	verify passed	VERIFIED WITH LIMITATIONS	Tamper-evident, not tamper-resistant	“Hash-chained local audit log.”
Universal agent firewall	Docs/site	No broad runtime proof	DOCUMENTED ONLY	Many bypasses	“Prototype integrations for selected surfaces.”
LLM correction loop	none found	no API keys/model	MISSING	Rule-only simulated loop	Do not claim.

6. Component Status Matrix
Component	Location	Exists	Tested	Status	Main problem
Rust core	[lib.rs](D:/Onus/onus/src/lib.rs)	Yes	Yes	VERIFIED	No mandatory boundary
Python Guardian	[__init__.py](D:/Onus/onus/bindings/python/src/onus/__init__.py)	Yes	Demo	VERIFIED WITH LIMITATIONS	Voluntary use
Policy engine	[engine.rs](D:/Onus/onus/src/policy/engine.rs)	Yes	Rust tests/demo	VERIFIED	Mostly regex/heuristic
Rules	[default.toml](D:/Onus/onus/rules/default.toml)	Yes	Demo	VERIFIED WITH LIMITATIONS	Missing hardcoded secret/test weakening rules
Audit DB	[db.rs](D:/Onus/onus/src/audit/db.rs)	Yes	Demo/verify	VERIFIED	Local writable DB
Session replay	[session.rs](D:/Onus/onus/src/cli/session.rs)	Yes	Demo DB	VERIFIED WITH LIMITATIONS	Payload truncation; generic session metadata
Dashboard	[dashboard.rs](D:/Onus/onus/src/cli/dashboard.rs)	Yes	HTTP smoke	VERIFIED WITH LIMITATIONS	No auth
Approvals	[approvals.rs](D:/Onus/onus/src/cli/approvals.rs)	Yes	HTTP smoke	VERIFIED WITH LIMITATIONS	No auth/replay protections
MCP proxy	[proxy.rs](D:/Onus/onus/src/mcp/proxy.rs)	Yes	Not full protocol	IMPLEMENTED BUT UNTESTED	Needs real MCP test
VS Code extension	[extension.js](D:/Onus/onus/bindings/vscode/src/extension.js)	Yes	Not runtime	PARTIALLY IMPLEMENTED	Fail-open, after-event hooks
Shell wrapper	[onus-shell-wrapper.sh](D:/Onus/onus/scripts/onus-shell-wrapper.sh)	Yes	Not runtime	IMPLEMENTED BUT UNTESTED	Shell-trap fragility

7. Integration Coverage Matrix
Platform	Adapter	Mechanism	Level	Runtime-tested	Covered actions	Missing coverage	Status
Claude Code	Partial	preToolUse JSON translator/install scripts	L1	No	Bash/Write/Edit mapping	Real Claude hook proof	PARTIALLY IMPLEMENTED
Cursor IDE/CLI	VS Code extension only	VS Code APIs	L1	No	Terminal/task best-effort	Native Cursor agent tools	DOCUMENTED ONLY/PARTIAL
Windsurf/Cascade	VS Code-style claim	none specific	L1	No	none proven	native hooks	DOCUMENTED ONLY
VS Code Copilot	VS Code extension	terminal/task notifications	L1	No	terminal/task only	Copilot tool internals	PARTIALLY IMPLEMENTED
GitHub Copilot SDK	none	none	L0	No	none	SDK middleware	MISSING
Google Antigravity	none	none	L0	No	none	adapter	MISSING
Gemini CLI	none	shell wrapper if user opts in	L1	No	shell best-effort	native tool calls	MISSING
OpenAI Codex CLI	none	shell wrapper if user opts in	L1	No	shell best-effort	native tool calls	MISSING
Cline	MCP proxy possible	MCP routing	L2 if routed	No	MCP tools	direct tools	IMPLEMENTED BUT UNTESTED
Continue	MCP proxy possible	MCP routing	L2 if routed	No	MCP tools	direct tools	IMPLEMENTED BUT UNTESTED
Aider	shell wrapper possible	bash/zsh hook	L1	No	shell commands	file writes/direct Python	IMPLEMENTED BUT UNTESTED
JetBrains Junie	none	none	L0	No	none	plugin/adapter	MISSING
OpenAI Agents SDK	Guardian only	manual SDK use	L2 for wrapped tools	Demo agent only	wrapped Python tools	official middleware	PARTIALLY IMPLEMENTED
LangChain/LangGraph	none specific	manual Guardian possible	L2 if wrapped	No	wrapped tools	callbacks/middleware	MISSING
CrewAI	none specific	manual Guardian possible	L2 if wrapped	No	wrapped tools	tool adapter	MISSING
MCP-compatible agents	MCP proxy	stdio proxy	L2 if routed	No	tools/call	direct MCP server access	IMPLEMENTED BUT UNTESTED

8. Security and Bypass Findings
Critical
Affected: architecture/all adapters. Cause: Onus is not mandatory; direct shell/file/API/DB access bypasses it. Impact: agent can act outside Onus. Repro: write file with normal Python Path.write_text. Fix: L3 sandbox/executor that owns filesystem/network/process authority.
Affected: [__init__.py](D:/Onus/onus/bindings/python/src/onus/__init__.py), [dashboard.rs](D:/Onus/onus/src/cli/dashboard.rs). Cause: full payloads/bodies/contents logged and served. Impact: secrets can leak into SQLite/dashboard. Repro: Guardian file write containing token. Fix: redaction, field classification, encrypted payload storage, dashboard masking.
High
Affected: [extension.js](D:/Onus/onus/bindings/vscode/src/extension.js). Cause: binary missing/disabled/evaluate failure returns { decision: 'allow' }; terminal hooks notify after command event. Impact: fail-open. Repro: unset binary path. Fix: fail-closed option and true pre-exec terminal profile.
Affected: [proxy.rs](D:/Onus/onus/src/mcp/proxy.rs). Cause: approval lookup by session_id + tool_name, not exact payload/action hash. Impact: approval may authorize later different payload for same tool. Repro: approve pending tool, retry with altered args. Fix: bind approvals to action payload hash.
Affected: local SQLite audit. Cause: no signing/key protection; DB can be edited then hashes recomputed. Impact: not immutable/tamper-resistant. Repro: modify DB with sqlite client. Fix: signed receipts with protected key and external anchoring.
Medium
Affected: [__init__.py](D:/Onus/onus/bindings/python/src/onus/__init__.py). Cause: TOCTOU between evaluation and execution; file or command context can change. Impact: approved payload may differ from actual side effect. Repro: alter target file after evaluation before write in custom code. Fix: executor-owned immutable action objects and post-execution receipt comparison.
Affected: [rules/default.toml](D:/Onus/onus/rules/default.toml). Cause: no static hardcoded secret or test weakening rules. Impact: agent can add secrets/delete tests undetected. Repro: Guardian file write with AWS_SECRET_ACCESS_KEY="x". Fix: file-content policy rules and AST/diff checks.
Affected: approval/dashboard servers. Cause: localhost HTTP, no auth/CSRF. Impact: local process/browser can approve or read actions. Repro: POST to approval endpoint. Fix: random token, origin checks, auth.
Low
Affected: cargo clippy. Cause: 24 style/maintainability warnings. Impact: code quality drift. Repro: cargo clippy. Fix: apply clippy suggestions.
Affected: session metadata. Cause: Guardian records session start as action; audit auto-creates generic session metadata. Impact: replay says agent onus-cli, workspace unknown. Fix: Guardian should call real session-start API.
9. Test and Build Results
Command	Result	Passed	Failed	Skipped	Environment limitation
cargo test	pass	32	0	0	none
cargo build	pass	n/a	0	n/a	none
cargo clippy	pass with warnings	n/a	0	n/a	24 warnings
python -m pytest -q	not run	0	0	n/a	pytest not installed
python onus\examples\reality_demo.py	pass	n/a	0	n/a	simulated agent
onus verify --db reality_demo_audit.db	pass	n/a	0	n/a	local DB only
Dashboard /api/actions	pass	7 rows	0	n/a	no auth
Approval UI/API smoke	pass	pending -> approved	0	n/a	smoke DB, not MCP runtime

10. Current Enforcement Reality
L0 Observation only: dashboard, audit log viewing, session replay.
L1 Cooperative hook / best-effort: Claude Code hook translator/install scripts, shell wrapper, VS Code extension.
L2 Onus-owned executor/proxy: Python Guardian methods; MCP proxy if agent is actually routed through it.
L3 Isolated boundary: none.
L4 Production authority/credentials: none.
11. Publicly Claimable Today
“Onus is a local technical alpha for governing AI-agent actions routed through its SDK or CLI.”
“The Python Guardian SDK can pre-check shell commands, file writes, API calls, and SQLite mutations.”
“Onus records governed actions in a local SQLite audit trail with hash-chain verification.”
“Onus can replay local sessions and serve a local dashboard from real audit data.”
“Onus includes prototype Claude Code, VS Code, shell, and MCP integration surfaces.”
“Onus can return deterministic correction messages for blocked or escalated actions.”
12. Claims That Must Not Be Published Yet
“Universal AI agent firewall.”
“Cannot be bypassed.”
“Production-ready.”
“Immutable ledger.”
“Tamper-proof audit log.”
“LLM-evaluated correction loop.”
“Works with Claude Code/Cursor/Windsurf/Copilot/etc. end-to-end.”
“Detects hardcoded credentials.”
“Detects deleted/weakened tests.”
“Provides OS/container isolation.”
“Controls production credentials or deployment authority.”
“All MCP-compatible agents are protected automatically.”
13. Top Ten Next Engineering Tasks
L3 sandbox/executor: reason: close critical bypass; files: new executor/sandbox modules; scope: subprocess/filesystem/network containment; acceptance: direct writes blocked outside sandbox; improvement: L3.
Secret redaction/classification: reason: prevent audit leaks; files: SDK, audit, dashboard; acceptance: token-like values masked; improvement: safer logging.
Exact approval binding: reason: prevent approval reuse; files: [proxy.rs](D:/Onus/onus/src/mcp/proxy.rs), [db.rs](D:/Onus/onus/src/audit/db.rs); acceptance: changed payload needs new approval; improvement: safer L2.
Real MCP integration test: reason: prove proxy; files: src/mcp, tests; acceptance: fake MCP server tool call blocked/allowed; improvement: validated L2.
Real Claude Code hook test: reason: prove flagship integration; files: installer/evaluate; acceptance: Claude hook blocks command in live session; improvement: validated L1.
Task contract API: reason: enforce scope/completion; files: scope, SDK; acceptance: declared files enforced in Guardian; improvement: stronger L2.
Hardcoded secret/test weakening rules: reason: key product claims; files: rules/policy engine; acceptance: demo writes secret/deletes test and gets blocked/warned; improvement: coverage.
Signed receipts: reason: tamper resistance; files: audit/verify; acceptance: DB edit without key fails verification; improvement: audit integrity.
Fail-closed modes: reason: reduce hook bypass; files: VS Code/shell/SDK; acceptance: missing evaluator blocks in strict mode; improvement: safer L1/L2.
Python test environment: reason: CI confidence; files: pyproject/CI; acceptance: python -m pytest -q passes in clean env; improvement: DX/reliability.
14. Final Scorecard
Core architecture: 6/10
Deterministic policy engine: 7/10
Interception: 4/10
Corrections: 5/10
Rollback: 4/10
Audit integrity: 5/10
Developer experience: 5/10
Adapter coverage: 3/10
L3 security: 0/10
Production readiness: 2/10
“Onus currently is a promising local technical alpha with a verified SDK-routed enforcement path, and the next milestone required to make it a credible product is an Onus-owned L3 executor/sandbox plus one runtime-tested flagship agent integration.