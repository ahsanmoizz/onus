# ONUS Guardian Intelligence, Memory and Approval System

This directive extends the existing Onus architecture and is now part of the target product specification.

Do not weaken or replace the current deterministic policy engine, Rust evaluator, Guardian SDK, audit system, approval system, rollback system, or L1–L4 architecture.

The purpose of this feature is to make Onus act as an intelligent guardian for inexperienced developers and a strict quality reviewer for senior engineers.

## 1. Core operating principle

Onus must evaluate both:

1. **What the user is asking the agent to do**
2. **What the agent is about to execute**

The LLM may understand intent, identify ambiguity, explain risk, propose corrections, maintain useful project context, and verify semantic alignment.

The LLM must never have final authority over deterministic security denials.

The decision order must be:

1. Environment and identity validation
2. Deterministic safety policy
3. Scope and task-contract policy
4. Reversibility and blast-radius analysis
5. Heuristic analysis
6. LLM semantic review
7. Human approval when uncertainty remains
8. Controlled execution
9. Independent verification
10. Receipt and memory update

A deterministic denial cannot be changed to allow by the LLM.

---

## 2. Prompt Intake Guardian

Before an agent begins work, Onus must inspect the user’s original request.

The Prompt Intake Guardian must detect:

* ambiguous requests;
* dangerously broad requests;
* missing target environment;
* missing repository or service scope;
* conflicting requirements;
* unnecessary destructive operations;
* requests that would weaken security or tests;
* requests that expose credentials;
* requests that the user may not understand;
* requests that cannot be safely reversed;
* requests that do not include a definition of completion.

Examples of dangerous or incomplete prompts:

* “Fix everything.”
* “Make all tests pass.”
* “Delete anything causing errors.”
* “Use my production database and test it.”
* “Put the API key directly in the source.”
* “Give the agent full access.”
* “Deploy whatever works.”
* “Disable security so this feature runs.”

Onus must not blindly pass these prompts to the coding agent.

It must return one of:

* `READY`
* `READY_WITH_SAFE_CONTRACT`
* `CLARIFICATION_REQUIRED`
* `REJECTED_AS_UNSAFE`

Example:

```text
Your request is too broad to execute safely.

“Fix everything” could allow the agent to modify unrelated modules,
delete tests, replace dependencies, or change production configuration.

Recommended task contract:

Objective:
Fix the authentication expiry failure.

Allowed scope:
src/auth/**
tests/auth/**

Protected:
.env
production configuration
database schema
CI/CD workflows

Required evidence:
Existing authentication tests remain enabled and pass.

[ACCEPT CONTRACT]
[EDIT CONTRACT]
[CANCEL]
```

Onus should ask the minimum number of questions necessary.

When the missing information can safely be inferred from repository evidence, Onus may generate a proposed contract instead of blocking the user with unnecessary questions.

---

## 3. Guardian modes

Implement the following user modes.

### 3.1 Beginner Guardian Mode

Designed for vibe coders and users with limited engineering knowledge.

Onus must:

* translate technical danger into simple language;
* explain why an action is dangerous;
* prevent broad destructive requests;
* automatically create safe checkpoints;
* use conservative defaults;
* ask before database, infrastructure, deployment, credential, or large-scope changes;
* prevent test deletion or weakening;
* prevent secrets from being committed;
* guide the user toward a safer task;
* show the consequences before approval;
* provide a recommended option.

Example:

```text
The agent wants to delete 28 files.

Your request was only to fix the login button.

This change is much larger than expected and may break unrelated features.

Recommended decision: BLOCK

[BLOCK]
[REVIEW FILES]
[ALLOW ONCE]
```

Do not display “Allow everything forever” as the primary option.

### 3.2 Professional Reviewer Mode

Designed for experienced developers.

Onus must:

* enforce task scope;
* enforce architecture and repository policies;
* detect excessive diffs;
* protect tests and coverage;
* require lint, typecheck, security scans, and targeted tests;
* detect dependency and configuration drift;
* require evidence before task completion;
* allow low-risk actions without interruption;
* escalate material deviations.

### 3.3 Enterprise Strict Mode

Designed for production and regulated environments.

Onus must:

* deny mutating actions by default unless authorized;
* require environment identity;
* use managed signed policies;
* require exact action-bound approvals;
* enforce credential isolation;
* require L3 or L4 boundaries for production actions;
* fail closed on critical evaluator failure;
* require independent verification;
* produce signed receipts.

---

