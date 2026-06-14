# Integration Coverage

Date: 2026-06-14

## Coverage Levels

- L0: no implementation.
- L1: cooperative/best-effort hook or UI warning.
- L2: action is actually routed through Onus before execution.
- L3: contained execution boundary.
- L4: Onus-controlled authority or credentials.

## Integration Matrix

| Integration / surface | Code found | Runtime proof | Current level | Classification | Honest claim |
| --- | --- | --- | --- | --- | --- |
| Python SDK / Guardian | `onus/bindings/python/src/onus/__init__.py` | Reality demo plus import proof | L2 for Guardian-routed actions | VERIFIED WITH LIMITATIONS | Works for voluntary Python callers using Guardian. |
| Rust CLI evaluator | `onus/src/cli/evaluate.rs` | Direct probes and cargo tests | Core evaluator | VERIFIED | Evaluates individual action JSON and records audit rows. |
| Claude Code hook | `onus/src/cli/evaluate.rs`, `onus/install/install.*` | Unit translator tests only | L1 target | PARTIAL | Adapter code exists, but no live Claude Code session was proven. |
| VS Code extension | `onus/bindings/vscode/src/extension.js` | Source inspection only | L1 best-effort | PARTIAL | Can warn/check via VS Code APIs, but fail-open and not true terminal pre-exec. |
| Cursor | VS Code extension only | None | L1 theoretical | DOCUMENTED ONLY / PARTIAL | No native Cursor agent interception proven. |
| Windsurf / Cascade | VS Code-style claim only | None | L0/L1 theoretical | DOCUMENTED ONLY | No specific adapter proof. |
| GitHub Copilot in VS Code | VS Code extension only | None | L1 theoretical | PARTIAL / UNVERIFIABLE | No Copilot tool-call interception proof. |
| GitHub Copilot SDK | None found | None | L0 | MISSING | No SDK adapter. |
| Shell wrapper | `onus/scripts/onus-shell-wrapper.sh`, CLI shell command | Not runtime-tested in this audit | L1 best-effort | PARTIAL | Cooperative shell protection only if installed/sourced. |
| MCP proxy | `onus/src/mcp/proxy.rs` | Source inspection and direct `m_c_p` evaluator probe | Intended L2, currently unsafe default | PARTIAL / BROKEN | Proxy exists, but default policy lets destructive MCP payloads allow. |
| Cline | MCP proxy could be routed | None | L2 only if routed and fixed | UNVERIFIABLE / PARTIAL | Not proven. |
| Continue | MCP proxy could be routed | None | L2 only if routed and fixed | UNVERIFIABLE / PARTIAL | Not proven. |
| Zed/Cody/other MCP agents | MCP proxy could be routed | None | L2 only if routed and fixed | UNVERIFIABLE / PARTIAL | Not proven. |
| LangChain | Manual Guardian possible | None | L0/L2 manual only | MISSING | No callback/tool adapter. |
| LangGraph | Manual Guardian possible | None | L0/L2 manual only | MISSING | No adapter. |
| CrewAI | Manual Guardian possible | None | L0/L2 manual only | MISSING | No tool adapter. |
| OpenAI Agents SDK | Manual Guardian possible | None | L0/L2 manual only | MISSING | No official middleware/adapter. |
| REST API / daemon IPC | `onus/src/ipc/*`, `onus/src/daemon.rs` | Rust unit tests only | Internal/prototype | PARTIAL | IPC code exists; not validated as public integration. |
| Dashboard | `onus/src/cli/dashboard.rs` | HTTP 200 `/api/actions` with real rows | Read-only local UI | VERIFIED WITH LIMITATIONS | Shows real audit data locally; no auth/redaction. |
| Approval UI | `onus/src/approval/*`, `onus/src/cli/approvals.rs` | HTTP 200 root and pending API | Local approval UI | PARTIAL | Renders and reads pending approvals; no live MCP approval flow proof. |
| Figma/design assets | None relevant to runtime | None | L0 | MISSING | No operational integration. |
| JetBrains / Junie | None found | None | L0 | MISSING | No adapter. |
| Gemini CLI | None found | None | L0 | MISSING | No adapter. |
| Codex CLI | None found | None | L0 | MISSING | No adapter. |
| Aider | None found | None | L0 | MISSING | No adapter. |

## Integration-Specific Findings

### Python Guardian

Status: VERIFIED WITH LIMITATIONS.

Capabilities proven:

- imports successfully;
- evaluates before SDK-managed side effects;
- writes file only after allow;
- captures before/after file content;
- blocks destructive shell command before execution;
- calls local API after allow;
- executes SQLite insert after allow;
- escalates destructive SQLite mutation before execution;
- restores file write via rollback stack.

Limitations:

- fail-open on malformed evaluator output;
- raw contents stored in audit;
- no framework-specific adapters;
- no real LLM agent loop.

### Claude Code

Status: PARTIAL.

Evidence:

- CLI translator maps hook tool names such as `Bash`, `Write`, and `Edit`.
- Rust tests cover hook translation.
- Install scripts attempt to write `preToolUse`.

Limitations:

- no live Claude Code run;
- installer path risk in bash script;
- no proof that correction output is consumed by Claude in this workspace.

### VS Code / Cursor / Windsurf / Copilot

Status: PARTIAL to DOCUMENTED ONLY.

Evidence:

- VS Code extension exists.
- Extension can evaluate commands/config and show warning messages.

Limitations:

- returns allow when binary missing, disabled, or evaluation fails;
- terminal/task events are not proven as true pre-execution gates;
- no native Cursor/Windsurf/Copilot tool-call adapter.

### MCP

Status: PARTIAL / BROKEN default policy.

Evidence:

- `onus mcp-proxy` code spawns a real MCP server and inspects `tools/call`.
- It wraps calls as `ActionType::MCP`.

Critical gaps:

- default ruleset has no `mcp` rules;
- direct evaluator probe for destructive `m_c_p` action returned `allow`;
- external JSON spelling `mcp` fails to parse while rule config/display use `mcp`;
- normal MCP actions are not recorded into the audit DB by the proxy path, only pending approvals are persisted;
- no full fake-real MCP server integration test was run.

### Dashboard and Approval UI

Status:

- Dashboard: VERIFIED WITH LIMITATIONS.
- Approval UI: PARTIAL.

Evidence:

- Dashboard `/api/actions` returned HTTP 200 and real JSON from the demo DB.
- Approval UI `/` returned HTTP 200 and `/api/pending` returned `[]`.

Limitations:

- no authentication;
- no redaction;
- approval is not exact-payload bound;
- approval UI was not verified in a live MCP escalation flow.

## Coverage Summary

Working today:

- Rust evaluator;
- Python Guardian;
- local SQLite audit/hash verification;
- session replay;
- local dashboard;
- local approval UI rendering;
- prototype adapter code.

Not yet covered:

- real external-agent proof;
- real MCP safety with default rules;
- LLM provider integrations;
- framework adapters;
- production containment;
- credential authority;
- exact approval security.

The current maximum honest enforcement claim is:

`L2 for Python Guardian-routed actions; L1/BEST-EFFORT or unverified for most other integrations; no L3/L4.`
