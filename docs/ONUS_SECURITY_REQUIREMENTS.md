## 14. Non-negotiable rules

* Never blindly execute an ambiguous user prompt.
* Never blindly approve an IDE action.
* Never use an LLM as the only security boundary.
* Never let an LLM override deterministic denial.
* Never store secrets in memory, receipts, or prompts.
* Never claim a task is complete without evidence.
* Never permit silent test weakening.
* Never reuse an approval for a changed payload.
* Never hide irreversible effects.
* Never allow a critical failure to silently fail open.
* Never claim rollback when only mitigation is available.
* Never make the user approve harmless actions repeatedly.
* Never remove human authority over production-risk decisions.

## Locked product definition

Onus is an intelligent execution guardian for AI-assisted software development.

For beginners, it converts unsafe or unclear requests into safe plans, explains risks, prevents destructive agent behavior, and guides the user toward a working result.

For professional engineers, it enforces scope, architecture, quality, evidence, reversibility, and production policy without interrupting harmless work.

For enterprises, it controls execution boundaries, approvals, credentials, and verifiable action records across agents and environments.

Its intelligence comes from LLM-assisted intent understanding, semantic review, correction, memory, and verification.

Its guarantees come from deterministic policy, controlled execution, isolation, exact approvals, checkpoints, and external authority.

This specification is now part of the target Onus product and must not be removed or weakened without an explicit architecture decision record.
