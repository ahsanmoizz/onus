# Onus for GitHub Copilot SDK

Status: `BLOCKED`.

The official package can be discovered, but this environment does not have the
GitHub CLI, GitHub/Copilot credentials, or an authenticated Copilot SDK runtime.
No production adapter is claimed from this directory.

Official control surfaces reviewed:

- <https://github.com/github/copilot-sdk>
- <https://docs.github.com/en/copilot/concepts/agents/cloud-agent/about-cloud-agent>

Registry discovery:

```json
{
  "package": "@github/copilot-sdk",
  "version": "1.0.1",
  "description": "TypeScript SDK for programmatic control of GitHub Copilot CLI via JSON-RPC"
}
```

## Intended Onus Route

The safe future route is an Onus-owned SDK/tool-executor wrapper:

1. Normalize every Copilot SDK action into canonical Onus action JSON.
2. Evaluate through Onus deterministic policy before the SDK call.
3. Require exact approval binding for risky actions.
4. Redact secrets before audit persistence.
5. Fail closed for critical evaluation or SDK-control failures.

This is not implemented here because no authenticated SDK runtime is available
for end-to-end testing.

## Claim Boundary

Safe claim:

```text
GitHub Copilot SDK integration is blocked locally pending authenticated SDK
runtime access.
```

Unsafe claims:

```text
Onus controls GitHub Copilot SDK calls.
Onus protects GitHub Copilot cloud agent actions.
Onus has runtime-tested Copilot SDK integration.
```
