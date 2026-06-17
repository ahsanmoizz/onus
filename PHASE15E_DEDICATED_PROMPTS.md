# ONUS Phase 15E — Dedicated Surface Prompts

## Use

Copy this file into the repository root as `PHASE15E_DEDICATED_PROMPTS.md`.
For each new Claude Code/Codex session paste:

```text
Read and obey AGENTS.md, CLAUDE.md, all canonical documents, and
PHASE15E_DEDICATED_PROMPTS.md.

Execute only prompt <PROMPT-ID>.

Do not execute another surface. Do not stop after inspection, planning,
reporting, adapter scaffolding, or test creation. Complete all engineering,
automated tests, setup/uninstall/doctor/version/capability work, runtime
harnesses, reports, Git commits, and cleanup possible without my interactive
login, payment, large IDE installation, or manual GUI action.

Where a real login or GUI launch is unavoidable, build a complete executable
user-run verification package and leave only those exact steps for me.
```

---

# Shared rules for all prompts

Read `AGENTS.md`, `CLAUDE.md`, all canonical docs, all Phase 15 reports, the
surface report, adapter code, L2/L3/L4 code, approvals, receipts, redaction,
installers, doctor checks, and tests.

Do not modify locked documents. Work on exactly one surface. Select the
strongest real route: native pre-action hook, Onus-owned executor, MCP gateway,
L3 workspace, then L4 authority. Never call a cooperative hook L2. Never call an
extension L3.

Automatically complete where applicable:

- official current-name and version verification;
- setup, idempotent setup, uninstall, reinstall;
- version detection and supported-version checks;
- `onus doctor` and capability reporting;
- event normalization and task/session linkage;
- allow, deny, correction, approval, exact payload binding;
- changed/expired/reused approval rejection;
- fail-closed behavior;
- receipts and secret redaction;
- L3 fallback;
- protocol, failure, bypass, and regression tests;
- a disposable runtime workspace;
- a user-run live-test package;
- report updates and clean Git commits.

Automatically install only small, free, reversible dependencies from official
sources. Do not silently install large IDEs, paid software, drivers, or host
security components. Never ask for secrets in chat.

For a surface needing real login or GUI interaction create:

`runtime-verification/<surface-slug>/`

with:

- `README.md`
- `setup.ps1`
- `run-live-test.ps1`
- `verify-results.ps1`
- `cleanup.ps1`
- allowed file
- protected file
- protected test
- fake secret marker
- expected-state manifest
- evidence directory

The package must prepare everything and verify:

- allowed read/write;
- denied destructive command;
- denied protected write;
- denied test deletion;
- denied secret insertion;
- structured correction;
- retry;
- exact approval;
- changed payload rejection;
- bypass behavior;
- L3 fallback;
- cleanup.

Run all applicable Rust, Python, extension-host, protocol, approval-binding,
redaction, receipt, bypass, and L3 tests. For every deny, prove the side effect
did not occur. For every allow, prove it occurred.

Use one final status:

- `LIVE PRODUCT VERIFIED`
- `LIVE PRODUCT VERIFIED WITH DOCUMENTED LIMITATIONS`
- `ENGINEERING COMPLETE — USER LIVE VERIFICATION PENDING`
- `LIVE FRAMEWORK RUNTIME VERIFIED`
- `SUPPORTED THROUGH TESTED L3 CONTAINMENT`
- `PROVEN UNSUPPORTED BY CURRENT PLATFORM`
- `BLOCKED ONLY BY USER INSTALLATION`
- `BLOCKED ONLY BY USER AUTHENTICATION`
- `BLOCKED ONLY BY PAID SUBSCRIPTION`
- `BLOCKED ONLY BY OPERATING-SYSTEM LIMITATION`
- `FAILED — ENGINEERING DEFECT REMAINS`

`ENGINEERING COMPLETE — USER LIVE VERIFICATION PENDING` closes Phase 15
engineering for that surface, but it is not a public live-compatibility claim.

Commit only this surface. Do not commit tokens, caches, databases, runtime
workspaces, installed product files, or sensitive transcripts.

Final response must include: surface, version, branch, changed files, engineering
completed, tests, enforcement route, known bypasses, live-package path, remaining
user action, commits, and final status.

---

# P15E-01 — Claude Code CLI

Complete Claude Code CLI local interactive-agent support.

- Verify current official CLI name, executable, version, and auth state.
- Implement/repair project hook setup and uninstall.
- Route real `PreToolUse` Bash, file-write/edit, MCP, and supported subagent
  events into Onus.
- Return corrections using the exact Claude Code hook protocol.
- Add approval binding, changed/expired/reused approval tests.
- Test timeout, crash, malformed output, disabled hook, Onus unavailable, and
  direct-hook bypass.
- Provide tested L3 launch fallback.
- Build an authenticated user-run Claude live package with exact prompts and
  automatic evidence verification.

# P15E-02 — OpenAI Codex CLI

Complete OpenAI Codex CLI support separately from OpenAI Agents SDK.

- Verify exact executable, version, authentication, quota, and permission state.
- Distinguish CLI, IDE extension, and delegated/cloud execution.
- Use the strongest available MCP, owned-executor, approval, or L3 route.
- Do not invent a native hook.
- Implement setup/uninstall/doctor/capability/version checks, correction,
  approvals, receipts, fail-closed behavior, and L3 fallback.
- Test binary permission failure, quota/auth failure, child process, direct
  shell/file bypass, and direct MCP bypass.
- Build a user-run authenticated Codex CLI live package.

# P15E-03 — Gemini CLI

Complete Gemini CLI support.

- Verify current official command, version, auth flow, hooks, MCP, permissions,
  headless mode, and subagent behavior.
- Implement setup/uninstall/doctor/capability/version checks, normalization,
  correction, exact approvals, receipts, and fail-closed behavior.
- Test interactive/headless modes, disabled integration, timeout, malformed
  events, child processes, direct MCP bypass, and L3 fallback.
- Build a user-run login/live-test package.

# P15E-04 — Cursor CLI

Complete Cursor CLI separately from Cursor IDE and Background Agents.

- Verify current official command and version.
- Implement strongest MCP, permission, owned-executor, or L3 route.
- Implement setup/uninstall/doctor/capability/version checks, correction,
  approvals, receipts, strict failure behavior, and L3 fallback.
- Test terminal actions, file mutations, disabled configuration, child process,
  direct shell, and direct MCP bypass.
- Build a user-run authenticated CLI package.

# P15E-05 — Continue CLI

Complete Continue CLI separately from Continue IDE extensions and Continue
Checks.

- Verify official command and version.
- Implement supported MCP, permission, tool-wrapper, or L3 route.
- Complete setup/uninstall/doctor/capability/version checks, correction,
  approvals, receipts, and failure handling.
- Test interactive/headless modes, direct tool bypass, and child processes.
- Build a user-run live package. Do not claim VS Code/JetBrains support.

# P15E-06 — JetBrains Junie CLI / ACP

Complete Junie CLI/ACP separately from Junie IDE.

- Verify whether the current product exposes CLI, ACP, or both.
- Detect version, auth, executable, and OS restrictions.
- Implement strongest ACP/MCP/owned-executor/L3 route.
- Complete setup/uninstall/doctor/capability/version checks, correction,
  approvals, receipts, and fail-closed behavior.
- Test ACP disconnects, malformed events, direct filesystem bypass, child
  processes, and L3 fallback.
- Build a user-run package. Do not claim IDE support.

# P15E-07 — Aider

Complete Aider support.

- Verify executable/version and provider requirements.
- Integrate controlled process launch, Git checkpoints, command mediation, and
  L3 containment.
- Complete setup/uninstall/doctor/capability/version checks, correction,
  approval binding, receipts, rollback, and fail-closed behavior.
- Test allowed edit, protected edit, test deletion, secret insertion,
  destructive command, Git restore, direct shell, and out-of-workspace writes.
- Build a user-run authenticated Aider package.

# P15E-08 — Windsurf Editor / Cascade

Complete Windsurf Editor/Cascade support.

- Verify exact official product identity/version. Do not equate Windsurf,
  Cascade, or Devin Desktop solely from local metadata.
- Research official pre-read/pre-write/pre-command/pre-MCP hooks and failure
  semantics.
- Complete hook/extension setup/uninstall, doctor, capability, version checks,
  normalization, correction, approvals, receipts, and strict failure behavior.
- Prove protocol deny-before-side-effect and provide L3 fallback for disabled
  hooks/direct paths.
- Build a GUI live package with exact workspace and prompts plus automatic
  verification.

# P15E-09 — Visual Studio Code Agents

Complete generic VS Code Agents support.

- Verify current VS Code version and official agent-hook API status.
- Distinguish local, background, cloud, and third-party agents.
- Complete extension install/activation/uninstall, doctor, capability, version
  checks, normalization, correction, approvals, receipts, and failure handling.
- Never treat post-hoc events as pre-action blocking.
- Use L3 where native APIs cannot block before side effects.
- Add extension-host tests and a real-agent GUI live package.

# P15E-10 — Cline for VS Code

Complete Cline separately from generic VS Code Agents.

- Verify official extension identifier/version and safely install the free
  official extension where allowed.
- Research current hooks, MCP, permissions, and approvals.
- Complete setup/uninstall/doctor/capability/version checks, normalization,
  correction, approvals, receipts, and strict failure handling.
- Add protocol and extension-host tests plus L3 fallback.
- Build a GUI live package for allow, deny, correction, retry, approval,
  changed payload, disabled extension, bypass, and cleanup.

# P15E-11 — Google Antigravity

Complete Google Antigravity support.

- Verify current official product architecture/version and exact local surface.
- Do not infer agent support merely from VS Code extension-host compatibility.
- Verify extension loading, not only file copying.
- Identify official hooks/plugins/MCP/SDK/approval surfaces.
- Complete setup/uninstall/doctor/capability/version checks, normalization,
  correction, approvals, receipts, and failure behavior.
- Add extension-host tests, L3 fallback, and a user-run Antigravity live package.

# P15E-12 — Cursor Agent in Cursor IDE

Complete Cursor IDE agent separately from Cursor CLI and Background Agents.

- Verify current IDE version and native permission/MCP surfaces.
- Do not invent universal pre-action hooks.
- Complete setup/uninstall/doctor/capability/version checks, MCP/approval
  routing, correction, receipts, and fail-closed behavior.
- Use L3 as the enforceable boundary where native interception is incomplete.
- Test settings tampering, disabled extension, direct terminal/file access,
  child processes, and MCP bypass.
- Build a GUI live package.

# P15E-13 — Continue Agent for VS Code

Complete Continue VS Code separately from Continue CLI and JetBrains.

- Verify official extension identifier/version/current architecture.
- Safely install the free extension where allowed.
- Complete MCP/tool routing, setup/uninstall/doctor/capability/version checks,
  correction, approvals, receipts, and failure handling.
- Add extension-host/protocol tests and L3 fallback.
- Build a GUI live package. Do not use CLI tests as IDE proof.

# P15E-14 — Continue Agent for JetBrains

Complete Continue JetBrains separately from Continue CLI and VS Code.

- Verify supported JetBrains products, plugin identifier/version, and OS needs.
- Complete plugin/config setup automation where possible, uninstall, doctor,
  capability/version checks, MCP/tool routing, correction, approvals, receipts,
  and failure handling.
- Add all non-GUI tests and L3 fallback.
- Build a precise JetBrains GUI package.
- If JetBrains is absent, complete all engineering and finish as
  `BLOCKED ONLY BY USER INSTALLATION`.

# P15E-15 — JetBrains Junie IDE Agent

Complete Junie IDE separately from Junie CLI/ACP.

- Verify official plugin identity, supported IDEs, version, auth, and OS needs.
- Identify official ACP/MCP/tool/permission/approval surfaces.
- Complete setup automation where possible, uninstall, doctor, capability,
  version checks, normalization, correction, approvals, receipts, and failure
  behavior.
- Add all non-GUI tests and L3 fallback.
- Build a complete JetBrains/Junie GUI package.

# P15E-16 — Cursor Background Agents

Complete remote Cursor Background Agents separately from local Cursor.

- Verify official remote product/account/subscription requirements.
- Identify cloud execution, repository access, credentials, webhook/API,
  policy, and approval surfaces.
- Complete local control-plane configuration, capability detection, signed API
  validation, exact approvals, receipts, and fail-closed behavior.
- Define L4 authority boundaries.
- Add automated API/protocol/security tests without credentials.
- Build a user-run remote verification package for disposable repo connection,
  task execution, deny/correction, approval, and cleanup.

# P15E-17 — GitHub Copilot SDK

Complete GitHub Copilot SDK support.

- Verify current package/version/auth and official pre-tool control.
- Build a real example application using an Onus-owned executor.
- Complete setup/uninstall/doctor/capability/version checks, normalization,
  correction, approvals, receipts, fail-closed behavior, and unwrapped bypass
  tests.
- Add local real-library tests with deterministic provider fixtures.
- Build an authenticated SDK live-agent package.

# P15E-18 — OpenAI Agents SDK

Finish existing OpenAI Agents SDK work.

- Preserve passing wrapper-runtime behavior.
- Verify current SDK version and tool-guardrail scope.
- Add exact approval, changed/expired approval, fail-closed, evaluator failure,
  receipt failure, redaction, unwrapped-function, and hosted/built-in bypass tests.
- Add tested L3 containment for bypassable local paths.
- Build a secure hosted-model user-run package.
- Separate custom function-tool coverage from hosted/built-in tools.

# P15E-19 — LangChain / LangGraph

Finish existing LangChain/LangGraph work.

- Preserve passing StructuredTool/callback/graph-node behavior.
- Verify current versions.
- Add exact approval, changed/expired approval, fail-closed, evaluator failure,
  receipt failure, and redaction tests.
- Add direct `.func()`/unwrapped bypass and tested L3 containment.
- Test LangGraph node/state error and retry behavior.
- Build a hosted-model user-run package.

# P15E-20 — CrewAI

Complete CrewAI support.

- Verify current package/version/process/tool hooks/auth requirements.
- Build a real CrewAI example using official before-tool interception or an
  Onus-owned executor.
- Complete setup/uninstall/doctor/capability/version checks, normalization,
  correction, approvals, receipts, and fail-closed behavior.
- Add real-library tests with deterministic provider fixtures.
- Test exception/fail-open paths, direct tool bypass, and L3 fallback.
- Build a hosted-model user-run package.

---

# Final Phase 15 engineering-closure prompt

After P15E-01 through P15E-20:

```text
Read AGENTS.md, CLAUDE.md, all canonical documents, every Phase 15E surface
report, and PHASE15E_DEDICATED_PROMPTS.md.

Perform Phase 15 engineering-closure verification only. Do not begin Phase 16.

For all 20 surfaces verify that all automatically solvable engineering is
complete; setup, uninstall, doctor, capability, version checks, correction,
approval binding, fail-closed behavior, receipts, redaction, bypass tests, and
L3 fallback exist where applicable; automated tests pass; and every login/GUI
surface has a complete executable user-run live package.

Create reports/integrations/PHASE15_ENGINEERING_CLOSURE.md.

Phase 15 engineering may be marked complete only when no surface has
FAILED — ENGINEERING DEFECT REMAINS and every user-dependent surface has a
complete executable live-verification package.

Do not claim public live compatibility for any surface still pending user live
verification.

Run the complete regression suite, verify the spec lock, inspect Git status,
commit the closure report, and stop.
```
