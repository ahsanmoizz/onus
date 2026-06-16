# Visual Studio Code Agents Integration Report

Milestone surface: 4 of 20, Visual Studio Code Agents.

Branch: `integration/visual-studio-code-agents`.

Date: 2026-06-16.

## Claim

This surface is `VERIFIED WITH LIMITATIONS` for the existing VS Code extension
package and local static/runtime checks.

The integration remains L1 cooperative and `BEST-EFFORT`. It does not prove
universal pre-execution control over all VS Code agent or GitHub Copilot tool
calls.

## Official Control Surface

Official documentation reviewed:

- <https://code.visualstudio.com/docs/agents/overview>
- <https://code.visualstudio.com/api>

The VS Code extension API is the available local route. Stronger control should
prefer Onus-owned execution, MCP routing, L3 workspace isolation, or L4 authority
where the agent supports those boundaries.

## Existing Files

- `onus/bindings/vscode/package.json`
- `onus/bindings/vscode/src/extension.js`

## Files Added

- `integrations/vscode-agents/README.md`

## Runtime Evidence

VS Code version:

```text
code --version
1.124.2
6928394f91b684055b873eecb8bc281365131f1c
x64
```

Installed extensions probe:

```text
code --list-extensions
```

The command succeeded and returned no installed extension IDs in this
environment.

Extension JavaScript syntax:

```text
node --check D:\Onus\onus\bindings\vscode\src\extension.js
passed
```

Package metadata probe:

```text
Raw JSON.parse failed because package.json begins with a UTF-8 BOM.
Parsing after stripping the BOM succeeded.
```

Parsed package metadata:

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

Spec lock validation:

```text
python tools\spec_lock\verify_spec_lock.py
SPEC LOCK VERIFICATION PASSED
```

## Security Notes

- The extension must remain labeled L1 `BEST-EFFORT`.
- Terminal/task observation is not the same as a mandatory agent tool gate.
- Missing binary, disabled configuration, direct agent execution, and VS Code
  agent internals outside extension control remain bypasses.
- No live VS Code agent/Copilot tool-call interception was runtime-tested.
- No L2/L3/L4 claim is supported by this extension alone.

## Final Classification

```text
VERIFIED WITH LIMITATIONS
```

The package and local VS Code runtime are present, but full VS Code Agent
tool-call governance remains unverified.
