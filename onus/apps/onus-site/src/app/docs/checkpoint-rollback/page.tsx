import Link from 'next/link';

export default function CheckpointRollbackPage() {
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
        <h1 className="text-3xl font-bold text-white mb-4">Checkpoints &amp; Rollback</h1>
        <p className="text-zinc-300 mb-8">Workspace snapshots and precise undo capabilities for governed actions.</p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Checkpoints</h2>
        <p className="text-zinc-300 mb-4">Create workspace snapshots at any point during a session:</p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto mb-4">onus checkpoint create --name &quot;before-refactor&quot;</pre>
        <p className="text-zinc-300 mb-4">List checkpoints: <code className="text-accent bg-zinc-900 px-1 rounded">onus checkpoint list</code></p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Rollback Modes</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Action Rollback</h3>
            <p className="text-sm text-zinc-400">Undo a single action by ID. Fastest, most targeted rollback.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Group Rollback</h3>
            <p className="text-sm text-zinc-400">Undo a related group of actions (e.g., all edits to a file).</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Session Rollback</h3>
            <p className="text-sm text-zinc-400">Roll back an entire session to its starting state.</p>
          </div>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Usage</h2>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto mb-4">onus rollback --id &lt;id&gt; --mode action --reason &quot;reverted breaking change&quot;</pre>
        <p className="text-zinc-300">A reason is required for every rollback, recorded in the audit trail.</p>
      </main>
    </>
  );
}
