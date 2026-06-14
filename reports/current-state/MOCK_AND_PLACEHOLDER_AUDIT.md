# Mock and Placeholder Audit

Date: 2026-06-14

## Summary

No production path was found that obviously fabricates static dashboard rows as live audit data. The dashboard smoke test served real rows from `reality_demo_audit.db`.

However, several areas are demo-only, best-effort, fail-open, or claim more than the runtime evidence supports. The largest issue is not classic "mock data"; it is that some production/integration paths degrade to allow or are not connected to real agent runtimes.

## Demo-Only or Simulation Paths

| File / area | Classification | Evidence | Issue |
| --- | --- | --- | --- |
| `onus/examples/reality_demo.py` | DEMO ONLY | Uses `DemoAgent`, local HTTP server, local SQLite DB, and printed demo outputs | The file does not contain the required `DEMO_ONLY` or `SIMULATED` label from `AGENTS.md`. |
| `DemoAgent.receive_correction` in `reality_demo.py` | DEMO ONLY / PARTIAL | Receives deterministic correction string after blocked action | Not a real LLM agent and not an automatic provider-backed correction loop. |
| Local HTTP API in `reality_demo.py` | DEMO ONLY | `HTTPServer(("127.0.0.1", 0), DemoHandler)` | Valid safe demo, but not a production API integration. |
| `site/index.html` and `onus/site/index.html` | DOCUMENTED ONLY / PRESENTATION | Static marketing pages | Not the live dashboard; must not be presented as operational UI. |

## Fail-Open or Hardcoded-Allow Behavior

| File / area | Classification | Evidence | Risk |
| --- | --- | --- | --- |
| `onus/bindings/python/src/onus/__init__.py` | BROKEN SECURITY INVARIANT | If evaluator stdout cannot be parsed, SDK builds `{"decision": "allow"}` | Critical evaluator failure can silently fail open. |
| `onus/bindings/vscode/src/extension.js` | BROKEN SECURITY INVARIANT | Missing binary, disabled config, or evaluate error returns `{ decision: 'allow' }` | IDE protection fails open. |
| `onus/src/lib.rs` | BROKEN SECURITY INVARIANT | `evaluate_request` returns `Verdict::Allow` when default rules fail to load | MCP proxy and other library callers can fail open. |
| `onus/src/mcp/proxy.rs` | PARTIAL / BYPASS | Parse errors are forwarded to real MCP server | Malformed or nonstandard messages may bypass evaluation. |

## Placeholder or Under-Proven Integration Claims

| Claim area | Classification | Evidence | Reality |
| --- | --- | --- | --- |
| Claude Code live protection | PARTIAL | Hook translator tests pass; install scripts exist | No live Claude Code hook run was verified. |
| Cursor/Windsurf/Copilot protection | DOCUMENTED ONLY / PARTIAL | VS Code extension source exists | No native Cursor/Windsurf/Copilot tool interception verified. |
| MCP universal protection | BROKEN / PARTIAL | Proxy exists, but default `m_c_p` probe allowed destructive payload | Default rules do not protect MCP action type. |
| LangChain/CrewAI/OpenAI Agents SDK adapters | MISSING | No framework-specific adapter code found | Manual Guardian use only. |
| LLM semantic evaluation | MISSING | No provider code found; API keys unset | No real LLM call path. |

## Test Mocks

Legitimate isolated test-style scaffolding exists:

- `onus/bindings/python/tests/test_onus.py` uses pytest fixtures and temp paths.
- Rust unit tests create in-memory/simple policy rules.

These are acceptable as test scaffolding, but the Python test suite could not run because `pytest` is not installed in this environment.

## Mock-Like Documentation Risk

Some docs and static pages describe target behavior that is not yet implemented. This is acceptable when labeled as target architecture/vision, but risky if used as a current product claim:

- LLM roles and semantic correction;
- exact payload-hash approval binding;
- hardcoded secret detection;
- test deletion/weakening detection;
- universal MCP coverage;
- production credential/authority control;
- full rollback/checkpoint recovery.

## Required Cleanup Before Strong Claims

1. Label `onus/examples/reality_demo.py` as `DEMO_ONLY` or `SIMULATED`.
2. Remove fail-open allow fallbacks from production/integration code paths or make them explicit best-effort modes.
3. Add runtime integration tests for claimed adapters.
4. Add visible claim boundaries in static pages and README content.
5. Ensure demo output is never presented as evidence for missing LLM, MCP, or production containment features.
