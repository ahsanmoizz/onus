import Link from 'next/link';

export default function MemoryPage() {
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
        <h1 className="text-3xl font-bold text-white mb-4">Memory System</h1>
        <p className="text-zinc-300 mb-8">Onus provides a structured memory system for maintaining agent context across actions and sessions.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Memory Scopes</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Session-scoped Memory</h3>
            <p className="text-sm text-zinc-400">Tied to a single session. Automatically cleaned when the session ends. Best for temporary context like current task focus.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Project-scoped Memory</h3>
            <p className="text-sm text-zinc-400">Persists across sessions within a project. Useful for project conventions, architecture decisions, and learned patterns.</p>
          </div>
        </div>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Lifecycle Operations</h2>
        <div className="grid grid-cols-2 gap-3 mb-6">
          <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-3 text-sm text-zinc-300 font-mono">onus memory list</pre>
          <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-3 text-sm text-zinc-300 font-mono">onus memory inspect &lt;id&gt;</pre>
          <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-3 text-sm text-zinc-300 font-mono">onus memory export</pre>
          <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-3 text-sm text-zinc-300 font-mono">onus memory delete &lt;id&gt;</pre>
        </div>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Retention</h2>
        <p className="text-zinc-300 mb-4">Configurable TTL per scope. Session memory defaults to 24h, project memory to 30 days. Incident-tagged entries have separate retention rules.</p>
        <p className="text-zinc-300">Memory is distinct from the audit log. Memory stores agent context; the audit log stores action receipts.</p>
      </main>
    </>
  );
}
