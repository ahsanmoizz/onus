import Link from 'next/link';

export default function ApprovalsPage() {
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
        <h1 className="text-3xl font-bold text-white mb-4">Human Approvals</h1>
        <p className="text-zinc-300 mb-8">Human-in-the-loop approval workflow. Actions that exceed risk thresholds trigger approval requests that bind to the exact canonical action payload.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">How Approvals Work</h2>
        <p className="text-zinc-300 mb-4">When an action is evaluated and exceeds the configurable risk threshold (MEDIUM by default), Onus creates an approval request. The action is paused until a human approves or denies it.</p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto mb-4">onus approvals list</pre>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Approval Binding</h2>
        <p className="text-zinc-300 mb-4">Each approval is cryptographically bound to the exact action payload. Any modification to the payload invalidates the approval and requires re-approval. This prevents approval bypass through payload tampering.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">CLI Commands</h2>
        <div className="space-y-2 mb-6">
          <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-3 text-sm text-zinc-300 font-mono">onus approvals list                # List pending approvals</pre>
          <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-3 text-sm text-zinc-300 font-mono">onus approvals serve --token TOKEN # Serve local approval UI</pre>
          <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-3 text-sm text-zinc-300 font-mono">onus approvals approve &lt;action-id&gt; # Approve a pending request</pre>
          <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-3 text-sm text-zinc-300 font-mono">onus approvals deny &lt;action-id&gt;    # Deny a pending request</pre>
        </div>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Web Console</h2>
        <p className="text-zinc-300">The web console at /approvals provides a visual interface for managing approvals with approve/deny buttons, filter tabs (pending/approved/rejected/all), and full action context.</p>
      </main>
    </>
  );
}
