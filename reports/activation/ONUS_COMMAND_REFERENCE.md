# ONUS COMMAND REFERENCE

Generated: 2026-06-18
Branch: final/activation-readiness
Binary: onus/target/release/onus.exe

---

## `onus --help`

```
Usage: onus.exe [COMMAND]

Commands:
  setup         Set up and configure integrations (Claude Code, Codex, Cursor, etc.)
  doctor        Run system diagnostics for all integrations
  daemon        Start the Onus daemon (approval server, IPC, action intake)
  status        Show daemon and session status
  intake        Submit actions to the daemon for evaluation
  contract      Create, validate, or manage task contracts
  session       List active or historical sessions
  run           Run a governed command or task
  rules         Manage static rules
  limits        Manage approval limits
  approvals     Manage pending approvals
  evaluate      Evaluate an action by payload
  dashboard     Launch the Onus dashboard (CLI-based session viewer)
  memory        Manage session memory
  workspace     Create, export, or manage workspaces
  authority     Manage governance authority
  verify        Verify hash chain integrity of the audit trail
  checkpoint    Manage filesystem checkpoints
  rollback      Roll back individual actions, groups, or entire sessions
  compensation  Inspect or execute compensation for previously evaluated actions
  log           Show session log
  upgrade       Upgrade the Onus daemon
  uninstall     Uninstall and clean up Onus integrations
  shell         Start an interactive governed shell
  mcp-proxy     Start the MCP proxy server
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

---

## `onus setup`

```
Usage: onus.exe setup [OPTIONS]

Options:
  --claude       Set up only the Claude Code CLI hook
  --codex        Set up only the OpenAI Codex CLI MCP proxy
  --antigravity  Set up only the Google Antigravity extension and MCP proxy
  --vscode       Set up only the VS Code extension
  --cursor       Set up only the Cursor IDE hooks and MCP proxy
  -h, --help     Print help
```

**Purpose:** Configure an agent integration so that the agent sends actions to Onus for evaluation. Each integration type installs a hook, MCP proxy, or extension depending on the agent.

**Required configuration:** Onus data directory must be initialised (created automatically on first start). Some integrations (Codex, Cursor) require an MCP config file.

**Supported operating systems:** Windows, macOS, Linux

**Expected output:** Prints success/failure messages for each setup step.

**Failure behavior:** If a required tool (e.g. Claude CLI, Cursor) is not found, prints a warning but continues. Exits with non-zero only on internal error.

**Example:**
```
onus setup --claude
onus setup --codex --cursor
```

---

## `onus doctor`

```
Usage: onus.exe doctor [OPTIONS]

Options:
  --claude       Run only Claude Code CLI checks
  --codex        Run only OpenAI Codex CLI checks
  --antigravity  Run only Google Antigravity checks
  --cursor       Run only Cursor IDE checks
  -h, --help     Print help
```

**Purpose:** Diagnose all or specific agent integrations. Reports installation status, hook/MCP configuration, and version.

**Required configuration:** None. Runs against the environment.

**Supported operating systems:** Windows, macOS, Linux

**Expected output:** Tabular diagnostics showing each integration's availability, version, hook status, and MCP config.

**Failure behavior:** Non-issues are reported as warnings, not errors. Exit code is 0 unless an internal error occurs.

**Example:**
```
onus doctor
onus doctor --claude
```

---

## `onus daemon`

```
Usage: onus.exe daemon [OPTIONS]

Options:
      --port <PORT>  Port to listen on [default: 4837]
  -h, --help         Print help
```

**Purpose:** Start the Onus daemon — the long-lived background process that receives actions from agent hooks, evaluates them against policies, manages approvals, and records receipts.

**Required configuration:** Provider must be configured via environment variables (`ONUS_SEMANTIC_PROVIDER`, etc.). Or uses Deterministic mode by default.

**Supported operating systems:** Windows, macOS, Linux

**Expected output:** Prints startup banner and listens on the configured port. Runs until interrupted (Ctrl+C).

**Failure behavior:** Fails on port conflict. Fails if data directory cannot be created.

**Example:**
```
onus daemon
onus daemon --port 4837
```

---

## `onus status`

```
Usage: onus.exe status [OPTIONS]

Options:
  -h, --help  Print help
```

**Purpose:** Show daemon and session status.

**Required configuration:** A running daemon or database.

**Supported operating systems:** Windows, macOS, Linux

**Expected output:** Displays whether the daemon is running, active sessions, and recent actions.

---

## `onus intake`

```
Usage: onus.exe intake [OPTIONS] --action <ACTION>

Options:
      --action <ACTION>        Action payload JSON string or file path with @ prefix
      --session-id <SESSION_ID>
      --env <ENV>              JSON string describing environment context
      --timeout-ms <TIMEOUT_MS>
  -h, --help                   Print help
```

**Purpose:** Submit an action to the daemon for evaluation (Allow/Deny/NeedsApproval).

---

## `onus contract`

```
Usage: onus.exe contract <COMMAND>

Commands:
  create    Create a new task contract
  validate  Validate a task contract
  list      List task contracts
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

**Purpose:** Manage task contracts — agreements that define the scope, boundaries, and required evidence for a governed task.

---

## `onus session`

```
Usage: onus.exe session <COMMAND>

Commands:
  list          List active or historical sessions
  show          Show session details
  end           End a governed session
  list-sessions List active or historical sessions
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

**Purpose:** View and manage governed sessions.

---

## `onus run`

```
Usage: onus.exe run [OPTIONS] [COMMAND]...

Arguments:
  [COMMAND]...  Command to run

Options:
      --contract <CONTRACT>  Path or JSON string for a task contract
  -h, --help                 Print help
```

**Purpose:** Run a governed command or task under Onus supervision.

---

## `onus rules`

```
Usage: onus.exe rules <COMMAND>

Commands:
  list    List static rules
  add     Add a static rule
  remove  Remove a static rule
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

**Purpose:** Manage static rules — Always Allow or Always Deny patterns.

---

## `onus limits`

```
Usage: onus.exe limits <COMMAND>

Commands:
  list    List approval limits
  set     Set an approval limit
  remove  Remove an approval limit
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

**Purpose:** Manage approval limits — thresholds that trigger approval requirements.

---

## `onus approvals`

```
Usage: onus.exe approvals <COMMAND>

Commands:
  list    List pending approvals
  allow   Approve a pending action
  deny    Deny a pending action
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

**Purpose:** Review and respond to actions requiring human approval.

---

## `onus evaluate`

```
Usage: onus.exe evaluate [OPTIONS] --payload <PAYLOAD>

Options:
      --payload <PAYLOAD>  Action payload JSON string or file path with @ prefix
      --context <CONTEXT>  Optional context string
  -h, --help               Print help
```

**Purpose:** Evaluate a single action payload and return the verdict.

---

## `onus dashboard`

```
Usage: onus.exe dashboard [OPTIONS]

Options:
      --session-id <SESSION_ID>  Filter to a specific session
  -h, --help                     Print help
```

**Purpose:** Launch an interactive CLI dashboard showing sessions, actions, approvals.

---

## `onus memory`

```
Usage: onus.exe memory <COMMAND>

Commands:
  list    List session memory
  show    Show a specific memory entry
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

**Purpose:** View session memory records.

---

## `onus workspace`

```
Usage: onus.exe workspace <COMMAND>

Commands:
  create  Create a new workspace (clone repository snapshot)
  export  Export workspace changes back to the original repository
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

**Purpose:** Create isolated sandbox workspaces for governed tasks.

---

## `onus authority`

```
Usage: onus.exe authority <COMMAND>

Commands:
  init      Initialise governance authority
  status    Show governance authority status
  register  Register a governed environment
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

**Purpose:** Manage governance authority — the root of trust for Onus enforcement.

---

## `onus verify`

```
Usage: onus.exe verify [OPTIONS]

Options:
      --db <DB>                  Path to the audit database (default: data_dir/audit.db)
      --session-id <SESSION_ID>  Optional session ID to verify only that session
  -h, --help                     Print help
```

**Purpose:** Verify the hash chain integrity of the audit trail. Checks each session's chain independently. Also verifies the cross-session anchor chain.

**Required configuration:** Onus data directory containing `audit.db`.

**Expected output:** Lists any chain integrity violations. Silent (exit 0) if all chains are valid.

**Failure behavior:** Returns list of action IDs with hash mismatches.

**Example:**
```
onus verify
onus verify --session-id sess-abc
```

---

## `onus checkpoint`

```
Usage: onus.exe checkpoint <COMMAND>

Commands:
  create    Create a new filesystem checkpoint
  list      List all checkpoints
  inspect   Inspect a specific checkpoint's manifest
  restore   Restore workspace to a checkpoint
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `onus checkpoint create`

```
Usage: onus.exe checkpoint create [OPTIONS]

Options:
      --session-id <SESSION_ID>  Governor session ID for the checkpoint
      --label <LABEL>            Optional human-readable label for the checkpoint
      --workspace <WORKSPACE>    Workspace root path (default: current directory)
  -h, --help                     Print help
```

**Purpose:** Create a filesystem checkpoint — snapshots the current state of all tracked files in the workspace and stores copies for later restoration.

### `onus checkpoint list`

```
Usage: onus.exe checkpoint list [OPTIONS]

Options:
  -h, --help  Print help
```

**Purpose:** List all checkpoints.

### `onus checkpoint inspect`

```
Usage: onus.exe checkpoint inspect <CHECKPOINT_ID>

Arguments:
  <CHECKPOINT_ID>  Checkpoint ID to inspect

Options:
  -h, --help  Print help
```

**Purpose:** Show the full checkpoint manifest — session, workspace, file entries with hashes.

### `onus checkpoint restore`

```
Usage: onus.exe checkpoint restore <CHECKPOINT_ID>

Arguments:
  <CHECKPOINT_ID>  Checkpoint ID to restore

Options:
      --workspace <WORKSPACE>  Workspace root path (default: current directory)
  -h, --help                   Print help
```

**Purpose:** Restore workspace to a checkpoint. Copies files from checkpoint storage, removes files created after the checkpoint, verifies final manifest equality. Creates a restore receipt.

---

## `onus rollback`

```
Usage: onus.exe rollback <COMMAND>

Commands:
  action   Roll back a single action
  group    Roll back a group of actions in reverse order
  session  Roll back an entire session
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

**Purpose:** Roll back actions by executing compensation logic.

---

## `onus compensation`

```
Usage: onus.exe compensation <COMMAND>

Commands:
  inspect  Inspect available compensation for an action
  execute  Execute compensation for an action
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

**Purpose:** Inspect or execute compensation for previously evaluated actions.

---

## `onus log`

```
Usage: onus.exe log <SESSION_ID>

Arguments:
  <SESSION_ID>  Session ID to show log for

Options:
  -h, --help    Print help
```

**Purpose:** Show the session log for a given session.

---

## `onus upgrade`

```
Usage: onus.exe upgrade [OPTIONS]

Options:
      --version <VERSION>  Version to upgrade to
      --source             Build from source
  -h, --help               Print help
```

**Purpose:** Upgrade the Onus daemon.

---

## `onus uninstall`

```
Usage: onus.exe uninstall [OPTIONS]

Options:
      --claude       Remove Claude Code integration
      --codex        Remove OpenAI Codex CLI MCP proxy
      --antigravity  Remove Google Antigravity extension
      --cursor       Remove Cursor IDE integration
      --vscode       Remove VS Code extension
      --all          Remove all integrations
  -h, --help         Print help
```

**Purpose:** Uninstall and clean up Onus integrations.

---

## `onus shell`

```
Usage: onus.exe shell

Options:
  -h, --help  Print help
```

**Purpose:** Start an interactive governed shell where every command is evaluated by Onus.

---

## `onus mcp-proxy`

```
Usage: onus.exe mcp-proxy [OPTIONS]

Options:
      --port <PORT>  Port for the MCP proxy server [default: 4838]
      --target <TARGET>  Target MCP server URL
  -h, --help         Print help
```

**Purpose:** Start the MCP proxy server that intercepts tool calls.

---

## `onus handoff` (Rust API only — not exposed as CLI subcommand)

The `handoff` module provides `create_handoff()` and `complete_handoff()` functions used by the `checkpoint create` workflow. It is used internally when a checkpoint is created with handoff metadata.

## `onus lease` (Rust API only — not exposed as CLI subcommand)

The `lease` module provides `acquire_lease()`, `renew_lease()`, `release_lease()`, and `heartbeat()` for coordinating exclusive workspace access across agents during handoff scenarios.
