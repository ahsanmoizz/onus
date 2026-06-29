'use client';

import Link from 'next/link';
import { GitBranch, ArrowRight, CheckCircle, AlertTriangle } from 'lucide-react';

export default function WorkspacesPage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-zinc-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <Link href="/" className="flex items-center gap-2">
              <div className="w-7 h-7 rounded-full bg-accent flex items-center justify-center">
                <span className="text-black text-xs font-bold">O</span>
              </div>
              <span className="font-bold text-white text-lg">Onus</span>
            </Link>
          </div>
        </div>
      </nav>

      <div className="pt-24 pb-20 px-4">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-4xl font-bold text-white text-center mb-4">Workspaces</h1>
          <p className="text-zinc-400 text-center max-w-2xl mx-auto mb-12">
            Containerized workspaces for AI agent operations with filesystem, network, and credential isolation (L3).
          </p>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
            {[
              { title: 'Isolated Execution', desc: 'Agent processes run in isolated environments with contained filesystem access' },
              { title: 'Bubblewrap Sandbox', desc: 'Linux L3 workspaces use bubblewrap for lightweight OS-level virtualization' },
              { title: 'Clean State', desc: 'Each workspace starts from a clean baseline with controlled external access' },
            ].map((item, i) => (
              <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5">
                <h3 className="font-semibold text-white text-sm mb-2">{item.title}</h3>
                <p className="text-xs text-zinc-400">{item.desc}</p>
              </div>
            ))}
          </div>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <h2 className="text-lg font-semibold text-white mb-4">Workspace Management</h2>
            <ul className="space-y-2 text-sm text-zinc-300">
              {['Create isolated workspaces for each agent session', 'Configure filesystem mounts and permissions', 'Set network access policies (allow/block specific endpoints)', 'Workspace lifecycle managed through Onus CLI and Console', 'Clean teardown after session completion'].map((item, i) => (
                <li key={i} className="flex items-start gap-2"><CheckCircle className="w-3.5 h-3.5 text-accent mt-0.5 flex-shrink-0" />{item}</li>
              ))}
            </ul>
          </div>

          <div className="mt-6 flex items-start gap-2 text-xs text-zinc-500 bg-zinc-800/50 rounded-lg p-3">
            <AlertTriangle className="w-3 h-3 text-yellow-500 mt-0.5 flex-shrink-0" />
            L3 workspace isolation requires bubblewrap on Linux. Limited support on Windows.
          </div>
        </div>
      </div>

      <footer className="border-t border-zinc-800 py-8 px-4">
        <div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus — AI Agent Firewall. Open source (MIT).</div>
      </footer>
    </div>
  );
}
