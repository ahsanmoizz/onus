## 4. LLM roles

Implement the LLM as separate controlled roles.

### 4.1 Intent Interpreter

Input:

* original user request;
* repository metadata;
* issue or ticket;
* existing project policy;
* current environment.

Output:

* normalized objective;
* allowed scope;
* protected scope;
* completion evidence;
* ambiguities;
* risk assumptions;
* questions requiring user confirmation.

### 4.2 Semantic Risk Critic

Input:

* task contract;
* proposed action;
* relevant diff;
* previous actions;
* repository architecture;
* policy findings.

Output:

* whether the action appears aligned with the task;
* whether the magnitude is proportionate;
* possible quality problems;
* possible hidden side effects;
* confidence;
* recommended decision.

The output is advisory.

It cannot override deterministic policy.

### 4.3 Correction Generator

When an action is denied or escalated, produce a structured correction:

```json
{
  "violation": "acceptance_test_deleted",
  "reason": "The deleted test is required evidence for the task.",
  "required_action": "Restore the test and repair the implementation.",
  "constraints": [
    "Do not skip the test",
    "Do not reduce its assertions",
    "Do not modify the database schema"
  ],
  "required_evidence": [
    "The original test passes",
    "The test remains enabled"
  ],
  "retry_allowed": true
}
```

The correction generator must never invent a verified root cause.

Unverified statements must be labeled as hypotheses.

### 4.4 Independent Verifier

Use a separate verification stage after the executor claims completion.

The verifier receives:

* original task;
* task contract;
* final diff;
* complete action trace;
* denied and corrected actions;
* tests and evidence;
* policy exceptions.

The verifier must determine:

* whether the requested outcome was achieved;
* whether tests were weakened;
* whether the implementation drifted;
* whether security was reduced;
* whether required evidence is sufficient;
* whether the task may be accepted.

The executor’s own success statement is not proof.

### 4.5 User Guidance Assistant

For inexperienced users, explain:

* what the agent wants to do;
* why it may be dangerous;
* what safer alternative exists;
* what can and cannot be reverted;
* what the recommended decision is.

This assistant should act as a technical guardian and mentor, not as an uncontrolled executor.

---

## 5. Onus memory system

Implement memory as explicit, scoped data stores.

### 5.1 Session Memory

Stores only the current task:

* original request;
* clarifications;
* task contract;
* agent plan;
* actions;
* denials;
* corrections;
* approvals;
* evidence;
* final outcome.

Expires or is archived when the session ends.

### 5.2 Project Memory

Stores stable project knowledge:

* architecture;
* coding conventions;
* approved dependencies;
* protected paths;
* test commands;
* deployment process;
* repository-specific risks;
* previous accepted design decisions.

Project memory must be versioned and reviewable.

### 5.3 Policy Memory

Stores organizational rules:

* forbidden operations;
* approval requirements;
* environment restrictions;
* credential policy;
* compliance requirements;
* risk thresholds.

Policy memory is authoritative and cannot be silently changed by an agent.

### 5.4 Incident Memory

Stores previous failures:

* action pattern;
* policy violation;
* correction;
* human decision;
* final outcome;
* false-positive or true-positive label.

Use incident memory to improve future warnings.

### 5.5 User Capability Preferences

Optional and user-controlled.

May store:

* preferred explanation level;
* preferred approval frequency;
* whether the user wants beginner or professional explanations;
* trusted local commands;
* preferred development workflow.

Do not secretly profile users.

Do not infer sensitive personal information.

### 5.6 Memory safety

Never store raw secrets.

Apply:

* redaction;
* encryption;
* tenant isolation;
* retention limits;
* deletion controls;
* provenance;
* versioning;
* access control.

The LLM must receive only the minimum relevant memory for the current decision.

---

## 6. Approval Decision Broker

Implement an approval broker that replaces blind IDE approval behavior.

The broker must support these decisions:

* `ALLOW_AUTOMATICALLY`
* `ALLOW_WITH_OBLIGATIONS`
* `REQUIRE_HUMAN_APPROVAL`
* `DENY_WITH_CORRECTION`
* `TERMINATE_SESSION`

### 6.1 Automatic approval

Onus may automatically approve only when all are true:

* the action is inside the task contract;
* the action is low risk;
* no protected resource is touched;
* the environment is non-production;
* the blast radius is below policy limits;
* the action is reversible or read-only;
* no deterministic rule is triggered;
* no approval policy requires a human;
* the action payload is exactly bound to the approval.

Examples:

* reading an allowed source file;
* running a local test command;
* writing a small change inside the allowed module;
* formatting a file;
* generating a temporary artifact.

### 6.2 Human approval

Require human approval for:

* production operations;
* database schema changes;
* destructive SQL;
* infrastructure changes;
* external communications;
* credential use;
* changes outside declared scope;
* large diffs;
* test removal or weakening;
* irreversible or uncertain operations;
* low-confidence semantic decisions.

The approval request must show:

* exact action;
* exact arguments;
* environment;
* reason;
* risk;
* blast radius;
* reversibility;
* expected result;
* safer alternative;
* Onus recommendation.

### 6.3 Approval binding

Every approval must bind to:

* session ID;
* task-contract hash;
* action ID;
* canonical action payload hash;
* policy version;
* environment identity;
* expiry time;
* approver identity.

Changing any argument invalidates the approval.

An approval may not authorize a later different action merely because it uses the same tool name.

### 6.4 IDE integration

Preferred integration order:

1. Native IDE or agent approval API
2. Pre-tool hook response
3. SDK callback
4. MCP approval protocol
5. Onus-owned executor
6. Onus local approval window
7. Experimental UI automation fallback

Do not use blind mouse clicks as the primary approval mechanism.

If UI automation is required because no API exists:

* restrict it to local non-production actions;
* confirm the exact window, action text, and payload hash;
* require accessibility or DOM identity checks;
* disable it in Strict Mode;
* record it as `UI_AUTOMATION_APPROVAL`;
* fail closed if the displayed action cannot be matched exactly.

---

