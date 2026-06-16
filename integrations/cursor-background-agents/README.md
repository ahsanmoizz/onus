# Onus for Cursor Background Agents

Status: `BLOCKED`.

Cursor Background Agents are a cloud/service surface. This local environment
does not have Cursor credentials, a Cursor cloud workspace, or a service-native
control hook that Onus can runtime-test.

Official documentation reviewed:

- <https://cursor.com/docs/cloud-agent>

## Future Onus Routes

Possible routes require service-specific evidence:

1. Cursor service-native policy/hook surface, if available.
2. Repository policy gates and required checks controlled by Onus receipts.
3. L4 authority control for privileged actions, so the cloud agent never
   receives raw deployment/database credentials.

Local VS Code or Cursor CLI checks do not prove this surface.

## Claim Boundary

Safe claim:

```text
Cursor Background Agents integration is blocked locally pending Cursor cloud
runtime access and a verified control surface.
```

Unsafe claims:

```text
Onus controls Cursor Background Agents.
Local Cursor/VS Code tests prove cloud-agent protection.
Onus can prevent all Cursor cloud side effects without service integration.
```
