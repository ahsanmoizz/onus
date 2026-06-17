# IDE Enforcement Gate Analysis

**Date**: 2026-06-17
**Phase**: 15D/15E environment audit
**Purpose**: Analyze L2-or-stronger enforcement for IDE agent actions across all surfaces, documenting the gap between current best-effort (L1) hooks and the pre-execution interception required for real agent safety.

---

## 1. Summary of Enforcement Levels

| Level | Name | Definition | Example |
|-------|------|------------|---------|
| **L1** | BEST-EFFORT cooperative hook | Fires after the action has already started. Cannot pre-emptively block. Can observe, log, warn, and in some cases cancel after initiation but before side-effect propagates. | `vscode.tasks.onDidStartTask`, `onDidChangeTerminalShellIntegration` |
| **L2** | Onus-routed executor/proxy | Onus sits between the agent and the execution target. Tool calls are routed through Onus for evaluation *before* the target receives them. Block/deny decisions prevent the tool call from reaching its target at all. | `onus mcp-proxy`, SDK wrappers (`OnusToolWrapper`) |
| **L3** | Isolated workspace | Process/filesystem/network containment via OS-level sandbox. Agent runs inside an isolated workspace with no access to host resources unless explicitly allowed. | bubblewrap on Linux |
| **L4** | Onus-controlled authority | Credentials and authority are locked behind Onus. Agent cannot act without Onus-issued credentials. Onus revokes authority when policy is violated. | Onus binary itself (owns its own evaluation) |

### Key distinction

L1 observes after the fact. L2 evaluates *before* execution. L3 and L4 prevent execution at the OS and credential level respectively. Only L2+ can provide the "deny before side effect" guarantee that the Onus acceptance requirements demand.

All VS Code extension API hooks are architecturally L1 only.

---

## 2. IDE-by-IDE Enforcement Analysis

### 2.1 VS Code

| Property | Detail |
|----------|--------|
| **Native hook used** | `vscode.tasks.onDidStartTask` (line 248, `extension.js`), `vscode.window.onDidChangeTerminalShellIntegration` (line 156) |
| **Activation event** | `onStartupFinished` (`package.json` line 28) |
| **onDidStartTask equivalent** | `vscode.tasks.onDidStartTask` -- fires **after** VS Code has already begun executing the task. The execution has been dispatched to the terminal/shell by the time Onus sees it. |
| **onDidFinish equivalent** | None implemented. No `onDidEndTask` handler. No completion tracking. |
| **Potential for pre-execution hook** | **None at the agent tool-call level.** VS Code's extension API does not expose a pre-execution hook for arbitrary agent tool calls. The `vscode.tasks` API has `onDidStartTask` (post-hoc) but no `onWillStartTask` that allows cancellation. The `vscode.chat` and `vscode.lm` APIs (introduced in VS Code 1.93+) provide language model tool participation but are cooperative extension points -- not interception hooks. An extension can *provide* tools via `vscode.lm.registerTool()`, but Onus would need to intercept tools that *other* extensions (Copilot, Cline) register, which is not supported. |
| **Current enforcement level** | **L1 -- BEST-EFFORT** |
| **Maximum achievable level** | **L1 -- BEST-EFFORT** (via native extension API alone). L2 is achievable only through non-VS-Code paths: MCP proxy routing, SDK wrappers, or CLI agent hooks. |
| **Gap analysis** | The status bar explicitly reports "Onus: best-effort" (line 298, `extension.js`). The extension tests (5 tests in `extension.test.js`) verify activation, command registration, and configuration -- they do NOT verify agent tool-call interception. Zero tests prove that Copilot, VS Code Chat, or any VS Code agent tool call is routed through Onus. VS Code 1.124.2 does not expose an API to intercept third-party agent tool calls. Microsoft's evolving `vscode.lm` API allows registering *new* tools but not intercepting *existing* ones. The architectural gap is fundamental and cannot be bridged by a VS Code extension alone. |

### 2.2 Google Antigravity

| Property | Detail |
|----------|--------|
| **Native hook used** | Same as VS Code -- Antigravity is a VS Code fork (v1.107.0, confirmed via `product.json` at `D:\Antigravity\resources\app\product.json`). The Onus extension files are deployed to `C:\Users\A\.antigravity\extensions\onus.onus-firewall-0.1.0\`. |
| **Activation event** | Same `onStartupFinished` -- identical extension contract |
| **onDidStartTask equivalent** | Same `vscode.tasks.onDidStartTask` -- fires **after** task execution begins |
| **onDidFinish equivalent** | Not implemented |
| **Potential for pre-execution hook** | Same as VS Code -- Antigravity inherits VS Code's extension API. No pre-execution hook for agent tool calls. Antigravity may add proprietary APIs, but they are not documented in examined control surfaces. |
| **Current enforcement level** | **L1 -- BEST-EFFORT** |
| **Maximum achievable level** | **L1 -- BEST-EFFORT** (via extension API). L2 only via MCP proxy routing if Antigravity supports MCP server configuration. |
| **Gap analysis** | The extension has been file-copied to Antigravity's extension directory but **never loaded or tested**. No runtime verification exists. Even if loaded, the same architectural limitations apply as VS Code. Antigravity does not appear to expose additional agent interception APIs beyond VS Code's standard extension API. |

### 2.3 Devin Desktop / Windsurf

| Property | Detail |
|----------|--------|
| **Native hook used** | Same VS Code extension API. Windsurf (formerly Devin Desktop) is a VS Code fork. Extension deployed to user extensions directory (`C:\Users\A\.windsurf\extensions\` or equivalent). |
| **Activation event** | `onStartupFinished` -- shared VS Code extension contract |
| **onDidStartTask equivalent** | `vscode.tasks.onDidStartTask` -- post-hoc |
| **onDidFinish equivalent** | Not implemented |
| **Potential for pre-execution hook** | Same architectural limit as VS Code. Windsurf may have proprietary agent APIs but they are not documented in examined materials. |
| **Current enforcement level** | **L1 -- BEST-EFFORT** |
| **Maximum achievable level** | **L1 -- BEST-EFFORT** (extension API). L2 only via MCP proxy routing (Windsurf Cascade has documented MCP server support). |
| **Gap analysis** | Confirmed as a VS Code fork rebranded from Devin Desktop to Windsurf. The `product.json` identity was verified in Phase 15C. Extension files are present but never agent-verified. The MCP routing template exists at `integrations/windsurf-cascade/README.md` for agents that support MCP server configuration. |

### 2.4 Cursor IDE

| Property | Detail |
|----------|--------|
| **Native hook used** | Not installed locally. Cursor is a VS Code fork and uses the same extension API. |
| **Activation event** | `onStartupFinished` -- shared VS Code contract |
| **onDidStartTask equivalent** | `vscode.tasks.onDidStartTask` -- post-hoc |
| **onDidFinish equivalent** | Not implemented |
| **Potential for pre-execution hook** | Same VS Code architectural limitation. Cursor has proprietary agent features (Cursor Tab, Cursor Agent) but these do not expose interception hooks through the extension API. |
| **Current enforcement level** | **L1 -- BEST-EFFORT** |
| **Maximum achievable level** | **L1 -- BEST-EFFORT** (extension API). L2 only via MCP proxy routing if MCP servers are configured. |
| **Gap analysis** | Cursor is fully blocked in the current environment -- not installed. The Phase 15D matrix lists it as `BLOCKED -- INSTALL REQUIRED`. Even if installed, the VS Code extension API limitation applies. Cursor's agent mode (Cmd-K agent, background agents) uses its own tool execution paths that are not interceptable through VS Code extension hooks. |

### 2.5 JetBrains (any IDE)

| Property | Detail |
|----------|--------|
| **Native hook used** | **None.** JetBrains is a completely different IDE platform (IntelliJ platform, JVM-based). No Onus extension exists for JetBrains. No extension API has been examined. |
| **Activation event** | N/A |
| **onDidStartTask equivalent** | N/A |
| **onDidFinish equivalent** | N/A |
| **Potential for pre-execution hook** | Unknown -- JetBrains has a plugin SDK that may or may not expose pre-execution hooks. Junie (JetBrains' AI agent) has its own execution model. JetBrains Marketplace supports plugin development in Kotlin/Java. No analysis has been performed. |
| **Current enforcement level** | **L0 -- NO ENFORCEMENT** |
| **Maximum achievable level** | **Unknown.** JetBrains plugin SDK needs analysis. Junie CLI may support MCP routing (template at `integrations/jetbrains-junie-cli/README.md`), which could provide L2 for MCP-routed tools. |
| **Gap analysis** | JetBrains is the most gaping hole in IDE enforcement. No code, no extension, no analysis. The Junie IDE and Continue for JetBrains surfaces are both `BLOCKED -- PLATFORM UNAVAILABLE` in Phase 15D. JetBrains Toolbox and a full JetBrains IDE (~1.5 GB) would need to be installed before any enforcement analysis is possible. MCP routing for Junie CLI is the most promising path, but the CLI itself requires `npm install -g @jetbrains/junie` plus a JetBrains account. |

### 2.6 Cline (within VS Code)

| Property | Detail |
|----------|--------|
| **Native hook used** | None. Cline runs as a VS Code extension inside the same extension host as Onus. Cline does not use the VS Code task system for its tool calls -- it has its own tool execution paths (file read/write, shell commands, MCP, etc.). Onus's `onDidStartTask` hook does NOT fire for Cline's tool calls. |
| **Activation event** | N/A -- Cline activates separately from Onus |
| **onDidStartTask equivalent** | Not applicable -- Cline does not use `vscode.tasks` for agent tool calls |
| **onDidFinish equivalent** | Not applicable |
| **Potential for pre-execution hook** | **None via extension API.** Cline executes tool calls in its own JavaScript context within the VS Code extension host. Onus cannot intercept these calls because they never pass through the VS Code API surfaces that Onus hooks into. The only enforcement path is MCP-based: if Cline uses an MCP server configured through `onus mcp-proxy`, those specific tool calls can be intercepted at L2. Cline's built-in tools (read, write, bash, etc.) and any non-MCP tools cannot be intercepted by Onus. |
| **Current enforcement level** | **L0 -- NO ENFORCEMENT** (for Cline's internal tools) |
| **Maximum achievable level** | **L2 -- but only for MCP-routed tools.** Cline's built-in tools (read, write, bash) are not interceptable by any VS Code extension. They execute directly in the extension host. The MCP routing template at `integrations/cline/README.md` provides L2 for MCP tools explicitly routed through `onus mcp-proxy`. |
| **Gap analysis** | This is a critical gap: Cline is one of the most widely used open-source coding agents, and Onus cannot intercept its core tool calls. Cline's tool execution happens entirely within the extension host process -- the same process Onus runs in. No VS Code API exists for one extension to intercept another extension's tool calls. The only mitigation is configuring Cline's MCP servers to route through Onus, which covers MCP tools but not Cline's built-in file/shell tools. |

### 2.7 Continue (within VS Code)

| Property | Detail |
|----------|--------|
| **Native hook used** | Same as Cline. Continue is a VS Code extension with its own tool execution paths. Onus cannot intercept Continue's tool calls through VS Code extension API hooks. |
| **Activation event** | N/A -- Continue activates separately |
| **onDidStartTask equivalent** | Not applicable -- Continue uses its own tool execution |
| **onDidFinish equivalent** | Not applicable |
| **Potential for pre-execution hook** | Same architectural problem as Cline. Continue's tool calls execute in its own extension host context. MCP routing is the only viable L2 path for MCP-configured tools. |
| **Current enforcement level** | **L0 -- NO ENFORCEMENT** |
| **Maximum achievable level** | **L2 -- MCP tools only** via `onus mcp-proxy`. Continue also has a CLI variant (`npm install -g @continuedev/continue`) with MCP support. |
| **Gap analysis** | Continue is not installed in the current environment (`BLOCKED -- INSTALL REQUIRED`). The same fundamental limitation applies as Cline: Onus runs in the same extension host but cannot intercept another extension's internal tool calls. The Continue for JetBrains surface is also blocked (`BLOCKED -- PLATFORM UNAVAILABLE`). |

---

## 3. MCP Gateway as Enforcement Path

### Architecture

The MCP gateway (`onus/src/mcp/proxy.rs`) runs as a stdio-to-stdio bridge between an MCP client (agent) and an MCP server. For each `tools/call` JSON-RPC message, the proxy:

1. Reads the full request from stdin (standard MCP transport).
2. Parses the tool name and arguments from the `params` field.
3. Normalizes the payload with server identity metadata.
4. Classifies the payload (SHA-256 hashing for approval binding).
5. Builds an `ActionRequest` with session tracking and sequencing.
6. Calls `evaluate_mcp_action()` which routes through the policy engine (guardian, rules, task contracts).
7. Based on verdict:
   - **Allow/Warn**: Forwards the original request to the upstream server, annotates the response with `_onus_receipt`.
   - **Escalate**: Checks for existing approval binding. If approved, forwards; if rejected, returns error; otherwise creates pending approval with timeout.
   - **Block**: Returns a JSON-RPC error with code -32001 ("Blocked by Onus") and the receipt.
8. All responses include `_onus_gateway` metadata and `_onus_receipt` tracking.

### L2 enforcement property

Unlike VS Code extension hooks (L1 -- fires after action starts), the MCP gateway evaluates every `tools/call` **before** forwarding to the upstream MCP server. If the verdict is Block, the upstream server never receives the request. This is a true L2 pre-execution interception.

### Which agents can use it

| Agent | MCP support | L2 achievable via MCP proxy? |
|-------|-------------|-------------------------------|
| Cline | Yes -- configurable MCP servers | Yes -- for MCP-routed tools only |
| Continue | Yes -- MCP server config | Yes -- for MCP-routed tools only |
| Cursor IDE | Yes -- MCP server config | Yes -- for MCP-routed tools only |
| Windsurf/Cascade | Yes -- MCP server config | Yes -- for MCP-routed tools only |
| Google Antigravity | Yes -- MCP server config | Yes -- for MCP-routed tools only |
| VS Code (Copilot) | Partial -- Copilot can use MCP but configuration is limited | Partial -- depends on Copilot MCP support level |
| JetBrains Junie | Yes (CLI) | Yes -- for Junie CLI MCP tools only |
| Claude Code CLI | No native MCP client (uses custom hook protocol) | No -- different protocol |

### Limitation

The MCP gateway only intercepts tool calls that are explicitly routed through it. An agent that connects directly to an upstream MCP server bypasses Onus entirely. The gateway annotates responses with a clear disclaimer:

> `"direct_server_bypass": "Onus protects only MCP traffic routed through this proxy. Direct client connections to the upstream server bypass Onus."`

This is visible in `proxy.rs` line 247.

### Current status

The MCP proxy is implemented and has passing unit tests (approval binding, payload hash verification, task contract persistence). It has NOT been tested with a real agent making live MCP tool calls. The Phase 15D matrix lists all MCP surfaces as `PROTOCOL VERIFIED ONLY` or `BLOCKED`.

---

## 4. L3 Containment as Fallback

### Architecture

The L3 workspace module (`onus/src/workspace.rs`) provides filesystem-level containment through:

1. `create_workspace()` -- copies the repo to an isolated worktree, creates initial checkpoint with file manifest.
2. `run_isolated()` -- executes commands inside a bubblewrap sandbox on Linux with:
   - `--unshare-all` (isolated namespaces for mount, PID, network, IPC, UTS)
   - `--proc /proc`, `--dev /dev` (minimal device nodes)
   - `--tmpfs /tmp --tmpfs /run` (ephemeral temp storage)
   - `--ro-bind /original` (read-only reference to original repo)
   - `--bind /workspace` (writable isolated worktree)
   - `--clearenv` with filtered environment (only PATH, HOME, TMPDIR, ONUS_* vars)
   - `--die-with-parent --new-session` (cleanup on parent exit)
   - Optional `--share-net` (network policy controlled by metadata)
   - Resource limits via `setrlimit` (CPU, memory, processes, file handles)
3. `inspect_workspace()` / `export_workspace()` / `destroy_workspace()` -- lifecycle management.
4. Checkpoint/revert with SHA-256 manifest tracking.

### L3 enforcement property

L3 is the only enforcement level that can contain agents whose tool calls cannot be intercepted at L2 (e.g., Cline's built-in tools, Continue's internal tools, any agent executing arbitrary code in the extension host). By running the entire agent inside a bubblewrap sandbox, all filesystem writes, process creation, and network access are contained regardless of whether the individual tool calls are intercepted.

### Linux-only constraint

L3 is **Linux-only**. The `require_linux_l3_available()` function (line 278 of `workspace.rs`) explicitly fails closed on non-Linux platforms:

```rust
#[cfg(not(target_os = "linux"))]
{
    anyhow::bail!(
        "L3 workspace isolation is currently implemented only on Linux; refusing to run without a real L3 boundary"
    );
}
```

On Windows, there is no `bubblewrap` equivalent. The Rust test `test_run_isolate_fails_closed_without_linux_boundary` confirms this behavior: the function returns an error rather than silently running uncontained.

### Gap for IDE agents

Even on Linux, L3 containment has a fundamental limitation for IDE agents: the agent runs inside VS Code / JetBrains / Cursor (a GUI application that cannot easily be sandboxed with bubblewrap). L3 containment is practical for CLI agents (Claude Code CLI, Aider, Gemini CLI) but not for IDE-integrated agents. To contain an IDE agent at L3, you would need to sandbox the entire IDE process, which is impractical for normal development workflows.

### Current status

L3 has passing tests (workspace creation/manifest/inspection/destruction, fail-closed on non-Linux). No L3 test has been performed with a real agent. The current environment is Windows, where L3 is unavailable.

---

## 5. Recommendation for v1

### Enforcement strategy by surface

| Surface | v1 enforcement path | Level | Priority | Rationale |
|---------|-------------------|-------|----------|-----------|
| **VS Code (native extension)** | L1 best-effort hooks (terminal + task post-hoc observation) | L1 | High | Best available via extension API. Document as L1 only. |
| **VS Code (Copilot/Chat)** | No direct interception. v1: document as not enforceable. | L0 | Medium | No API exists. Monitor VS Code `vscode.lm` API evolution. |
| **Cline (built-in tools)** | No direct interception. v1: document as not enforceable via VS Code extension. | L0 | High | Cline's internal tools cannot be intercepted by any extension. |
| **Cline (MCP tools)** | MCP gateway (`onus mcp-proxy`) configured as Cline's MCP server | L2 | High | Covers MCP-routed tools. Template documented in `integrations/cline/README.md`. |
| **Continue (VS Code)** | MCP gateway for MCP tools. No interception for internal tools. | L2 (MCP only) | Medium | Same pattern as Cline. |
| **Google Antigravity** | L1 extension hooks + MCP gateway for MCP tools | L1 / L2 (MCP) | Low | Extension not agent-verified. MCP template exists. |
| **Windsurf/Cascade** | MCP gateway for MCP tools. Extension for L1 best-effort. | L1 / L2 (MCP) | Low | Not agent-verified. MCP template exists. |
| **Cursor IDE** | MCP gateway for MCP tools. Extension for L1 best-effort. | L1 / L2 (MCP) | Low | Not installed. VS Code fork limitation applies. |
| **JetBrains (any)** | No enforcement. v1: document as unsupported. Explore JetBrains plugin SDK. | L0 | Low | No code exists. Requires full JetBrains plugin development. |
| **Claude Code CLI** | Hook protocol (stdin/stdout JSON) | L2 (hook) | High | Implemented at `onus/src/cli/claude_hook.rs`. Only tested via Python subprocess, not live agent. |
| **Gemini CLI** | MCP gateway | L2 (MCP) | Low | Not installed. Template exists. |
| **OpenAI Agents SDK** | SDK wrapper (`OnusToolWrapper` + `OnusClient.evaluate()`) | L2 (advisory) | High | Passing tests (16/16) including bypass + approval binding. Advisory: bypassable via direct `.func()` call. |
| **LangChain/LangGraph** | SDK wrapper (`OnusToolWrapper` + `@tool` decorator) | L2 (advisory) | High | Passing tests (21/21) including bypass + approval binding. Advisory: bypassable via `.func()`/`.invoke()`. |
| **CrewAI** | No adapter exists | L0 | Low | Not installed. No code. |
| **Aider** | Hooks or MCP depending on Aider's protocol | TBD | Low | Not installed. No analysis performed. |
| **GitHub Copilot SDK** | No adapter exists | L0 | Low | No code. No credentials. |

### Summary: v1 enforcement portfolio

```
L1 (best-effort hooks):
  VS Code extension (terminal + task)
  Antigravity extension (same hooks, not agent-tested)
  Windsurf extension (same hooks, not agent-tested)

L2 (pre-execution evaluation):
  MCP gateway -- for any agent using MCP through onus mcp-proxy:
    Cline (MCP tools)
    Continue (MCP tools)
    Cursor (MCP tools)
    Windsurf (MCP tools)
    Antigravity (MCP tools)
    Gemini CLI (MCP tools)
    Junie CLI (MCP tools)
  Hook protocol:
    Claude Code CLI (stdin/stdout hook)
  SDK wrappers (advisory):
    OpenAI Agents SDK (OnusToolWrapper)
    LangChain/LangGraph (OnusToolWrapper)

L3 (container isolation, Linux only):
  CLI agents running in bubblewrap workspaces (not IDE agents)

L4 (credential control):
  Onus binary itself (evaluate path)
```

### Critical gaps to document for v1

1. **VS Code / Cline / Cannot-intercept gap**: The most widely used coding agents (Cline, Continue, VS Code Copilot) execute tool calls in the extension host where Onus cannot intercept them. VS Code's extension API does not provide pre-execution hooks for agent tool calls. This is an architectural limitation of the VS Code extension platform, not a missing Onus feature.

2. **Windows L3 gap**: L3 isolated workspace is Linux-only. On Windows, there is no equivalent of bubblewrap. Development environments on Windows cannot use workspace-level containment. WSL2 could potentially provide L3 on Windows, but zero WSL distros are installed and no code supports this path.

3. **JetBrains blind spot**: No analysis has been performed on JetBrains' plugin SDK. The JetBrains platform (IntelliJ IDEA, PyCharm, WebStorm) is a completely different IDE ecosystem with no Onus code. Junie and Continue for JetBrains are entirely unexamined.

4. **SDK wrappers are advisory**: The OpenAI Agents SDK and LangChain wrappers evaluate tool calls but can be bypassed by calling the underlying function directly. The bypass tests confirm this. These wrappers provide L2-style evaluation but without mandatory enforcement.

5. **MCP gateway is opt-in**: The MCP gateway only protects agents that are explicitly configured to route through it. An agent connecting directly to an upstream MCP server bypasses Onus. The gateway documentation clearly labels this limitation.

### Practical v1 enforcement priority

```
1. MCP gateway (highest impact, least effort)
   - Already implemented and tested (unit level)
   - Supports multiple agents through a single proxy
   - True L2 pre-execution enforcement
   - Needs live agent test (any agent with MCP support)

2. Claude Code CLI hook (high impact for Anthropic users)
   - Hook protocol implemented and tested
   - Needs: ANTHROPIC_API_KEY + live agent loop test
   - L2 via stdin/stdout evaluation

3. SDK wrappers (for Python agent frameworks)
   - Implemented, tested, with bypass + approval binding tests
   - Advisory but better than nothing
   - Document bypass surface clearly

4. L3 workspace (Linux only, CLI agents only)
   - Implemented and unit-tested
   - Not applicable to IDE agents
   - Cannot be proven on Windows

5. Native IDE extension hooks (L1 only, accept the limitation)
   - Best-effort observation and warning
   - Status bar already says "best-effort"
   - Document as L1 only -- do not claim pre-execution enforcement
```

### What to NOT claim in v1

- Onus does NOT block destructive actions from Cline, Continue, or VS Code Copilot agents.
- Onus does NOT provide L2+ enforcement for any GUI IDE agent through native extension hooks.
- Onus does NOT provide L3 containment on Windows or for IDE agents on any platform.
- Onus does NOT intercept JetBrains agent actions.
- Onus SDK wrappers do NOT prevent direct invocation of underlying tool functions.
- Onus MCP gateway does NOT protect agents that connect to MCP servers directly.

### What CAN be claimed in v1

- Onus provides L2 pre-execution evaluation for MCP tool calls explicitly routed through `onus mcp-proxy`.
- Onus provides L2 evaluation for Claude Code CLI actions routed through the hook protocol.
- Onus provides L2-wrapper evaluation for OpenAI Agents SDK and LangChain/LangGraph tool calls through Python SDK wrappers.
- Onus provides L3 workspace isolation for CLI agents on Linux (bubblewrap).
- Onus provides L1 best-effort observation and warning for VS Code, Antigravity, and Windsurf terminal/task actions through native extension hooks.
- All levels are documented with their precise enforcement boundaries and known bypass surfaces.

---

*Report produced: 2026-06-17*
*Sources: Phase 15D live product matrix, Phase 15D final compatibility report, Phase 15E environment inventory, VS Code extension source (`extension.js`, `package.json`), MCP proxy implementation (`proxy.rs`), L3 workspace module (`workspace.rs`), integration templates (`integrations/*/README.md`), and independent Phase 15C verification report.*
