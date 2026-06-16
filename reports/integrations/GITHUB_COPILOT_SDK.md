# GitHub Copilot SDK Integration Report

Milestone surface: 5 of 20, GitHub Copilot SDK.

Branch: `integration/github-copilot-sdk`.

Date: 2026-06-16.

## Claim

This surface is `BLOCKED` in Phase 15.

The official SDK package exists, but this local environment does not contain the
credentials or runtime tooling needed to prove a real Copilot SDK integration.

## Official Control Surface

Official documentation reviewed:

- <https://github.com/github/copilot-sdk>
- <https://docs.github.com/en/copilot/concepts/agents/cloud-agent/about-cloud-agent>

## Runtime Discovery

Package registry probe:

```text
npm view @github/copilot-sdk version description repository.url --json
```

Result:

```json
{
  "version": "1.0.1",
  "description": "TypeScript SDK for programmatic control of GitHub Copilot CLI via JSON-RPC",
  "repository.url": "git+https://github.com/github/copilot-sdk.git"
}
```

GitHub CLI probe:

```text
gh NOT_FOUND
```

Credential probe:

```text
GITHUB_TOKEN=ABSENT
GH_TOKEN=ABSENT
COPILOT_TOKEN=ABSENT
```

## Files Added

- `integrations/github-copilot-sdk/README.md`

## Security Notes

- No SDK calls were made.
- No cloud agent action was run.
- No GitHub credential was accessed or required.
- This branch must not be used to claim Copilot SDK enforcement.
- A future adapter must be Onus-owned around the SDK call boundary and fail
  closed if evaluation, authorization, or SDK control fails.

## Final Classification

```text
BLOCKED
```

Runtime proof requires authenticated GitHub/Copilot SDK access.
