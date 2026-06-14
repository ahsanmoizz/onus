## 7. Vibe Coder protection flow

Implement this complete flow.

```text
User submits vague request
        ↓
Onus analyzes intent
        ↓
Onus identifies missing scope and danger
        ↓
Onus proposes a safe task contract
        ↓
User accepts or edits
        ↓
Agent starts inside governed environment
        ↓
Each action is evaluated
        ↓
Low risk: automatically approved
Medium risk: Onus recommendation + user approval
High risk: denied with correction
Critical risk: session paused or terminated
        ↓
Agent retries
        ↓
Onus verifies evidence
        ↓
User receives a working product or a precise unresolved report
```

Example:

User:

```text
Make the whole website work and remove anything causing errors.
```

Onus:

```text
I should not send this request directly to the agent.

“Remove anything causing errors” could cause deletion of tests,
features, configuration, or data.

I found these current failures:

1. Authentication test failure
2. Missing frontend environment variable
3. Database connection error

Recommended plan:

Stage 1: Repair authentication without deleting tests
Stage 2: Validate frontend configuration
Stage 3: Diagnose database connectivity without schema changes

A checkpoint will be created before every stage.

[START SAFE PLAN]
[EDIT]
[CANCEL]
```

---

## 8. Senior quality maintenance

For professional teams, Onus must enforce quality obligations.

Support:

* architecture rules;
* module boundaries;
* API compatibility;
* test preservation;
* minimum coverage;
* linting;
* type checking;
* security scanning;
* dependency review;
* performance budgets;
* migration review;
* backward compatibility;
* documentation obligations;
* required reviewers;
* repository-specific completion checks.

Example:

```text
ALLOW WITH OBLIGATIONS

The code change is inside scope.

Before completion, Onus requires:

- authentication unit tests;
- integration test for expired JWT;
- typecheck;
- secret scan;
- no coverage reduction;
- no public API break.
```

Onus should maintain quality without interrupting every harmless action.

---

## 9. Recovery and rollback

Before a mutating session:

* create a Git worktree or isolated overlay;
* create an initial checkpoint;
* record repository state;
* identify external resources;
* classify reversibility.

Use these classes:

* `R0` — read-only
* `R1` — automatically reversible
* `R2` — snapshot reversible
* `R3` — compensatable external action
* `R4` — irreversible or mitigation-only

For every mutating action store:

* pre-state;
* proposed mutation;
* decision;
* exact executed payload;
* post-state;
* inverse operation;
* compensation operation;
* verification result.

Support:

* revert individual action;
* revert action group;
* revert full session;
* restore checkpoint;
* execute compensation;
* explain what cannot be reversed.

Never claim a leaked secret is repaired by reverting a file.

Trigger rotation and revocation where required.

---

## 10. Evidence-based completion

The agent cannot mark a task complete by text alone.

Onus must check the task contract.

Possible evidence:

* required tests pass;
* original tests remain enabled;
* no secret detected;
* lint passes;
* typecheck passes;
* expected API response observed;
* browser behavior verified;
* migration dry-run passes;
* final diff remains inside scope;
* no unresolved approvals;
* no critical policy violations;
* independent verifier accepts the result.

Final statuses:

* `COMPLETED_VERIFIED`
* `COMPLETED_WITH_EXCEPTIONS`
* `HUMAN_REVIEW_REQUIRED`
* `FAILED_SAFELY`
* `ROLLED_BACK`
* `TERMINATED`

---

## 11. Model provider architecture

Do not tightly couple Onus to one LLM vendor.

Create a provider interface supporting:

* cloud model providers;
* enterprise-hosted models;
* local models;
* disabled/offline mode.

Example interface:

```rust
pub trait SemanticReviewer {
    fn interpret_task(
        &self,
        input: TaskInterpretationRequest
    ) -> Result<TaskInterpretation>;

    fn review_action(
        &self,
        input: ActionReviewRequest
    ) -> Result<SemanticAssessment>;

    fn generate_correction(
        &self,
        input: CorrectionRequest
    ) -> Result<StructuredCorrection>;

    fn verify_completion(
        &self,
        input: CompletionReviewRequest
    ) -> Result<CompletionAssessment>;
}
```

Provider configuration must include:

* model;
* endpoint;
* timeout;
* privacy mode;
* maximum data exposure;
* redaction policy;
* fallback behavior;
* cost budget.

On LLM failure:

* deterministic rules continue;
* critical actions fail closed;
* low-risk actions follow configured policy;
* no action is approved merely because the model is unavailable.

---

## 12. Required implementation sequence

Implement in this order:

1. Task-contract lifecycle
2. Prompt Intake Guardian
3. Memory schemas and redaction
4. LLM provider interface
5. Intent Interpreter
6. Semantic Risk Critic
7. Structured Correction Generator
8. Approval Decision Broker
9. Exact action-hash approval binding
10. Beginner Guardian Mode
11. Professional Reviewer Mode
12. Independent Completion Verifier
13. Transactional checkpoints and reversibility classes
14. Native hook approval adapters
15. Local approval interface
16. L3 isolated workspace
17. L4 credential and privileged-operation broker

Do not start with UI polish.

---

