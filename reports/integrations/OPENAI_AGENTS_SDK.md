# OpenAI Agents SDK Integration Report

Milestone surface: 18 of 20, OpenAI Agents SDK.

Branch: `integration/openai-agents-sdk`.

Date: 2026-06-16.

## Claim

This surface is `BLOCKED` in Phase 15.

The SDK package is not installed locally and no OpenAI API key is present. No
SDK tool call or model run was tested.

## Official Control Surface

Official documentation reviewed:

- <https://developers.openai.com/api/docs/guides/agents>
- <https://openai.github.io/openai-agents-python/agents/>

## Runtime Evidence

Package probe:

```text
python -m pip show openai-agents
WARNING: Package(s) not found: openai-agents
```

Credential probe:

```text
OPENAI_API_KEY=ABSENT
```

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Files Added

- `integrations/openai-agents-sdk/README.md`

## Security Notes

- No SDK call was made.
- No model call was made.
- Future proof must test an Onus-owned tool wrapper, not a standalone mock.
- L2 claims apply only to tools routed through that wrapper.

## Final Classification

```text
BLOCKED
```

Runtime proof requires the SDK package, credentials, and a real tool-wrapper
test.
