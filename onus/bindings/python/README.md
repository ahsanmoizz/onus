# Onus Python SDK

Python bindings for the [Onus](https://github.com/ahsanmoizz/onus) AI Agent Firewall.

```python
from onus import OnusClient

client = OnusClient()
verdict = client.evaluate("shell", "rm -rf /")
print(verdict.decision)  # "block"
```

## Installation

```bash
pip install onus
```

Requires the `onus` binary on your PATH. Install from:
https://github.com/ahsanmoizz/onus/releases

## Quick Start

```python
from onus import OnusClient

# Create client (auto-finds onus binary)
client = OnusClient()

# Start a session for tracking
with client.session("my task"):
    # Evaluate actions — returns Verdict
    result = client.evaluate("shell", "ls -la")
    print(result.decision)  # "allow"

    # Dangerous commands are blocked
    result = client.evaluate("shell", "sudo rm -rf /etc")
    print(result.decision)  # "block" or "escalate"
    print(result.correction)  # Correction message
    print(result.rule_id)  # e.g. "SAFETY_001"
```

## API

### `OnusClient(bin_path=None)`

- `bin_path`: Path to `onus` binary. Defaults to `onus` on PATH.

### Methods

- `evaluate(action_type, payload, session_id=None, tool=None) -> OnusResult`
  - Sync evaluate an action through Onus Core.

- `session(task_description) -> SessionContext`
  - Context manager for a tracked session.

- `check_command(command, session_id=None) -> OnusResult`
  - Convenience: evaluate a shell command.

- `install_shell_wrapper(path=None)`
  - Install the shell wrapper for terminal interception.

- `remove_shell_wrapper()`
  - Remove the shell wrapper.

- `load_rules() -> list[dict]`
  - List current policy rules.

### `OnusResult`

- `decision`: `"allow"` | `"warn"` | `"block"` | `"escalate"`
- `allowed`: bool (decision is allow or warn)
- `blocked`: bool (decision is block or escalate)
- `correction`: Optional[str]
- `rule_id`: Optional[str]
- `latency_us`: Optional[int]
