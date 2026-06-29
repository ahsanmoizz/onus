'use client';

import Link from 'next/link';
import { ArrowRight, CheckCircle2, Terminal } from 'lucide-react';

const workflow = [
  ['Start Onus', 'onus start'],
  ['Open local console', 'onus console --port 3001'],
  ['Classify a request before execution', 'onus intake --prompt "..." --provider disabled'],
  ['Run a routed command', 'onus run -- cargo test'],
  ['Review the audit trail', 'onus log --limit 20'],
  ['Verify receipts', 'onus verify'],
];

const groups = [
  'Lifecycle: start, stop, restart, status, doctor, console',
  'Governance: intake, contract, evaluate, run, approvals',
  'Evidence: log, verify, checkpoint, rollback, compensation',
  'Integrations: setup, mcp-proxy, claude-hook, cursor-hook, shell',
  'Policy and memory: rules, memory, handoff, lease',
  'Advanced boundaries: workspace and authority, where runtime prerequisites are available',
];

export default function CliPage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-zinc-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <Link href="/" className="flex items-center gap-2">
              <div className="w-7 h-7 rounded-full bg-accent flex items-center justify-center"><span className="text-black text-xs font-bold">O</span></div>
              <span className="font-bold text-white text-lg">Onus</span>
            </Link>
            <Link href="/docs" className="text-sm text-zinc-400 hover:text-white transition-colors">Docs</Link>
          </div>
        </div>
      </nav>

      <main className="pt-24 pb-20 px-4">
        <div className="max-w-5xl mx-auto">
          <div className="flex items-center gap-3 mb-6">
            <Terminal className="w-8 h-8 text-accent" />
            <h1 className="text-3xl font-bold text-white">CLI Guide</h1>
          </div>
          <p className="text-zinc-400 max-w-3xl mb-8">
            The Onus CLI is the operator entry point for starting the daemon, routing agent actions, reviewing approvals, verifying receipts, and opening the local admin console.
          </p>

          <div className="grid md:grid-cols-2 gap-4 mb-10">
            {workflow.map(([label, command]) => (
              <div key={label} className="rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
                <div className="flex items-center gap-2 text-sm font-semibold text-white mb-3">
                  <CheckCircle2 className="w-4 h-4 text-accent" />
                  {label}
                </div>
                <code className="block rounded-lg bg-black px-3 py-2 text-xs text-zinc-300 overflow-x-auto">{command}</code>
              </div>
            ))}
          </div>

          <section className="rounded-xl border border-zinc-800 bg-zinc-900/50 p-6 mb-8">
            <h2 className="text-lg font-semibold text-white mb-4">Command Groups</h2>
            <div className="space-y-3">
              {groups.map((item) => (
                <div key={item} className="flex gap-3 text-sm text-zinc-400">
                  <ArrowRight className="w-4 h-4 text-accent shrink-0 mt-0.5" />
                  <span>{item}</span>
                </div>
              ))}
            </div>
          </section>

          <section className="rounded-xl border border-accent/30 bg-accent/10 p-6">
            <h2 className="text-lg font-semibold text-white mb-3">Exact Reference</h2>
            <p className="text-sm text-zinc-300 mb-4">
              Use the generated CLI reference for command names and examples. For the installed binary, the authoritative source is always <code className="rounded bg-black px-1.5 py-0.5 text-accent">onus --help</code> and <code className="rounded bg-black px-1.5 py-0.5 text-accent">onus command --help</code>.
            </p>
            <Link href="/docs/cli-reference" className="inline-flex items-center gap-2 rounded-full bg-accent px-5 py-2 text-sm font-semibold text-black hover:bg-accent/90 transition-colors">
              Open CLI reference <ArrowRight className="w-4 h-4" />
            </Link>
          </section>
        </div>
      </main>

      <footer className="border-t border-zinc-800 py-8 px-4">
        <div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus - AI Agent Firewall. Open source (MIT).</div>
      </footer>
    </div>
  );
}
