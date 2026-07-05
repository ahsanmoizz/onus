import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function TaskContractsPage() {
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
        <h1 className="text-3xl font-bold text-white mb-4">Task Contracts</h1>
        <p className="text-zinc-300 mb-8">Every governed action is bound by a contract defining exactly what is permitted, what evidence is required, and what limits apply.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Contract Components</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Action List</h3>
            <p className="text-sm text-zinc-400">Each permitted action with type, target, and expected outcome. Out-of-scope actions require escalation.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Risk Assessment</h3>
            <p className="text-sm text-zinc-400">NONE, LOW, MEDIUM, HIGH, or CRITICAL. Determines auto-allow vs human approval threshold.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Change Budget</h3>
            <p className="text-sm text-zinc-400">Max files, lines, shell commands, and network egress. Exceeding requires renegotiation.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Required Evidence</h3>
            <p className="text-sm text-zinc-400">Test results, lint output, type checks, review confirmation — attached per action.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Permissions</h3>
            <p className="text-sm text-zinc-400">File access globs, shell allowlist, network targets, credential scope.</p>
          </div>
        </div>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Contract Lifecycle</h2>
        <ol className="list-decimal list-inside space-y-2 text-zinc-300 mb-6">
          <li><strong>Generate:</strong> Created by Prompt Intake Guardian</li>
          <li><strong>Review:</strong> User reviews before accepting</li>
          <li><strong>Activate:</strong> Bound to session; agent works within scope</li>
          <li><strong>Enforce:</strong> Each action checked against contract</li>
          <li><strong>Complete:</strong> All actions executed and evidenced</li>
          <li><strong>Archive:</strong> Contract + audit trail stored in receipt chain</li>
        </ol>
      </main>
    </>
  );
}
