# Onus for Visual Studio Code Agents

Status: `VERIFIED_WITH_LIMITATIONS`.

Onus includes a VS Code extension skeleton at:

- `onus/bindings/vscode/package.json`
- `onus/bindings/vscode/src/extension.js`

The extension is an L1 cooperative integration. It can surface Onus evaluation
for VS Code terminal/task-related flows and provide a status/log UI. It must
not be described as a universal pre-execution gate for every VS Code agent or
GitHub Copilot tool call.

Official control surfaces reviewed:

- <https://code.visualstudio.com/docs/agents/overview>
- <https://code.visualstudio.com/api>

## Local Checks

The local environment has VS Code installed:

```text
1.124.2
```

The extension JavaScript parses:

```text
node --check onus/bindings/vscode/src/extension.js
passed
```

The extension package metadata can be parsed after stripping a UTF-8 BOM:

```json
{
  "name": "onus-firewall",
  "version": "0.1.0",
  "engines": {
    "vscode": "^1.85.0"
  },
  "activationEvents": ["onStartupFinished"],
  "contributes": ["commands", "configuration"]
}
```

## Claim Boundary

Safe claim:

```text
Onus includes a VS Code L1 BEST-EFFORT extension surface that can evaluate
configured terminal/task-related flows and show status/log UI.
```

Unsafe claims:

```text
Onus universally controls VS Code Agents.
Onus intercepts all GitHub Copilot or VS Code agent tools before execution.
Onus provides L2/L3/L4 enforcement through the VS Code extension.
```

Use MCP gateway or L3 workspace routing for stronger enforcement where the
agent supports those routes.
