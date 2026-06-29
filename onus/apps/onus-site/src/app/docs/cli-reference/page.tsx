import Link from 'next/link';

const groups = [
  {
    title: 'Start, Stop, Status',
    commands: [
      ['onus doctor', 'Run system diagnostics. Exits non-zero when a surface is not ready.'],
      ['onus start [--foreground]', 'Start the daemon. Convenience alias for `onus daemon start`.'],
      ['onus stop', 'Stop the daemon. Convenience alias for `onus daemon stop`.'],
      ['onus restart', 'Restart the daemon.'],
      ['onus status', 'Show daemon and session status.'],
      ['onus console --port 3001 --token <token>', 'Serve the local admin console. Alias for dashboard serving.'],
    ],
  },
  {
    title: 'Task Intake and Contracts',
    commands: [
      ['onus intake --prompt "<task>" --provider disabled', 'Classify a prompt and propose a safe task contract when possible.'],
      ['onus contract create', 'Create and manage task contracts. Run `onus contract --help` for subcommands.'],
      ['onus run -- <command>', 'Run a command through Onus evaluation. Use `--isolate` only with a verified Linux workspace.'],
      ['onus evaluate', 'Evaluate one action JSON payload and return a verdict. Used by hooks and adapters.'],
    ],
  },
  {
    title: 'Integrations',
    commands: [
      ['onus setup --claude', 'Install/configure Claude Code cooperative hook. L1 BEST-EFFORT.'],
      ['onus setup --codex', 'Configure Codex MCP routing where supported. L2 only for routed calls.'],
      ['onus setup --cursor', 'Configure Cursor hooks/MCP routing where supported.'],
      ['onus setup --antigravity', 'Configure Antigravity extension/MCP routing where supported.'],
      ['onus setup --vscode', 'Configure VS Code integration where supported.'],
      ['onus mcp-proxy --experimental --server <cmd> -- <args>', 'Run the Onus MCP gateway around an MCP server.'],
      ['onus claude-hook', 'Claude Code PreToolUse hook entry point.'],
      ['onus cursor-hook', 'Cursor hook entry point.'],
      ['onus shell', 'Install or remove terminal shell wrapper integration.'],
    ],
  },
  {
    title: 'Audit, Receipts, Approvals',
    commands: [
      ['onus log --limit 20', 'View audit trail rows.'],
      ['onus session <id>', 'View a specific session summary.'],
      ['onus verify', 'Verify local hash-chain integrity. This is tamper-evident, not immutable.'],
      ['onus approvals list', 'List pending or resolved approvals.'],
      ['onus approvals serve --port 9191 --token <token>', 'Serve the local approval UI.'],
      ['onus approvals approve <action-id>', 'Approve a pending exact action.'],
      ['onus approvals deny <action-id>', 'Deny a pending action.'],
    ],
  },
  {
    title: 'Rollback, Workspace, Authority',
    commands: [
      ['onus checkpoint create --name <name>', 'Create a workspace checkpoint.'],
      ['onus checkpoint list', 'List checkpoints.'],
      ['onus rollback action <action-id>', 'Rollback one supported action. Run `onus rollback --help` for exact syntax.'],
      ['onus compensation inspect <action-id>', 'Inspect mitigation/compensation for supported action classes.'],
      ['onus workspace create', 'Create a Linux L3 workspace when supported.'],
      ['onus workspace inspect', 'Inspect a workspace.'],
      ['onus workspace export', 'Export controlled artifacts from a workspace.'],
      ['onus workspace destroy', 'Destroy a workspace.'],
      ['onus authority', 'Experimental narrow L4 authority proof commands. Do not use for production without independent verification.'],
    ],
  },
  {
    title: 'Policy and Lifecycle',
    commands: [
      ['onus rules', 'Manage safety rules.'],
      ['onus memory', 'Manage scoped memory lifecycle operations.'],
      ['onus handoff', 'Create/import/display cross-agent handoff manifests.'],
      ['onus lease', 'Acquire/release/heartbeat session leases.'],
      ['onus upgrade', 'Download and install the latest Onus version.'],
      ['onus uninstall [--purge]', 'Remove Onus configuration. `--purge` deletes retained local data.'],
    ],
  },
];

export default function CliReferencePage() {
  return (
    <div className="min-h-screen bg-black text-zinc-100">
      <nav className="fixed inset-x-0 top-0 z-50 border-b border-zinc-800 bg-black/85 backdrop-blur-sm">
        <div className="mx-auto flex h-14 max-w-6xl items-center justify-between px-4">
          <Link href="/" className="text-lg font-bold text-white">Onus</Link>
          <div className="flex items-center gap-6 text-sm text-zinc-400">
            <Link href="/install" className="hover:text-white">Install</Link>
            <Link href="/admin" className="hover:text-white">Admin</Link>
            <Link href="/docs" className="text-accent">Docs</Link>
          </div>
        </div>
      </nav>

      <main className="mx-auto max-w-5xl px-4 pb-16 pt-20">
        <Link href="/docs" className="mb-8 inline-flex items-center gap-1 text-sm text-zinc-400 hover:text-white">&larr; Back to Docs</Link>
        <h1 className="mb-4 text-3xl font-bold text-white">CLI Reference</h1>
        <p className="mb-8 max-w-3xl leading-7 text-zinc-400">
          These are the top-level commands exposed by the current `onus` binary. For exact subcommand flags,
          run `onus command --help` against the installed version.
        </p>

        <div className="space-y-8">
          {groups.map((group) => (
            <section key={group.title}>
              <h2 className="mb-3 text-xl font-semibold text-white">{group.title}</h2>
              <div className="space-y-3">
                {group.commands.map(([command, description]) => (
                  <div key={command} className="rounded-lg border border-zinc-800 bg-zinc-900/45 p-4">
                    <code className="block overflow-x-auto pb-2 font-mono text-sm text-accent">{command}</code>
                    <p className="text-sm leading-6 text-zinc-400">{description}</p>
                  </div>
                ))}
              </div>
            </section>
          ))}
        </div>

        <section className="mt-10 rounded-lg border border-zinc-800 bg-zinc-950 p-5">
          <h2 className="mb-2 text-lg font-semibold text-white">Safe claims while using the CLI</h2>
          <p className="text-sm leading-6 text-zinc-500">
            L1 integrations are cooperative and BEST-EFFORT. L2 applies only to actions routed through Onus.
            L3 requires Linux process/filesystem/network containment. L4 requires Onus-controlled authority or credentials.
          </p>
        </section>
      </main>
    </div>
  );
}
