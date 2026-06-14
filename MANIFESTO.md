# Onus — The Agent Firewall

**One-sentence:** Onus intercepts AI agent actions before they execute, blocks mistakes, and sends correction prompts — so agents don't destroy what they were asked to build.

---

## The Problem

AI coding agents (Claude Code, Cursor, Devin, Copilot) execute tasks blindly. They write files, run shell commands, make API calls. No guardrail exists between "agent decides" and "agent does."

The result:
- Agents delete or corrupt files outside the task scope
- Agents run dangerous shell commands with no human in the loop
- Agents drift — asked to add tests, they refactor the entire module
- Every team discovers this through an incident, not through planning

This is the agent safety gap. Nobody has solved it.

---

## How Onus Works

```
Agent runs task
      ↓
Onus intercepts (BEFORE the file saves, BEFORE the command runs)
      ↓
Policy engine evaluates: Safe? In-scope? Correct magnitude?
      ↓
    ┌─ ALLOW ──────────────────────────────┐
    │  Action executes                     │
    │  Logged silently to audit trail      │
    │  Agent continues                     │
    └──────────────────────────────────────┘
    
    ┌─ BLOCK ──────────────────────────────┐
    │  Action halted                       │
    │  Correction prompt generated         │
    │  Sent back to agent: "You tried to   │
    │  delete X. The task was Y. Re-evaluate."│
    │  Agent retries with better context   │
    └──────────────────────────────────────┘
    
    ┌─ ESCALATE ───────────────────────────┐
    │  After N retries or critical risk    │
    │  Human gets a Slack notification:    │
    │  "Agent tried to rm -rf /project.    │
    │   [Approve] [Deny] [Take over]"      │
    └──────────────────────────────────────┘
```

Onus is not observability. It is **intervention**. It runs in the critical path between agent decision and agent action.

---

## Integration Surfaces (Universal IDE + Agent Coverage)

Onus is a **local policy engine** with multiple integration surfaces, all sharing the same evaluation core:

| Surface | Agents covered | Mechanism |
|---------|---------------|-----------|
| **Claude Code hook** | Claude Code | `preToolUse` hook in settings.json — intercepts every tool call before execution |
| **VS Code extension** | Cursor, Windsurf, Copilot, CodeBuddy | `onWillSaveTextDocument`, terminal profile contributions, FileSystemWatcher |
| **Python SDK** | LangChain, CrewAI, OpenAI Agents SDK | `@onus.guard` decorator wrapping tool/function calls |
| **MCP proxy** | Any MCP-based agent | Onus runs as MCP middleware — agent → Onus → real tools. All tool calls pass through eval |
| **Shell wrapper** | Terminal-based agents (Devin, Aider) | Wraps `bash`/`zsh` — evaluates commands before they hit the shell |

**Key principle:** The agent doesn't need to know Onus exists. Onus sits between the agent and the world, silent until something needs blocking.

---

## Detection Tiers

```
TIER 1 — Deterministic (ship first)
├── Safety rules:  rm -rf, sudo, chmod 777, curl | bash, env var exfiltration
├── Scope rules:   file write outside allowed paths, edit to non-target files
└── Permission:    agent accessing credentials or secrets it shouldn't

TIER 2 — Heuristic (ship second)
├── Magnitude:     500-line change for a "fix typo" task → flag
├── Goal drift:    asked to write tests, started changing auth middleware
└── Pattern:       agent repeatedly failing on same file → stuck loop detection

TIER 3 — Learned (moat, improvement over time)
├── Anomaly:       this agent's behavior differs from baseline for this task type
├── Cross-deployment: similar task failed at 3 other companies this week → pre-flag
└── Agent fingerprinting: Claude Code behaves differently than Copilot for same task type
```

Tier 3 is the compounding moat — every blocked failure improves detection for every other deployment.

---

## Where Onus Lives

Onus does **not** demand a new dashboard. It meets developers where they already are:

- **In the IDE** — inline warnings: "Onus blocked a file deletion outside scope"
- **In the terminal** — Claude Code sees: `[Onus] BLOCKED: rm -rf ./src. Reason: safety rule #4. Correction: ...`
- **In the PR/MR** — Onus adds a comment: "3 agent actions were blocked during this task. 2 were corrected. 1 was escalated. See trace."
- **In Slack** — escalation notifications with approve/deny buttons

The control plane (SaaS) provides the org-wide view: which agents, which repos, block rate, correction rate, incident history. But the **revert button lives in the merge request**, not in our app.

---

## The Wedge → Control Plane Path

```
PHASE 1 — Ship the Claude Code hook + safety rules
         Open source. One `npx install onus` command.
         "Onus blocked 14 dangerous actions this week."
         Devs install it because it prevents midnight pages.

PHASE 2 — Add VS Code extension + Python SDK
         Cover Cursor, Windsurf, Copilot, LangChain.
         Audit trail becomes visible in PRs.
         "Onus corrected 200+ agent mistakes this quarter."

PHASE 3 — SaaS control plane
         Org-wide policy management. SOC 2.
         Cross-deployment learning kicks in.
         "Onus detected a pattern: agents from vendor X fail on Y."

PHASE 4 — Control plane for all agent activity
         Pre-approval gates. Reversibility classification.
         Blast-radius mapping. Compensation infrastructure.
```

Phase 1 is the wedge. It's small, it's free, it solves an immediate pain (Claude Code running destructive commands). Phase 4 is the $50B outcome.

---

## Why This Is Defensible

1. **Interception > Observation.** Everyone is building dashboards. Nobody is building an agent firewall. Observability is a crowded market; intervention is empty.

2. **The data moat.** Every blocked bad action is a training example. After 10,000 deployments, Onus has the world's largest dataset of agent failure modes. No competitor can replicate that without the deployments.

3. **Neutrality is structural.** Anthropic can't credibly audit Claude Code. GitHub can't credibly audit Copilot. They're incentivized to make their agents look good. Onus has no such conflict — it works across all agents and all vendors.

4. **Integration depth.** The Claude Code hook approach (preToolUse) is deep integration that a generic SaaS can't easily replicate. Each surface requires understanding the specific agent's execution model.

5. **No API dependency.** Onus evaluates at the action level (file writes, shell commands, API calls), not at the LLM API level. It doesn't need API keys, doesn't send data to third parties. The core engine runs locally.

---

## Architecture

```
┌─────────────────────────────────────────────┐
│                  Onus Core                   │
│              (Rust binary, local)            │
│                                             │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐ │
│  │ Policy   │  │ Scope    │  │ Learning  │ │
│  │ Engine   │  │ Tracker  │  │ Loop      │ │
│  │          │  │          │  │           │ │
│  │ Rules +  │  │ What     │  │ Feedback  │ │
│  │ Heuristics│ │ files/   │  │ from      │ │
│  │          │  │ paths are│  │ blocks →  │ │
│  │ Tier 1-3 │  │ in scope │  │ better    │ │
│  └──────────┘  └──────────┘  └───────────┘ │
│                                             │
│  ┌──────────────────────────────────────┐   │
│  │         Audit Trail (immutable)       │   │
│  │    Every action + eval + outcome      │   │
│  └──────────────────────────────────────┘   │
└──────────┬──────────┬──────────┬────────────┘
           │          │          │
    ┌──────┘    ┌─────┘    ┌────┘
    ▼           ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐
│ Claude │ │  VS    │ │Python  │
│ Code   │ │ Code   │ │  SDK   │
│ Hook   │ │  Ext   │ │        │
└────────┘ └────────┘ └────────┘
    ▼           ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐
│  MCP   │ │ Shell  │ │ REST   │
│ Proxy  │ │Wrapper │ │  API   │
└────────┘ └────────┘ └────────┘
```

---

## The Honest Risk Register

| Risk | Answer |
|------|--------|
| Platforms build interception natively | They audit their own agents — conflict of interest. Neutrality wins |
| Interception adds latency | Policy engine is local, sub-5ms eval. Only blocks cause user-visible delay |
| Agents find ways around interception | Surface coverage is the moat. You'd need to bypass the shell, the editor, AND the SDK simultaneously |
| Rollback is hard to generalize | True. Pre-action blocking delivers 80% of value without full reversibility. We block first, rollback is bonus |
| Open source → hard to monetize | Developer trust earned via OSS. Org features (policies, cross-deployment learning, SOC 2) are SaaS |

---

## Operating Principles

- **Block before you log.** Intervention over observation. Always.
- **Live where the work lives.** IDE, terminal, PR, Slack. Never a new dashboard.
- **Default allow, selectively block.** Onus is not a permission gate for every action. It's a safety net for the dangerous ones.
- **Be honest about what can't be undone.** Say it before the action, not after.
- **Ship weekly.** Speed is the strategy. Platforms are coming.

---

## Phase 1 — What To Build Now

1. **Onus Core** — Rust binary. Policy engine + scope tracker + audit trail. Runs locally, no cloud dependency.
2. **Claude Code Hook** — `preToolUse` hook that calls Onus Core before every tool execution. Block/allow/correct decision in under 10ms.
3. **Safety rule set v0** — 10-15 deterministic rules covering the most common dangerous actions (rm -rf, sudo, env var leaks, writes outside working directory, curl-to-bash patterns).
4. **CLI installer** — `npx @onus/install` or `brew install onus`. One command, hooks wired into Claude Code automatically.
5. **Minimal audit trail** — SQLite, local. Every action + decision logged. Viewable via `onus log`.

**Ship this in 2 weeks. Get it in front of 100 Claude Code users. The feedback from real blocks will tell you what to build next.**

---

Start with the block. End as the control plane for AI labor.
