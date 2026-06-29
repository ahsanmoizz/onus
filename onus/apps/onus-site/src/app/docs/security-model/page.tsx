import Link from 'next/link';

export default function SecurityModelPage() {
  return (
    <>
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
        <Link href="/docs" className="text-sm text-zinc-400 hover:text-white transition-colors inline-flex items-center gap-1 mb-8">&larr; Back to Docs</Link>
        <h1 className="text-3xl font-bold text-white mb-4">Security Model</h1>
        <p className="text-zinc-300 mb-8">Onus protects AI agent interactions through a layered security model spanning four enforcement levels.</p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Assets Protected</h2>
        <ul className="list-disc list-inside space-y-2 text-zinc-300 mb-6">
          <li>Source code and intellectual property</li>
          <li>Credentials and secrets (API keys, tokens, certificates)</li>
          <li>Production infrastructure and configuration</li>
          <li>Data integrity (codebases, databases, artifacts)</li>
          <li>AI agent integrity (preventing prompt injection, tool abuse)</li>
        </ul>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Trust Boundaries</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">User &harr; Onus</h3>
            <p className="text-sm text-zinc-400">Onus authenticates user identity via environment and token. The user trusts Onus to enforce governance.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Onus &harr; Provider</h3>
            <p className="text-sm text-zinc-400">Semantic evaluation requests are sent to the configured provider (cloud or local). Data may leave the machine for cloud providers.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Onus &harr; Daemon</h3>
            <p className="text-sm text-zinc-400">The daemon runs locally and communicates via IPC. The web dashboard connects via HTTP with token authentication.</p>
          </div>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Enforcement Levels</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">L1 — Best-effort Hook</h3>
            <p className="text-sm text-zinc-400">Cooperative pre-tool hook in the agent environment. Labeled BEST-EFFORT. Least strict, easiest to bypass.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">L2 — Onus-routed Actions</h3>
            <p className="text-sm text-zinc-400">Actions routed through the Onus MCP proxy. All tool calls are intercepted and evaluated. Applies only to actions routed through Onus.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">L3 — Process/FS/Net Containment</h3>
            <p className="text-sm text-zinc-400">Containerized execution with filesystem, network, and resource isolation. Requires real container technology (Docker/podman on Linux).</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">L4 — Controlled Authority</h3>
            <p className="text-sm text-zinc-400">Onus manages disposable credentials with short TTLs and exact scope binding. Strongest level. Requires Onus-controlled authority.</p>
          </div>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Security Invariants</h2>
        <ul className="list-disc list-inside space-y-2 text-zinc-300 mb-6">
          <li>Deterministic denial cannot be overridden by an LLM</li>
          <li>Critical evaluator failure must not silently fail open</li>
          <li>Secrets must not appear in logs, receipts, prompts, or dashboard responses</li>
          <li>Approval binds to the exact canonical action payload</li>
          <li>Modified payloads require new approval</li>
          <li>Production actions require verified environment identity</li>
        </ul>
      </main>
    </>
  );
}
