# Onus — AI Agent Firewall
## Technical Specification v0.2

**Real-time interception, evaluation, and correction for AI agent actions.**
Prevents agents from destroying what they were asked to build.

---

## 1. Core Architecture Principle

**Onus runs in the critical path between agent decision and agent action.**

It is NOT an observability tool that records after the fact. It is an interception engine that evaluates BEFORE the file saves, BEFORE the command runs, BEFORE the API call fires. The agent doesn't know Onus exists — Onus sits between the agent and the world, silent until something needs blocking.

```
AGENT                    ONUS                        WORLD
  │                        │                           │
  │  "I'll delete ./src"   │                           │
  ├───────────────────────►│                           │
  │                        │  Policy eval (sub-5ms)    │
  │                        │  Scope check              │
  │                        │  Safety check             │
  │                        │                           │
  │   ┌─ ALLOW ────────────┼──────────────────────────►│
  │   │                    │  Action executes          │
  │   │                    │  Logged to audit trail    │
  │   │                    │                           │
  │   └─ BLOCK ────────────┤                           │
  │     ◄──────────────────┤  Blocked + correction     │
  │     "Task was X,       │                           │
  │      you tried Y.      │                           │
  │      Re-evaluate."     │                           │
  │                        │                           │
  │   ┌─ ESCALATE ─────────┤                           │
  │   │                    │  Slack/Discord notify     │
  │   │                    │  Human decides            │
```

---

## 2. System Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    DEVELOPER MACHINE (local)                  │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                    ONUS CORE                         │   │
│  │                 (Rust binary, ~5MB)                  │   │
│  │                                                      │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐             │   │
│  │  │ Policy   │ │ Scope    │ │ Audit    │             │   │
│  │  │ Engine   │ │ Tracker  │ │ Trail    │             │   │
│  │  │          │ │          │ │          │             │   │
│  │  │ Rules    │ │ Task     │ │ SQLite   │             │   │
│  │  │ DSL eval │ │ boundary │ │ local    │             │   │
│  │  │ <5ms     │ │ tracking │ │ store    │             │   │
│  │  └──────────┘ └──────────┘ └──────────┘             │   │
│  │                                                      │   │
│  │  ┌──────────────────────────────────────────┐       │   │
│  │  │              IPC Interface                │       │   │
│  │  │         (Unix socket / named pipe)        │       │   │
│  │  └──────────────────────────────────────────┘       │   │
│  └────────┬─────────────┬─────────────┬────────────────┘   │
│           │             │             │                     │
│     ┌─────┘       ┌─────┘       ┌─────┘                     │
│     ▼             ▼             ▼                           │
│  ┌──────┐   ┌──────────┐  ┌──────────┐                     │
│  │Claude│   │ VS Code  │  │  Python  │                     │
│  │ Code │   │Extension │  │   SDK    │                     │
│  │ Hook │   │          │  │          │                     │
│  └──────┘   └──────────┘  └──────────┘                     │
│     ▼             ▼             ▼                           │
│  ┌──────┐   ┌──────────┐  ┌──────────┐                     │
│  │Shell │   │   MCP    │  │  Agent   │                     │
│  │Wrap  │   │  Proxy   │  │Frameworks│                     │
│  └──────┘   └──────────┘  └──────────┘                     │
│                                                              │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           │ Optional: sync to cloud
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                  ONUS CONTROL PLANE (Phase 3, SaaS)           │
│                                                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │
│  │ Org      │ │ Cross-   │ │ Incident │ │ Compliance   │   │
│  │ Policy   │ │ Deploy   │ │ History  │ │ Reports      │   │
│  │ Manager  │ │ Learning │ │          │ │              │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Tenant event store                       │   │
│  │         (anonymized, opt-in, SOC 2)                   │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

**Key architectural decisions:**

1. **Local-first.** Onus Core runs on the developer's machine. No cloud dependency. No API keys. No data leaves the machine unless the org opts into cloud sync.

2. **IPC over HTTP.** Integrations talk to Onus Core via Unix socket (Unix) or named pipe (Windows). No network overhead. Sub-millisecond communication.

3. **Rust for the core.** Single binary, no runtime dependency, sub-5ms policy evaluation. Python/Node integrations are thin clients that serialize the action, send it over IPC, and receive a verdict.

4. **SQLite for audit.** Local, zero-config, fast enough for single-machine write load. Cloud sync uses change-data-capture from SQLite WAL.

---

## 3. Integration Surfaces (Detailed)

### 3.1 Claude Code Hook (Phase 1 primary)

**Mechanism:** Claude Code's `settings.json` supports a `hooks` stanza with `preToolUse` — fires before every tool execution, receives tool name + input, can block or modify.

```json
{
  "hooks": {
    "preToolUse": {
      "command": "onus evaluate",
      "timeout": 5000
    }
  }
}
```

**Data flow:**
1. Claude Code is about to execute `Bash(command="rm -rf ./src")`
2. `preToolUse` hook fires, passes JSON to `onus evaluate`:
   ```json
   {
     "tool": "Bash",
     "input": { "command": "rm -rf ./src" },
     "session_id": "abc123",
     "cwd": "/Users/alice/project",
     "agent": "claude-code",
     "agent_version": "1.2.3"
   }
   ```
3. Onus Core evaluates: safety rule #1 (destructive command) matches → BLOCK
4. Onus returns to Claude Code:
   ```json
   {
     "decision": "block",
     "reason": "Safety rule: destructive filesystem operation outside allowed scope",
     "correction": "Task was 'add unit tests to auth.ts'. Deleting ./src is not required. Re-evaluate your approach.",
     "rule_id": "SAFETY_001"
   }
   ```
5. Claude Code's `preToolUse` receives exit code 2 (block), reads stderr. Claude Code displays the correction to the agent, agent re-plans.

**Validation that this works:** Claude Code's hook system passes `ONUS_DECISION` via stdout. Exit code 0 = allow, exit code 2 = block, exit code 3 = escalate. The hook output is shown to the agent model as context, so the correction message directly influences the agent's next reasoning step.

### 3.2 VS Code Extension (Phase 2)

**Mechanism:** VS Code extension API intercepts at three points:

| Intercept point | API | What it catches |
|-----------------|-----|-----------------|
| File save | `vscode.workspace.onWillSaveTextDocument` | Agent writing/deleting files in the workspace |
| Terminal command | `vscode.window.onDidOpenTerminal` + shell integration | Agent running shell commands |
| File create/delete | `vscode.workspace.onDidCreateFiles` / `onDidDeleteFiles` | Agent file system mutations |

**Architecture:**
- Extension spawns Onus Core as a subprocess (or connects to running instance)
- On file save: captures file path + diff → sends to Onus Core → verdict returned before save completes
- On terminal: wraps shell with onus-shell-proxy → each command passes through eval
- Extension communicates with VS Code agent extensions (Copilot, Cursor, Windsurf) via workspace events — agent extensions don't need to explicitly integrate

**Risk:** `onWillSaveTextDocument` can only delay, not fully prevent edits that happen in-memory. Mitigation: also watch `onDidChangeTextDocument` for large diffs, but primary blocking is at save time.

### 3.3 Python SDK (Phase 2)

```python
from onus import guard, AuditContext

@guard
def write_file(path: str, content: str) -> None:
    """Agent calls this. Onus evaluates before execution."""
    with open(path, 'w') as f:
        f.write(content)

# Or as a context manager for framework integration:
with AuditContext(task="Add unit tests to auth.ts"):
    agent.run()  # All tool calls within this context are intercepted
```

**Mechanism:**
- `@guard` is a decorator that serializes the function call, sends it over IPC to Onus Core, and either proceeds or raises `OnusBlockError`.
- On `OnusBlockError`, the correction message is included in the exception — the agent framework can catch it and feed it back as context.
- Framework-specific adapters (LangChain callback, OpenAI SDK middleware, CrewAI tool wrapper) use the same IPC client internally.
- `AuditContext` sets the task boundary for scope tracking — Onus Core now knows "this agent session is about auth.ts, file writes to unrelated files are suspicious."

### 3.4 MCP Proxy (Phase 2)

**Mechanism:** Onus runs an MCP server that wraps real MCP servers:

```
Agent (MCP client)
    │
    ▼
Onus MCP Proxy (evaluates)
    │
    ▼
Real MCP server (filesystem, database, API)
```

Every `tool_call` from the agent passes through Onus before reaching the real tool. Onus adds an `X-Onus-Verdict` header pattern. On block, the proxy returns an MCP error with the correction message — standard MCP error handling surfaces this to the agent.

**Why this is high-leverage:** Any MCP-speaking agent (Claude Code, Continue.dev, Zed, Cody, custom agents) is automatically covered. No per-agent integration needed. The MCP standard is becoming the universal agent-tool interface.

### 3.5 Shell Wrapper (Phase 2)

**Mechanism:** `onus-shell` wraps bash/zsh:

```bash
# Instead of:
agent → /bin/bash -c "rm -rf ./src"

# Agent runs:
agent → onus-shell -- /bin/bash -c "rm -rf ./src"
```

`onus-shell` parses the command, sends it to Onus Core for evaluation, forwards to real shell on allow, blocks on deny. This covers terminal-based agents (Devin, Aider, OpenHands) that spawn shell processes directly.

---

## 4. Onus Core Internals

### 4.1 Policy Engine

**Rules DSL (TOML, versioned):**

```toml
# safety_rules.toml
[[rule]]
id = "SAFETY_001"
name = "destructive-filesystem-command"
description = "Blocks rm -rf, format, and other destructive filesystem operations outside allowed scope"
tier = 1
action_type = "shell"
pattern = '(rm\s+-rf|dd\s+if=|mkfs\.|:\(\)\s*\{\s*:)'
decision = "block"
correction = "This command is destructive. Confirm you need it for the stated task."
allowlist_paths = ["node_modules", ".cache", "__pycache__", "target"]  # safe deletes

[[rule]]
id = "SAFETY_002"
name = "sudo-execution"
description = "Blocks any sudo invocation"
tier = 1
action_type = "shell"
pattern = 'sudo\s+'
decision = "escalate"
escalation_message = "Agent attempted sudo. Human approval required."

[[rule]]
id = "SCOPE_001"
name = "write-outside-workspace"
description = "Blocks file writes outside the project working directory"
tier = 1
action_type = "file_write"
decision = "block"
scope_check = "workspace_root"
correction = "You attempted to write outside the project directory. Restrict changes to the task scope."

[[rule]]
id = "MAGNITUDE_001"
name = "large-diff-warning"
description = "Flags file changes that are disproportionately large for the task"
tier = 2
action_type = "file_write"
decision = "warn"  # warn = log + surface in IDE, but allow
heuristic = "diff_lines > 200 AND task_estimated_lines < 50"
```

**Policy evaluation pipeline (sub-5ms target):**

```
Action arrives via IPC
    │
    ▼
1. Parse action type + payload (string match, no regex yet)     ~0.1ms
    │
    ▼
2. Check deterministic rules (Tier 1) — exact or regex match   ~0.5ms
    │  First BLOCK hit → short-circuit, return immediately
    │
    ▼
3. If no block: check scope rules                               ~0.5ms
    │  Does file path fall within workspace root?
    │  Is this a known safe pattern?
    │
    ▼
4. If no block: apply heuristics (Tier 2)                       ~1-2ms
    │  Diff magnitude vs task estimate
    │  Goal drift detection (are we editing unexpected files?)
    │
    ▼
5. Log to audit trail                                            ~0.5ms
    │
    ▼
6. Return verdict                                               ~0.1ms

Total: ~2-5ms
```

Optimization: rules are pre-compiled into a DFA at startup. The fast-path (ALLOW) for common actions (edit a file, run `npm test`) hits stages 1-2 and returns in under 1ms.

### 4.2 Scope Tracker

The scope tracker maintains a model of "what the agent is supposed to be doing right now" so it can detect drift.

```rust
struct SessionScope {
    task_description: String,
    declared_files: Vec<String>,      // files the agent said it would touch
    allowed_paths: Vec<String>,       // expanded from .gitignore + declared
    estimated_complexity: Complexity, // "small" | "medium" | "large"
    touched_files: Vec<String>,       // files actually touched so far
    retry_count: u32,
}
```

**Scope is set once at session start** (from the task prompt) and updated as the agent declares intent. If the agent starts editing `auth.ts` when the task said "add tests to payment.ts," the scope tracker flags goal drift at Tier 2.

### 4.3 Audit Trail

Schema (SQLite, local):

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    agent_name TEXT NOT NULL,
    agent_version TEXT,
    task_description TEXT NOT NULL,
    workspace_root TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    total_actions INTEGER DEFAULT 0,
    blocked_actions INTEGER DEFAULT 0,
    escalated_actions INTEGER DEFAULT 0
);

CREATE TABLE actions (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    sequence_number INTEGER NOT NULL,  -- monotonic per session
    action_type TEXT NOT NULL,          -- "shell", "file_write", "file_delete", "api_call"
    tool_name TEXT,                     -- "Bash", "Write", "Edit", etc.
    payload TEXT NOT NULL,              -- the full action (command, file path, diff, etc.)
    before_state TEXT,                  -- snapshot before mutation (nullable)
    after_state TEXT,                   -- snapshot after mutation (nullable, only on ALLOW)
    verdict TEXT NOT NULL,              -- "allow", "block", "warn", "escalate"
    rule_id TEXT,                       -- which rule triggered, if any
    correction TEXT,                    -- correction prompt sent to agent
    latency_us INTEGER,                 -- eval time in microseconds
    timestamp INTEGER NOT NULL,
    -- Tamper evidence
    prev_hash TEXT,                     -- SHA-256 of previous action in session
    hash TEXT NOT NULL,                 -- SHA-256 of this action
    merkle_root TEXT                    -- filled on session close
);

CREATE TABLE merkle_roots (
    session_id TEXT PRIMARY KEY REFERENCES sessions(id),
    root_hash TEXT NOT NULL,
    action_count INTEGER NOT NULL,
    computed_at INTEGER NOT NULL
);
```

**Hash chain:** Each action includes `prev_hash` (hash of the previous action in the session). On session close, a Merkle tree is built over all actions, and the root hash is stored. This provides tamper-evidence: any modification to any action in the chain will break the root hash.

**External anchoring (Phase 3):** Merkle roots can be published to a public ledger (Ethereum, Certificate Transparency, or a simple timestamp authority) for verifiable third-party audit.

### 4.4 IPC Protocol

Onus Core listens on a Unix domain socket (Linux/macOS) or named pipe (Windows).

**Protocol:** length-prefixed JSON over the socket. Simple enough to implement in any language in <100 lines.

```
[4-byte BE length][JSON payload]
```

**Request:**
```json
{
  "version": 1,
  "session_id": "abc123",
  "sequence": 42,
  "action": {
    "type": "shell",
    "tool": "Bash",
    "payload": {
      "command": "rm -rf ./src",
      "cwd": "/Users/alice/project"
    }
  }
}
```

**Response:**
```json
{
  "version": 1,
  "session_id": "abc123",
  "sequence": 42,
  "decision": "block",
  "rule_id": "SAFETY_001",
  "rule_name": "destructive-filesystem-command",
  "correction": "This command is destructive. Task was 'add tests to auth.ts'. Re-evaluate.",
  "latency_us": 3400
}
```

**Decisions:**
| Exit code | Decision | Meaning |
|-----------|----------|---------|
| 0 | `allow` | Proceed. Logged silently. |
| 1 | `warn` | Proceed, but surface warning in IDE/terminal. |
| 2 | `block` | Halt. Return correction to agent. |
| 3 | `escalate` | Halt. Require human approval. |

---

## 5. Tech Stack

### 5.1 Onus Core (local binary)

| Component | Technology | Why |
|-----------|-----------|-----|
| Language | **Rust** | Single binary, no runtime, sub-ms eval, cross-platform |
| Policy engine | Custom DFA compiled from TOML rules | Zero-allocation fast path for common actions |
| IPC | Unix domain sockets / named pipes | No network overhead, no port conflicts |
| Audit store | SQLite (via `rusqlite`) | Zero-config, embedded, ACID, WAL mode |
| Rule parser | `toml` crate | Battle-tested, zero-copy where possible |
| Regex | `regex` crate | Pre-compiled, SIMD-accelerated |
| Hashing | `sha2` crate | SHA-256 for action chaining |
| Merkle tree | `rs_merkle` | For session-level tamper evidence |
| CLI | `clap` | Installer, `onus evaluate`, `onus log`, `onus status` |
| Cross-compile | `cross` / GitHub Actions | macOS (ARM+x86), Linux (ARM+x86), Windows |

### 5.2 Integration Shells (thin clients)

| Surface | Language | Size | Dependency |
|---------|----------|------|-----------|
| Claude Code hook | Bash + JSON (generated by installer) | ~10 lines | None — uses Claude's native hook system |
| VS Code extension | TypeScript | ~500 lines | VS Code Extension API |
| Python SDK | Python | ~300 lines | No dependencies beyond stdlib IPC |
| MCP proxy | Rust (reuses Onus Core) | ~200 lines | MCP SDK |
| Shell wrapper | Rust (reuses Onus Core) | ~150 lines | None |

### 5.3 Control Plane (Phase 3, SaaS)

| Component | Technology | Why |
|-----------|-----------|-----|
| API Gateway | **Go** | Raw throughput for event ingestion |
| Event Store | **ClickHouse** | Append-only, columnar, analytical queries at scale |
| Transactional DB | **PostgreSQL** | Tenants, orgs, policies, billing |
| Stream | **Redpanda** / Kafka | Ingestion buffer, anomaly pipeline |
| Eval Workers | **Python** (Celery/Temporal) | ML ecosystem (scikit-learn, transformers) |
| Workflow Engine | **Temporal** | Rollback orchestration MUST be durable |
| Dashboard | **Next.js + TypeScript + Tailwind** | Trace viewer, policy editor, org admin |
| Object Store | **S3-compatible** | Snapshots, diffs, large payloads |
| Auth | OIDC/SAML SSO, SCIM, API keys | Enterprise SSO is table stakes |

### 5.4 Infrastructure & DevOps

| Area | Choice |
|------|--------|
| Container orchestration | Kubernetes (SaaS) |
| IaC | Terraform |
| CI/CD | GitHub Actions |
| Observability | Grafana stack (we cannot be the unreliable reliability company) |
| Compliance | SOC 2 Type I by month 9, Type II by month 18 |

---

## 6. CLI & Developer Experience

```bash
# Install (Phase 1 target: one command)
npx @onus/install
# → Downloads Onus Core binary for platform
# → Wires Claude Code preToolUse hook into settings.json
# → Creates default safety_rules.toml
# → Starts Onus Core daemon (background, auto-restart)

# Check status
onus status
# → "Onus v0.1.0 running. 3 agents connected. 1,247 actions evaluated. 3 blocked. 0 escalated."

# View audit trail
onus log
# → Interactive terminal UI showing recent actions with color-coded verdicts

# View session summary
onus session abc123
# → "Session abc123: Claude Code. Task: Add tests to auth.ts. 42 actions, 1 blocked, 0 escalated."

# Update rules
onus rules pull   # Fetch latest community rules
onus rules edit   # Open safety_rules.toml in $EDITOR
onus rules test   # Dry-run rules against recent actions

# Upgrade
onus upgrade
# → Downloads latest binary, restarts daemon

# Uninstall
onus uninstall
# → Removes hooks, stops daemon, keeps audit trail (explicit --purge to delete)
```

---

## 7. Data Model

```
Session
├── id: UUID
├── agent_name: string (claude-code, cursor, copilot, etc.)
├── agent_version: string
├── task_description: string (the original prompt)
├── workspace_root: path
├── status: enum { active, completed, escalated, aborted }
├── started_at: timestamp
├── ended_at: timestamp?
├── action_count: uint32
├── blocked_count: uint32
└── escalated_count: uint32

Action
├── id: UUID
├── session_id: FK → Session
├── sequence: uint32 (monotonic, per-session)
├── type: enum { shell, file_write, file_delete, file_read, api_call, network }
├── tool: string (Bash, Write, Edit, WebFetch, etc.)
├── payload: JSON (tool-specific — command string, file path + diff, URL + method, etc.)
├── before_state: JSON? (for file writes: original content; for API calls: pre-request state)
├── after_state: JSON? (for file writes: new content; null if blocked)
├── verdict: enum { allow, warn, block, escalate }
├── rule_id: string? (which rule fired)
├── correction: string? (correction prompt, if blocked)
├── reversibility: enum { reversible, compensable, irreversible } (Phase 3)
├── eval_latency_us: uint32
├── prev_hash: SHA-256?
├── hash: SHA-256
└── timestamp: uint64

Policy
├── id: string (SAFETY_001, SCOPE_001, etc.)
├── name: string
├── tier: uint8 (1, 2, 3)
├── action_type: string
├── pattern: string (regex or glob)
├── decision: enum { allow, warn, block, escalate }
├── correction_template: string
├── allowlist: string[] (paths, commands, patterns)
└── enabled: bool

OrgPolicy (Phase 3, cloud)
├── id: UUID
├── org_id: FK → Org
├── rule_id: string (references a known rule)
├── override_decision: enum? (org can make a block into an escalate, etc.)
├── allowlist_additions: string[]
└── enabled: bool
```

---

## 8. Build Sequence

### Phase 1 — Wedge (Weeks 1–2)

**Deliverable:** Onus Core binary + Claude Code hook + 10-15 safety rules. Open source. One-command install.

| Week | What | Exit criteria |
|------|------|--------------|
| **Week 1** | Onus Core skeleton: IPC listener, policy engine, SQLite audit trail, `onus evaluate` CLI | Binary compiles, passes unit tests, eval <5ms |
| **Week 1** | Rule parser + 10-15 safety rules (Tier 1) | All rules have tests. `onus rules test` validates against sample actions |
| **Week 2** | Claude Code hook integration: installer wires `preToolUse`, end-to-end test with real Claude Code | A `rm -rf` command is blocked. Correction message appears in Claude Code. Allowed actions proceed silently. |
| **Week 2** | CLI: install, status, log, uninstall | `npx @onus/install` works on macOS + Linux. Wired into existing Claude Code settings. |
| **Week 2** | Release v0.1.0 to GitHub. Blog post + HN launch. | 100 installs. Real-world block data starts flowing. |

### Phase 2 — Coverage (Months 1–3)

| Month | What |
|-------|------|
| **Month 1** | VS Code extension: file save + terminal interception. Ship to VS Code marketplace. |
| **Month 2** | Python SDK + LangChain/OpenAI SDK adapters. MCP proxy. |
| **Month 3** | Shell wrapper. Tier 2 heuristic rules (magnitude, goal drift). GitHub PR integration (comments on agent-authored PRs: "Onus blocked 3 actions, corrected 2"). |

### Phase 3 — Control Plane (Months 4–9)

| Month | What |
|-------|------|
| **Months 4–5** | Cloud ingestion pipeline (ClickHouse + Redpanda). Org dashboard. Opt-in event sync from local Core. |
| **Months 6–7** | Cross-deployment learning (Tier 3). Policy management UI. SOC 2 Type I audit. |
| **Months 8–9** | Rollback orchestration (Temporal). Approval gates. Self-hosted enterprise tier. |

### Phase 4 — Platform (Months 10–18)

| Milestone | What |
|-----------|------|
| **Pre-approval gates** | Policy-defined rules requiring human sign-off before irreversible actions |
| **Blast radius mapping** | Visualize everything an agent touched: files, APIs, databases |
| **Compensation infrastructure** | Auto-generated rollback plans with execution tracking |
| **SOC 2 Type II** | Continuous compliance monitoring |
| **Self-hosted GA** | Enterprise deployment in customer VPC |

---

## 9. Safety Rules — v0 Reference Set

These 13 rules ship in Phase 1. Covers 90%+ of dangerous agent actions observed in production.

```
SAFETY_001  — Destructive filesystem: rm -rf, dd, mkfs, fork bomb
SAFETY_002  — Sudo execution: any sudo invocation
SAFETY_003  — Curl-to-shell: curl ... | bash, wget ... | sh
SAFETY_004  — Environment exfiltration: env, printenv, cat .env, echo $SECRET
SAFETY_005  — Permission escalation: chmod 777, chown to root
SAFETY_006  — Git force push: git push --force, git push --delete
SAFETY_007  — Database drop/truncate: DROP TABLE, DROP DATABASE, TRUNCATE
SAFETY_008  — Network bind: opening ports, listening on 0.0.0.0
SCOPE_001   — Write outside workspace: file write path not under workspace_root
SCOPE_002   — Delete outside workspace: file delete path not under workspace_root
SCOPE_003   — Read outside workspace: file read of sensitive paths (/etc/passwd, ~/.ssh)
SCOPE_004   — Git destructive: git reset --hard, git clean -fd, git checkout -- .
MAGNITUDE_001 — Large diff: file edit changes >500 lines (warn, don't block)
```

---

## 10. Hard Technical Risks (honest)

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Interception is bypassable.** Agents can spawn subprocesses that evade the hook. | High | Defense in depth: shell wrapper catches subprocesses, VS Code watcher catches file mutations even if bypassed. Coverage across surfaces means the agent must evade all of them. |
| **Latency is user-visible on BLOCK.** The agent pauses, human sees the correction. ALLOW is silent. | Medium | Sub-5ms target for ALLOW. BLOCK latency is acceptable — it means Onus just prevented a mistake. |
| **False positives erode trust.** Blocking a legitimate `rm -rf node_modules` destroys developer goodwill. | High | Aggressive allowlist for known-safe paths (node_modules, .cache, __pycache__, .next, target, dist, build, .venv, venv). Users can customize. Default to "warn" for uncertain cases, not "block." |
| **Correction quality matters.** A bad correction prompt ("you did something wrong" without specifics) makes the agent retry the same thing. | Medium | Correction templates are per-rule and specific: "You tried to delete X. The task was Y. This file is outside the task scope." |
| **Cross-platform IPC.** Windows named pipes behave differently from Unix sockets. | Medium | Abstract IPC behind a platform trait. Windows uses named pipes, everything else uses Unix domain sockets. |
| **Claude Code hooks API changes.** Anthropic could change the hook interface. | Medium | Hook protocol is minimal (stdin JSON, stdout JSON, exit codes). Adapting is a few-line change. Also: VS Code extension and shell wrapper don't depend on Claude hooks. |
| **Rollback generality.** Compensation logic is per-integration work; will never be fully universal. | Medium | Reversibility classification + pre-action gates deliver 80% of value. We block first, rollback is bonus. |
| **Platforms build this natively.** Anthropic adds safety rules to Claude Code. | Medium | Neutrality + cross-agent coverage. Enterprise runs agents from 5+ vendors. Platforms can only cover their own. |

---

## 11. What We Explicitly Do NOT Build (anti-scope)

- **Token-level tracing.** LangSmith, LangFuse, and Weights & Biases already do this well. We compete on actions, not tokens.
- **Prompt engineering tools.** Not our lane.
- **Model evaluation benchmarks.** We evaluate agent task completion, not model capability.
- **Agent hosting/runtime.** We are the safety layer, not the execution layer.
- **A new dashboard as the primary interface.** Integrations first. Dashboard is for admins only.

---

Start with the block. End as the control plane for AI labor.
