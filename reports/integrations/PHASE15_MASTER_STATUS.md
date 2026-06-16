# Phase 15 Integration Master Status

Milestone: Phase 15 complete integration sweep.

Created on: 2026-06-16.

Phase branch: `codex/phase15-integrations`.

Checkpoint tag: `phase15-start-7ea6979`.

Base commit: `7ea6979 feat: prove narrow L4 disposable authority`.

## Repository Contract

This report follows `AGENTS.md` and the Phase 15 instruction file. Locked
documents were read from their current on-disk names:

- `AGENTS.md`
- `docs/Onus_Whitepaper.txt`
- `docs/ONUS_PRODUCT_VISION.md`
- `docs/ONUS_TARGET_ARCHITECTURE.md`
- `docs/ONUS_SECURITY_REQUIREMENTS.md`
- `docs/ONUS_ACCEPTANCE_TESTS.md`
- `docs/ONUS_IMPLEMENTATION_ROADMAP.md`
- `docs/Onus_current_state.md`

No locked document was changed.

## Official Documentation Sources

The integration-control assessment used the current official documentation for
each surface where available:

- Claude Code: <https://code.claude.com/docs/en/hooks>
- Claude Code hooks guide: <https://code.claude.com/docs/en/hooks-guide>
- Windsurf/Cascade MCP: <https://docs.devin.ai/desktop/cascade/mcp>
- Windsurf/Cascade hooks: <https://docs.devin.ai/desktop/cascade/hooks>
- Cline MCP: <https://docs.cline.bot/mcp/mcp-overview>
- VS Code agents: <https://code.visualstudio.com/docs/agents/overview>
- VS Code extension API: <https://code.visualstudio.com/api>
- GitHub Copilot SDK: <https://github.com/github/copilot-sdk>
- GitHub Copilot cloud agent: <https://docs.github.com/en/copilot/concepts/agents/cloud-agent/about-cloud-agent>
- Google Antigravity MCP: <https://antigravity.google/docs/mcp>
- Google Antigravity agent: <https://ai.google.dev/gemini-api/docs/antigravity-agent>
- Cursor docs: <https://cursor.com/docs>
- Cursor cloud agent: <https://cursor.com/docs/cloud-agent>
- OpenAI Codex CLI: <https://developers.openai.com/codex/cli>
- OpenAI Codex CLI reference: <https://developers.openai.com/codex/cli/reference>
- OpenAI Codex MCP: <https://developers.openai.com/codex/mcp>
- Gemini CLI: <https://developers.google.com/gemini-code-assist/docs/gemini-cli>
- Gemini CLI repository: <https://github.com/google-gemini/gemini-cli>
- Continue docs: <https://docs.continue.dev/>
- Continue repository: <https://github.com/continuedev/continue>
- JetBrains Junie: <https://junie.jetbrains.com/docs/>
- JetBrains Junie CLI: <https://junie.jetbrains.com/docs/junie-cli-usage.html>
- JetBrains Junie MCP: <https://junie.jetbrains.com/docs/junie-cli-mcp-configuration.html>
- JetBrains AI Junie IDE agent: <https://www.jetbrains.com/help/ai-assistant/junie-agent.html>
- JetBrains MCP: <https://www.jetbrains.com/help/ai-assistant/mcp.html>
- Aider: <https://aider.chat/>
- Aider usage: <https://aider.chat/docs/usage.html>
- OpenAI Agents SDK: <https://developers.openai.com/api/docs/guides/agents>
- OpenAI Agents SDK Python: <https://openai.github.io/openai-agents-python/agents/>
- LangChain agents: <https://docs.langchain.com/oss/python/langchain/agents>
- CrewAI: <https://docs.crewai.com/en/introduction>

## Local Runtime Inventory

The local runtime inventory was probed before implementation:

| Runtime | Local status | Evidence |
| --- | --- | --- |
| Claude Code | Not installed on `PATH`; prior report used `npx @anthropic-ai/claude-code@2.1.177`, unauthenticated | `reports/current-state/CLAUDE_CODE_L1_RUNTIME.md` |
| Windsurf | Not installed on `PATH` | local command probe |
| Cline | Not installed on `PATH` | local command probe |
| VS Code | Installed | `code --version` returned `1.124.2` |
| Cursor | Not installed on `PATH` | local command probe |
| OpenAI Codex desktop binary | Present as Windows app, but direct version probe failed with access denied | local command probe |
| Gemini CLI | Not installed on `PATH` | local command probe |
| Continue CLI | Not installed on `PATH` | local command probe |
| JetBrains Junie | Not installed on `PATH` | local command probe |
| Aider | Not installed on `PATH` | local command probe |
| Node.js/npm | Installed | Node `v24.15.0`, npm `11.12.1` |
| Python/pip | Installed | Python `3.12.5`, pip `24.2` |

## Existing Integration Code

| Component | Current files | Current claim boundary |
| --- | --- | --- |
| Claude Code hook | `onus/src/cli/claude_hook.rs`, `onus/src/cli/evaluate.rs`, `onus/install/install.ps1`, `onus/install/install.sh` | L1 BEST-EFFORT. Hook translator and process-level probes exist; authenticated Claude Code agent loop is not proven in this environment. |
| MCP gateway | `onus/src/mcp/proxy.rs`, `onus/src/cli/mcp_proxy.rs` | L2 only when traffic is routed through `onus mcp-proxy`. Prior runtime harness exists; direct-server bypass remains documented. |
| VS Code extension | `onus/bindings/vscode/src/extension.js`, `onus/bindings/vscode/package.json` | L1 BEST-EFFORT. VS Code APIs cannot be claimed as mandatory pre-execution containment for every agent tool call. |
| Python Guardian SDK | `onus/bindings/python/src/onus/__init__.py` | L2 for actions routed through Guardian-owned methods. |
| L3 workspace | `onus/src/workspace.rs`, `onus/src/cli/workspace.rs`, `onus/src/cli/run_cmd.rs` | Linux-only L3 claim requires Linux/bubblewrap verifier evidence. |
| L4 authority proof | `onus/src/authority.rs`, `onus/src/cli/authority.rs` | Narrow disposable authority proof only; no production authority claim. |

## Surface Plan

Classification terms:

- `VERIFIED`: runtime-tested through the real integration surface in this repo.
- `VERIFIED WITH LIMITATIONS`: runtime-tested, but claim is bounded by explicit bypass or environment limits.
- `PARTIAL`: meaningful code exists, but end-to-end surface proof is incomplete.
- `PROTOCOL-ONLY`: the safest current integration is through an open protocol such as MCP or SDK wrapping; no native product runtime proof.
- `BLOCKED`: credentials, installation, OS, or closed platform access is unavailable locally.
- `MISSING`: no adapter exists yet.

| Order | Exact surface | Existing state | Work required | Runtime available | Final target |
| ---: | --- | --- | --- | --- | --- |
| 1 | Claude Code CLI | VERIFIED WITH LIMITATIONS in Phase 15. `onus claude-hook` exists and is explicitly L1 BEST-EFFORT. `@anthropic-ai/claude-code@2.1.177` was reachable, but unauthenticated. | No code change required for this surface. Keep BEST-EFFORT label and do not claim live authenticated agent-loop proof. | Package probe passed; authenticated Claude Code is not available. | VERIFIED WITH LIMITATIONS for hook process; BLOCKED for authenticated live agent loop. |
| 2 | Windsurf Editor / Cascade | PROTOCOL-ONLY in Phase 15. No Windsurf-specific native runtime adapter exists. Official docs expose MCP and hooks; this repo now provides a bounded MCP routing template. | Native hook/runtime testing remains blocked until Windsurf is installed. | Not installed locally. | PROTOCOL-ONLY via MCP; BLOCKED for native runtime proof. |
| 3 | Cline | PROTOCOL-ONLY in Phase 15. Cline can route MCP server traffic through `onus mcp-proxy`, but no native Cline runtime adapter is proven. | Native runtime testing remains blocked until Cline is installed. | Not installed locally. | PROTOCOL-ONLY via MCP; no native proof. |
| 4 | Visual Studio Code Agents | VERIFIED WITH LIMITATIONS in Phase 15. VS Code extension exists, VS Code `1.124.2` is installed, and extension JavaScript syntax passes. | Live VS Code agent/Copilot tool-call interception remains unverified. Package JSON has a UTF-8 BOM caveat. | VS Code `1.124.2` installed. | VERIFIED WITH LIMITATIONS for extension checks; L1 BEST-EFFORT. |
| 5 | GitHub Copilot SDK | BLOCKED in Phase 15. `@github/copilot-sdk@1.0.1` is discoverable, but no authenticated Copilot SDK runtime or GitHub CLI is available locally. | Future work requires an Onus-owned SDK/tool-executor wrapper and authenticated runtime tests. | SDK registry reachable; `gh` not installed; credentials unavailable. | BLOCKED. |
| 6 | Google Antigravity | PROTOCOL-ONLY in Phase 15. Official docs expose MCP; repo now includes a bounded Onus MCP routing template. | Native runtime testing remains blocked until Antigravity is installed. | Not installed locally. | PROTOCOL-ONLY via MCP; BLOCKED for native runtime proof. |
| 7 | Cursor CLI | BLOCKED in Phase 15. No local Cursor CLI runtime is available, and VS Code evidence is not reused as Cursor proof. | Future work requires installed Cursor CLI plus native hook/MCP/L3 route verification. | Not installed locally. | BLOCKED. |
| 8 | Cursor Agent in Cursor IDE | BLOCKED in Phase 15. Cursor IDE is not installed, and VS Code extension checks are not reused as Cursor Agent proof. | Future work requires Cursor IDE runtime plus native hook/MCP/L3 route verification. | Cursor not installed locally. | BLOCKED. |
| 9 | Cursor Background Agents | BLOCKED in Phase 15. This is a cloud/service surface and no Cursor cloud runtime or credentials are available locally. | Future work requires service-native hook/policy evidence or L4 authority for privileged side effects. | Credentials/service unavailable. | BLOCKED. |
| 10 | OpenAI Codex CLI | PROTOCOL-ONLY in Phase 15 with local runtime blocker. Official docs expose CLI and MCP; local Windows app binary exists but direct version probe failed with access denied. | Native runtime proof requires executable access. MCP support requires Codex configured to launch `onus mcp-proxy`. | Windows app binary found; access denied on version probe. | PROTOCOL-ONLY via MCP; BLOCKED for native runtime proof. |
| 11 | Gemini CLI | PROTOCOL-ONLY in Phase 15. Official CLI supports MCP; repo now includes a bounded Onus MCP routing template. | Native runtime testing remains blocked until Gemini CLI is installed. | Not installed locally. | PROTOCOL-ONLY via MCP; BLOCKED for native runtime proof. |
| 12 | Continue CLI | MISSING. Continue supports CLI/TUI and tool permissions; MCP-compatible routing may be possible. | Add config/report only after installing or detecting Continue. | Not installed locally. | PROTOCOL-ONLY or BLOCKED. |
| 13 | Continue Agent for VS Code | MISSING as a product-specific adapter. | Use extension configuration or MCP if available; distinguish from generic VS Code extension. | Continue extension not detected. | PROTOCOL-ONLY or BLOCKED. |
| 14 | Continue Agent for JetBrains | MISSING. | Use JetBrains/Continue configuration only with installed runtime. | JetBrains/Continue runtime not detected. | BLOCKED. |
| 15 | JetBrains Junie CLI | MISSING. Official docs show Junie CLI and MCP configuration. | Add MCP route or CLI wrapper only with installed, authenticated Junie. | Not installed locally. | PROTOCOL-ONLY or BLOCKED. |
| 16 | JetBrains Junie IDE Agent | MISSING. Official docs show IDE agent and MCP settings. | Add JetBrains MCP configuration report only with IDE runtime. | JetBrains runtime not detected. | PROTOCOL-ONLY or BLOCKED. |
| 17 | Aider | MISSING as a product adapter. Could be wrapped by L3 workspace or shell policy, but direct file edits outside Onus can bypass L1 hooks. | Add Aider wrapper/report; prefer L3 workspace for meaningful containment. | Not installed locally. | BLOCKED locally; L3 wrapper target on Linux. |
| 18 | OpenAI Agents SDK | MISSING as a framework adapter. Python Guardian can wrap tool functions manually. | Add a small SDK tool wrapper only if the official SDK can be installed/tested; keep provider credential claims separate. | No credentials assumed. Python available. | L2 for wrapped tools; no model-runtime claim without credentials. |
| 19 | LangChain Agents / LangGraph | MISSING as a framework adapter. | Add middleware/tool wrapper around Onus evaluator; test with local no-model tool calls or fixtures. | Python available; packages not confirmed installed. | L2 for wrapped tools if package is available; otherwise BLOCKED. |
| 20 | CrewAI | MISSING as a framework adapter. | Add tool/flow wrapper if package install/runtime is available; test without live model if possible. | Package not confirmed installed. | L2 for wrapped tools if package is available; otherwise BLOCKED. |

## Security Boundaries

- L1 integrations are cooperative and must be labeled `BEST-EFFORT`.
- L2 applies only to actions routed through Onus-owned SDK, gateway, or wrapper code.
- L3 cannot be claimed for any integration until Linux containment tests pass.
- L4 cannot be claimed for any integration except the narrow disposable authority
  proof already implemented and independently verified.
- Direct product runtimes, direct MCP server connections, direct shell/file access,
  and direct cloud-agent side effects remain bypasses unless routed through Onus
  or placed inside a proven L3/L4 boundary.

## Current Phase Status

| Count | Status |
| ---: | --- |
| 2 | Surface adapters newly verified in Phase 15 |
| 5 | Surface adapters added as protocol-only in Phase 15 |
| 4 | Surface adapters blocked with evidence in Phase 15 |
| 10 | Surface adapters merged from integration branches |
| 9 | Surface adapters remaining |

Next required branch: `integration/continue-cli`.

Next exact action: inspect Continue CLI availability and official tool/MCP
control surfaces; classify without claiming untested runtime behavior.
