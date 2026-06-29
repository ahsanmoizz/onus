import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function AgentHandoffPage() {
  return (
    <>
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
        <Link href="/docs" className="text-sm text-zinc-400 hover:text-white transition-colors inline-flex items-center gap-1 mb-8">&larr; Back to Docs</Link>
        <h1 className="text-3xl font-bold text-white mb-4">Agent Handoff</h1>
        <p className="text-zinc-300 mb-8">Cross-agent continuity enables transferring session state between different AI agents while preserving governance context.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Overview</h2>
        <p className="text-zinc-300 mb-4">Onus can serialize an active session&apos;s state — including action history, contract state, pending approvals, and memory — into a portable bundle. This bundle can be imported by a different agent to resume work seamlessly.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Exporting a Session</h2>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto mb-4">onus session export --format json --output session-bundle.json</pre>
        <p className="text-zinc-300 mb-4">The export includes: action history with verdicts, active contracts, pending approvals, memory entries, and a cryptographic signature for integrity verification.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Importing a Session</h2>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto mb-4">onus session import session-bundle.json</pre>
        <p className="text-zinc-300 mb-4">Import restores the full session state on the target agent. The session ID is preserved, maintaining audit trail continuity.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Use Cases</h2>
        <ul className="list-disc list-inside space-y-2 text-zinc-300 mb-6">
          <li>Switch between Claude Code and OpenAI Codex mid-task</li>
          <li>Escalate a complex task to a more capable model</li>
          <li>Share session context across team members</li>
          <li>Backup and restore in-progress work</li>
        </ul>
      </main>
    </>
  );
}
