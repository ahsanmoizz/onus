import Link from 'next/link';

export default function IntegrationsPage() {
  return (
    <div className="min-h-screen bg-black">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-sm border-b border-zinc-800">
        <div className="max-w-6xl mx-auto px-4 h-14 flex items-center justify-between">
          <Link href="/" className="text-white font-bold text-lg">Onus</Link>
          <div className="flex items-center gap-6 text-sm text-zinc-400">
            <Link href="/product" className="hover:text-white transition-colors">Product</Link>
            <Link href="/install" className="hover:text-white transition-colors">Install</Link>
            <Link href="/docs" className="text-accent transition-colors">Docs</Link>
          </div>
        </div>
      </nav>

      <main className="pt-20 pb-16 px-4 max-w-4xl mx-auto">
        <Link href="/docs" className="text-sm text-accent hover:underline mb-6 inline-block">&larr; Back to Docs</Link>

        <h1 className="text-3xl font-bold text-white mb-4">Integrations</h1>

        <p className="text-zinc-300 leading-relaxed mb-8">
          Onus integrates with five major AI agent surfaces, providing governance across different
          development environments and workflows. Each integration has a tested setup path and a
          status indicator reported by <code className="text-accent">onus doctor</code>. The <code className="text-accent">onus integration setup</code> command
          streamlines configuration for each integration type.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Claude Code CLI</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Integration with Claude Code CLI is achieved through an MCP hook that routes tool calls through
          the Onus governance layer. The Onus doctor command validates the Claude Code installation and
          configuration. Setup is automated via:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
onus integration setup claude-code</pre>
        <p className="text-zinc-300 leading-relaxed mb-4">
          The integration provides full governance over Claude Code agent sessions, including rule
          evaluation, approval workflows, and audit logging.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">OpenAI Codex CLI</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          OpenAI Codex CLI integrates with Onus through the MCP proxy. The proxy intercepts Codex&apos;s
          tool calls and applies governance before forwarding them. Setup is available through the
          integration command:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
onus integration setup codex-cli</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Google Antigravity</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Google Antigravity agents connect to Onus through the MCP protocol. The MCP proxy serves as
          the governance layer for Antigravity agent sessions, providing evaluation, logging, and audit
          capabilities. Setup is configured via:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
onus integration setup antigravity</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">VS Code Extension</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          The Onus VS Code extension provides in-editor governance for AI coding assistants running
          inside VS Code. Note that the VS Code integration is currently <strong className="text-white">partial</strong>. Refer to the
          STATUS section in the integration documentation for details on which features are available
          and what is planned. Setup:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
onus integration setup vscode</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Cursor IDE</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Cursor IDE integrates with Onus through an MCP hook, similar to Claude Code. The hook ensures
          that all Cursor agent tool calls are routed through the Onus governance layer. Setup:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
onus integration setup cursor</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Verifying Integration Status</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          The <code className="text-accent">onus doctor</code> command checks all configured integrations and reports their status.
          Each integration is tested to verify that the connection is working, the MCP proxy is
          reachable, and governance is active:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
# Check all integration status
onus doctor

# Check a specific integration
onus doctor --integration claude-code</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Integration Status Indicators</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Each integration has a status indicator that shows:
        </p>
        <ul className="list-disc pl-6 text-zinc-300 space-y-2 mb-8">
          <li><strong className="text-green-400">Connected</strong> &mdash; integration is configured and active.</li>
          <li><strong className="text-yellow-400">Partial</strong> &mdash; integration is configured but some features may be unavailable (see VS Code).</li>
          <li><strong className="text-red-400">Not detected</strong> &mdash; the target application is not installed or Onus cannot connect.</li>
          <li><strong className="text-zinc-500">Not configured</strong> &mdash; integration has not been set up yet.</li>
        </ul>
      </main>
    </div>
  );
}
