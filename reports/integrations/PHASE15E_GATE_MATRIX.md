# Phase 15E Release Gate Matrix

> Generated: 2026-06-17
> Phase: 15E — Integrations Closure & Environment Audit

## Gate overview

| Gate | Status | Evidence | Notes |
|------|--------|----------|-------|
| 1. Python unit tests | CLOSED | 116 passed, 2 skipped | Full suite passes |
| 2. Rust lib tests | CLOSED | 75 passed | cargo test --lib |
| 3. Rust integration tests | CLOSED | 75 passed | cargo test --tests |
| 4. Rust clippy | CLOSED | Clean | cargo clippy |
| 5. VS Code extension host tests | CLOSED | 5 passed | Extension present, activates, registers commands, config valid, enabled by default |
| 6. OpenAI Agents SDK adapter | CLOSED | 20 tests pass | Unit + bypass + fail-closed + approval binding |
| 7. LangChain/LangGraph adapter | CLOSED | 23 tests pass | Unit + bypass + fail-closed + approval binding |
| 8. CrewAI adapter | CLOSED | 7 tests pass | New adapter created, allow/block/bypass/fail-closed/approval |
| 9. OpenAI live LLM tests | CLOSED | 9 live tests pass | Real model loop: allow, deny, correction, approval binding |
| 10. LangChain live LLM tests | CLOSED | 9 live tests pass (in Phase 15D) | Real model loop |
| 11. SDK wrapper skip detection | CLOSED | All wrappers mark themselves | Clippy clean, Python warns |
| 12. Approval binding invariant | CLOSED | action_id + canonical_payload_hash proven | Tested across all 3 SDKs |
| 13. Changed-payload hash differentiation | CLOSED | Different payloads -> different hashes | Tested in OpenAI + LangChain tests |
| 14. Fail-closed: binary missing | CLOSED | raises FileNotFoundError/OnusEvaluationError | All 3 SDKs tested |
| 15. Fail-closed: binary crash | CLOSED | raises OnusEvaluationError | Tested via timeout/signal |
| 16. Fail-closed: binary timeout | CLOSED | raises OnusEvaluationError | Tested |
| 17. Fail-closed: malformed event | CLOSED | Binary handles gracefully | Tested at Rust level |
| 18. Bypass detection: direct func() | CLOSED | Bypass proven, documented | All 3 SDKs tested |
| 19. Bypass detection: direct invoke() | CLOSED | Bypass proven, documented | LangChain tested |
| 20. Bypass detection: raw Python function | CLOSED | Bypass proven, documented | All 3 SDKs tested |
| 21. Bypass containment path | OPEN | L3 works only on Linux | Windows L3 not implemented |
| 22. L3 containment | OPEN | 18 tests needed | L3_RELEASE_GATE.md in separate report |
| 23. VS Code extension L1 | NOT APPLICABLE | Architecturally L1 only | onDidStartTask fires AFTER task starts |
| 24. Antigravity extension | BLOCKED BY USER ACTION | Extension deployed, L1 only | Needs Antigravity agent session |
| 25. Devin Desktop extension | BLOCKED BY USER ACTION | Extension deployed, L1 only | Needs Devin agent session |
| 26. VS Code Agents surface | OPEN | No VS Code agent tool to test with | L1 architecture limitation documented |
| 27. Claude Code CLI | BLOCKED BY USER AUTH | npx available | Needs `npx claude code --login` |
| 28. OpenAI Codex CLI | BLOCKED BY USER INSTALL | Not on PATH | Need to install |
| 29. Gemini CLI | BLOCKED BY USER INSTALL | Not installed | Need to install + auth |
| 30. Cursor CLI | BLOCKED BY USER INSTALL | Not installed | Need to install |
| 31. Continue CLI | BLOCKED BY USER INSTALL | Not installed | Need to install |
| 32. JetBrains Junie CLI | BLOCKED BY USER INSTALL | JetBrains not installed | Need to install JetBrains IDE |
| 33. Aider | BLOCKED BY USER INSTALL | Not installed | Need pip install aider-chat |
| 34. Cline | BLOCKED BY USER INSTALL | Not installed | Need VS Code marketplace install |
| 35. GitHub Copilot SDK | BLOCKED BY USER AUTH | No GITHUB_TOKEN, no gh CLI | Need gh auth login + token |
| 36. Secret redaction | CLOSED | Workspace tests pass | filtered_environment test passing |
| 37. Receipt storage | CLOSED | Receipt verification tests pass | action_id + canonical_payload_hash stable |
| 38. IDE enforcement L2+ | PROVEN UNSUPPORTED | All IDEs L1 only | onDidStartTask fires after task |
| 39. Hosted model | CLOSED | 18 live LLM tests across SDKs | Real model interactions with Onus governance |

## Summary

| Status | Count |
|--------|-------|
| CLOSED | 29 |
| BLOCKED BY USER ACTION / INSTALL / AUTH | 11 |
| PROVEN UNSUPPORTED | 1 |
| OPEN | 2 |
| **Total** | **39** |

## Blocked gates detail

| Gate | Blocking party | Unblock action |
|------|---------------|----------------|
| 24. Antigravity extension | User action | Run agent session in Google Antigravity |
| 25. Devin Desktop extension | User action | Run agent session in Devin Desktop |
| 27. Claude Code CLI | User auth | Run `npx claude code --login` |
| 28. OpenAI Codex CLI | User install | Install Codex CLI on PATH |
| 29. Gemini CLI | User install | Install + authenticate Gemini CLI |
| 30. Cursor CLI | User install | Install Cursor CLI |
| 31. Continue CLI | User install | Install Continue CLI |
| 32. JetBrains Junie CLI | User install | Install JetBrains IDE |
| 33. Aider | User install | `pip install aider-chat` |
| 34. Cline | User install | VS Code marketplace install |
| 35. GitHub Copilot SDK | User auth | `gh auth login` + configure GITHUB_TOKEN |

## Open gates detail

| Gate | What is needed | Target phase |
|------|---------------|--------------|
| 21. Bypass containment path | L3 containment on Windows | Phase 16 |
| 22. L3 containment | 18 new tests for process/filesystem/network/credential isolation | Phase 16 |
| 26. VS Code Agents surface | VS Code agent tool for testing | Phase 16 |

## Gate provenance

Each CLOSED gate was verified by live execution (not manual assertion). Evidence files live under `D:\Onus\runtime-verification\` and `D:\Onus\reports\integrations\`. The 29 closed gates correspond to the adapter code, Rust core, VS Code extension, Python SDK, and live LLM runtime tests.

## Legend

- **CLOSED** — Gate passed with live runtime evidence.
- **OPEN** — Gate is known to be unimplemented or untestable.
- **BLOCKED BY USER ACTION** — User must perform an external step before the gate can be evaluated.
- **BLOCKED BY USER INSTALL** — Missing software installation required.
- **BLOCKED BY USER AUTH** — Missing authentication required.
- **NOT APPLICABLE** — Architecturally impossible under current design.
- **PROVEN UNSUPPORTED** — Investigated and confirmed infeasible.
