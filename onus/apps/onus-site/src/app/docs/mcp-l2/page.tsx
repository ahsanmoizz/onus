import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function McpL2Page() {
  return (
    <div className="min-h-screen bg-black">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-sm border-b border-zinc-800">
        <div className="max-w-6xl mx-auto px-4 h-14 flex items-center justify-between">
          <Link href="/" className="inline-flex items-center" aria-label="Onus home"><BrandLogo imageClassName="h-9 w-auto" /></Link>
          <div className="flex items-center gap-6 text-sm text-zinc-400">
            <Link href="/product" className="hover:text-white transition-colors">Product</Link>
            <Link href="/install" className="hover:text-white transition-colors">Install</Link>
            <Link href="/docs" className="text-accent transition-colors">Docs</Link>
          </div>
        </div>
      </nav>

      <main className="pt-20 pb-16 px-4 max-w-4xl mx-auto">
        <Link href="/docs" className="text-sm text-accent hover:underline mb-6 inline-block">&larr; Back to Docs</Link>

        <h1 className="text-3xl font-bold text-white mb-4">MCP L2: Model Context Protocol Proxy</h1>

        <p className="text-zinc-300 leading-relaxed mb-8">
          Level 2 enforcement routes AI agent tool calls through the Onus Model Context Protocol (MCP)
          proxy. Every tool invocation is intercepted, evaluated against rules and policies, and only
          forwarded to the target MCP server if it passes governance checks. L2 guarantees that all
          actions routed through Onus are governed, inspected, and audited.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">How MCP Routing Works</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          In a standard MCP setup, the agent communicates directly with MCP servers to invoke tools.
          Onus inserts itself as a proxy between the agent and its MCP servers, creating a governance
          layer that evaluates every call before forwarding.
        </p>

        <p className="text-zinc-300 leading-relaxed mb-4">The request flow is:</p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
Agent --&gt; MCP Client --&gt; Onus Proxy --&gt; Actual MCP Server</pre>
        <p className="text-zinc-300 leading-relaxed mb-4">
          At each step, Onus evaluates the action against installed rules. If a rule denies the action,
          the proxy returns a rejection to the agent without ever forwarding the request to the MCP server.
          This prevents potentially destructive or unauthorized tool calls from reaching their targets.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Supported Clients</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Onus&apos;s MCP proxy works with any MCP-compatible tool or client, including:
        </p>
        <ul className="list-disc pl-6 text-zinc-300 space-y-2 mb-6">
          <li><strong className="text-white">Claude Code CLI</strong> &mdash; configured via MCP hook in the Claude Code configuration.</li>
          <li><strong className="text-white">Cursor IDE</strong> &mdash; MCP integration through Cursor&apos;s MCP settings.</li>
          <li><strong className="text-white">OpenAI Codex CLI</strong> &mdash; proxy integration for governed agent sessions.</li>
          <li><strong className="text-white">Google Antigravity</strong> &mdash; MCP-based integration point.</li>
          <li><strong className="text-white">Any MCP-compatible tool</strong> &mdash; the proxy speaks standard MCP protocol.</li>
        </ul>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Setting Up the Proxy</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Start the Onus MCP proxy with the following command:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
onus mcp proxy --port 8080</pre>
        <p className="text-zinc-300 leading-relaxed mb-4">
          The proxy listens on the specified port and accepts MCP connections. You can configure your
          MCP client to point to the proxy address instead of the MCP server directly. The proxy
          forwards all standard MCP messages while interposing governance checks.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Evaluation Before Forwarding</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Each tool call is evaluated in real-time before being forwarded to the MCP server. The
          evaluation includes both Tier 1 deterministic rule matching and Tier 2 semantic analysis
          (if enabled). If the action passes all checks, it is forwarded and the response is relayed
          back to the agent. If denied, the agent receives a governed rejection message.
        </p>

        <h2 className="tex-xl font-semibold text-white mt-10 mb-3">L2 Claim</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Level 2 enforcement claims that actions routed through Onus are governed. This means every
          tool call that passes through the proxy is evaluated, logged, and auditable. However, L2
          does not provide containment guarantees&mdash;those are handled by L3 workspaces. L2 is a
          governance layer, not a sandbox.
        </p>
        <p className="text-zinc-300 leading-relaxed mb-8">
          If an agent can bypass the proxy (e.g., by connecting directly to an MCP server), those
          actions are not governed. Proper configuration of your agent&apos;s MCP settings is essential
          to ensure all traffic routes through Onus.
        </p>
      </main>
    </div>
  );
}
