# Onus Integration Configurations & Live-Test Runbooks

> Phase 17.14 — provider/integration configuration, live-test procedures, and runbooks.
> Purpose: document every integration's setup, configuration, health check, and live-test steps.

---

## 1. Claude Code CLI (L1 Cooperative Hook)

### Setup
```bash
onus setup --claude
```
This installs the `onusHook` into `~/.claude/settings.json`:
```json
{
  "onusHook": {
    "preToolUse": "onus evaluate --action-json",
    "postToolUse": "onus log"
  }
}
```

### Configuration
| Variable | Default | Description |
|---|---|---|
| `ONUS_STRICT` | `""` | Set to `1` or `true` to deny on evaluator failure |
| `ONUS_POLICY_VERSION` | crate version | Override policy version string |
| `ONUS_APPROVAL_TTL_SECS` | `300` | Approval TTL in seconds |

### Health Check
```bash
onus doctor --claude
```
Expected: configuration details printed, hook file present, JSON valid.

### Live Test
```bash
# Verify hook is active
claude "run echo hello" 2>&1 | grep -i "onus\|evaluated"

# Verify denial
claude "run rm -rf /tmp/test" 2>&1 | grep -i "denied\|blocked"
```

---

## 2. Open AI Codex CLI (L1 Cooperative Hook)

### Setup
```bash
onus setup --codex
```
Installs hook into Codex CLI configuration.

### Health Check
```bash
onus doctor --codex
```

### Live Test
```bash
# Run a command through codex with onus wrapper
codex "list files in /tmp" 2>&1 | grep -i "onus"
```

---

## 3. Google Antigravity (L1 Cooperative Hook)

### Setup
```bash
onus setup --antigravity
```

### Health Check
```bash
onus doctor --antigravity
```

---

## 4. Cursor IDE (L1 Cooperative Hook)

### Setup
```bash
onus setup --cursor
```
Installs `onusHook` into Cursor's `settings.json`.

### Health Check
```bash
onus doctor --cursor
```
Expected: `rules.json` present with valid agent rules.

---

## 5. VS Code Extension (L1 Cooperative Hook)

### Setup
```bash
onus setup --vscode
```

### Verification
Check that the extension is installed:
```bash
code --list-extensions | grep onus
```

---

## 6. MCP Proxy (L2 Intercepted)

### Setup
```bash
onus mcp-proxy --port 8082
```

### Configuration
MCP clients point to `http://127.0.0.1:8082/mcp` instead of the real MCP server. The proxy intercepts all tool calls, evaluates through the Onus policy engine, and only forwards allowed actions.

### Health Check
```bash
curl -s http://127.0.0.1:8082/health | grep -i ok
```

### Live Test
```bash
# Send a test tool call
curl -X POST http://127.0.0.1:8082/mcp \
  -H "Content-Type: application/json" \
  -d '{"tool":"read_file","arguments":{"path":"/etc/passwd"}}'
```
Expected: evaluated and logged by Onus policy engine.

---

## 7. Shell Wrapper (L2 Intercepted)

### Setup
```bash
onus shell install
```

### Verification
```bash
onus doctor | grep shell
```

### Live Test
```bash
onus run "echo hello"
# Should print evaluation verdict before executing
```

---

## 8. Approval Server (Human-in-the-Loop)

### Setup
```bash
onus approvals serve --port 9191
```

### Health Check
```bash
curl -s http://127.0.0.1:9191/api/approvals | head -20
```

### Live Test
Trigger a pending action requiring approval, approve via:
```bash
onus approvals approve <action-id>
```

---

## 9. Dashboard (Audit & Status)

### Setup
```bash
onus dashboard --port 9292
```

### Health Check
```bash
curl -s http://127.0.0.1:9292/api/log?limit=5
```

---

## 10. Policy Signing (Ed25519)

### Key Generation
```bash
onus rules generate-keys
```
Creates `onus_private_key.pem` and `onus_public_key.hex` in the Onus config directory.

### Sign Policy
```bash
onus rules sign policy.json --key onus_private_key.pem
```
Outputs `policy.signed.json`.

### Install & Verify
```bash
onus rules install policy.signed.json
onus rules verify policy.signed.json
```

---

## 11. Container Workspace (L3 Isolated)

### Linux Setup
```bash
onus workspace create --sandbox
```
Requires bubblewrap (`bwrap`) installed.

### Live Test
```bash
onus workspace exec "rm -rf /"  # Should fail — sandboxed
```

---

## 12. L4 Authority (Sovereign Mode)

### Setup
```bash
# Configure authority credentials
onus authority add --provider filesystem --credential ./credentials.json

# List configured authorities
onus authority list
```

### Live Test
```bash
# Execute a controlled operation through Onus
onus authority execute --action read-file --target /etc/hostname
```

---

## Quick Start: Integration Smoke Test

```bash
# 1. Verify binary
onus --version

# 2. Health check
onus doctor

# 3. Policy engine test
onus evaluate --action '{"tool":"write","arguments":{"path":"/etc/passwd","content":"bad"}}'
# Expected: DENY

# 4. Log test
onus log --limit 5

# 5. Approval server
onus approvals serve --port 9191 &
sleep 1
curl -s http://127.0.0.1:9191/api/approvals

# 6. Dashboard
onus dashboard --port 9292 &
sleep 1
curl -s http://127.0.0.1:9292/api/log?limit=3
```

---

## Troubleshooting

| Symptom | Likely Cause | Fix |
|---|---|---|
| `onus` command not found | Not installed | Run install script |
| Hook not firing | Config file not updated | Run `onus setup --claude` |
| "Failed to open memory store" | Missing directory | `mkdir -p ~/.config/onus` |
| daemon not responding | Not started | `onus daemon start` |
| MCP proxy connection refused | Not running | `onus mcp-proxy --port 8082 &` |

---

## Security Notes

- Never commit `onus_private_key.pem` to version control
- Approval server binds to `127.0.0.1` only — do not expose to network
- Audit logs may contain action metadata — handle with appropriate access controls
- L1 hooks are best-effort; L2+ required for mandatory enforcement
- The `--purge` flag on uninstall deletes all data including audit trail
