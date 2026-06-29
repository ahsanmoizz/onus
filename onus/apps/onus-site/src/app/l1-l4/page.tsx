'use client';

import Link from 'next/link';
import { Shield, ArrowRight, CheckCircle, AlertTriangle } from 'lucide-react';

const levels = [
  {
    level: 'L1', name: 'Prompt / Contractual', tag: 'BEST-EFFORT',
    desc: 'Best-effort cooperative hooks. Onus provides hooks that cooperating agents can call, but enforcement depends on the agent honoring the protocol.',
    features: ['Shell wrapper command interception', 'VS Code extension hooks', 'Prompt Intake Guardian for task definition'],
    note: 'Cooperative hooks are labeled BEST-EFFORT. They cannot enforce if the agent bypasses them.',
  },
  {
    level: 'L2', name: 'Deterministic Enforcement', tag: 'ENFORCED',
    desc: 'Actions are intercepted and evaluated by the Onus rules engine before execution. This is the core enforcement level for supported agents.',
    features: ['Always-block rules for destructive patterns', 'Path and command pattern matching', 'Secret and credential detection', 'Human approval escalation'],
    note: 'Applies only to actions explicitly routed through Onus.',
  },
  {
    level: 'L3', name: 'Workspace Isolation', tag: 'CONTAINMENT',
    desc: 'Agent processes run in isolated workspaces with filesystem, network, and credential containment via bubblewrap (Linux) or equivalent.',
    features: ['Filesystem-level isolation', 'Network access control', 'Credential sandboxing', 'Process containment'],
    note: 'Requires bubblewrap on Linux. Limited on Windows.',
  },
  {
    level: 'L4', name: 'Controlled Authority', tag: 'CONTROLLED',
    desc: 'Onus manages the agent\'s identity and credentials. Actions requiring elevated authority are gated through the Onus authority system.',
    features: ['Credential-controlled execution', 'Lease-based resource access', 'Audited privilege elevation', 'Authority verification receipts'],
    note: 'Requires Onus-controlled credentials. Preview stage.',
  },
];

export default function L1L4Page() {
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
        <div className="max-w-5xl mx-auto">
          <div className="text-center mb-12">
            <h1 className="text-4xl font-bold text-white mb-4">L1–L4 Security Model</h1>
            <p className="text-zinc-400 max-w-2xl mx-auto">
              Onus enforces governance at four progressive levels. Each level adds stronger guarantees with increasing infrastructure requirements.
            </p>
          </div>

          <div className="space-y-6">
            {levels.map((item, i) => (
              <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
                <div className="flex items-start gap-4">
                  <div className="w-16 h-16 rounded-xl bg-accent/10 flex items-center justify-center flex-shrink-0">
                    <span className="text-2xl font-bold text-accent">{item.level}</span>
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-2">
                      <h2 className="text-lg font-semibold text-white">{item.name}</h2>
                      <span className="text-xs px-2 py-0.5 rounded-full bg-accent/10 text-accent border border-accent/20">{item.tag}</span>
                    </div>
                    <p className="text-sm text-zinc-400 mb-4">{item.desc}</p>
                    <ul className="space-y-1.5 mb-3">
                      {item.features.map((f, j) => (
                        <li key={j} className="flex items-center gap-2 text-sm text-zinc-300">
                          <CheckCircle className="w-3.5 h-3.5 text-accent flex-shrink-0" />{f}
                        </li>
                      ))}
                    </ul>
                    <div className="flex items-start gap-2 text-xs text-zinc-500 bg-zinc-800/50 rounded-lg p-3">
                      <AlertTriangle className="w-3 h-3 text-yellow-500 mt-0.5 flex-shrink-0" />{item.note}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <footer className="border-t border-zinc-800 py-8 px-4">
        <div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus — AI Agent Firewall. Open source (MIT).</div>
      </footer>
    </div>
  );
}
