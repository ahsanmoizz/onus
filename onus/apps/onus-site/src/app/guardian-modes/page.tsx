'use client';

import Link from 'next/link';
import { Shield, ArrowRight, CheckCircle, AlertTriangle, Lock, Users } from 'lucide-react';

const modes = [
  {
    name: 'Beginner',
    tag: 'For evaluation and learning',
    icon: Shield,
    color: 'text-green-400',
    border: 'border-green-800/30',
    bg: 'bg-green-900/10',
    features: [
      'Default allow with warnings',
      'Basic deterministic rules active',
      'Prompt Intake Guardian enabled',
      'No approval escalations',
      'Console dashboard access',
      'Audit trail recording',
    ],
    limit: 'Some destructive actions may proceed with only a warning',
  },
  {
    name: 'Professional',
    tag: 'For individual developers',
    icon: Lock,
    color: 'text-accent',
    border: 'border-accent/30',
    bg: 'bg-accent/5',
    features: [
      'Full deterministic rule enforcement',
      'Semantic analysis enabled',
      'Human approval for high-risk actions',
      'Completion evidence verification',
      'Checkpoint and rollback',
      'Receipt chain verification',
    ],
    limit: 'Requires provider configuration for semantic features',
  },
  {
    name: 'Enterprise Strict',
    tag: 'For team and production use',
    icon: Users,
    color: 'text-blue-400',
    border: 'border-blue-800/30',
    bg: 'bg-blue-900/10',
    features: [
      'All Professional features',
      'Signed policy enforcement',
      'L3 workspace isolation',
      'L4 controlled authority',
      'Mandatory approvals for all risky actions',
      'Cross-agent handoff with verified manifests',
      'Lease-based resource access',
    ],
    limit: 'Preview — full L3/L4 features depend on OS support',
  },
];

export default function GuardianModesPage() {
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
            <Link href="/docs" className="text-sm text-zinc-400 hover:text-white transition-colors">Docs</Link>
          </div>
        </div>
      </nav>

      <div className="pt-24 pb-20 px-4">
        <div className="max-w-5xl mx-auto">
          <div className="text-center mb-16">
            <h1 className="text-4xl font-bold text-white mb-4">Guardian Modes</h1>
            <p className="text-zinc-400 max-w-2xl mx-auto">
              Choose the level of governance that matches your workflow. Modes control how Onus evaluates actions, handles approvals, and enforces policies.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {modes.map((mode, i) => {
              const Icon = mode.icon;
              return (
                <div key={i} className={`bg-zinc-900/50 border ${mode.border} rounded-xl p-6 flex flex-col`}>
                  <Icon className={`w-8 h-8 ${mode.color} mb-4`} />
                  <h2 className="text-lg font-semibold text-white mb-1">{mode.name}</h2>
                  <p className="text-xs text-zinc-500 mb-4">{mode.tag}</p>

                  <ul className="space-y-2 mb-6 flex-1">
                    {mode.features.map((f, j) => (
                      <li key={j} className="flex items-start gap-2 text-sm text-zinc-300">
                        <CheckCircle className={`w-3.5 h-3.5 ${mode.color} mt-0.5 flex-shrink-0`} />
                        {f}
                      </li>
                    ))}
                  </ul>

                  {mode.limit && (
                    <div className="flex items-start gap-2 text-xs text-zinc-500 bg-zinc-800/50 rounded-lg p-3">
                      <AlertTriangle className="w-3 h-3 text-yellow-500 mt-0.5 flex-shrink-0" />
                      {mode.limit}
                    </div>
                  )}
                </div>
              );
            })}
          </div>

          <div className="mt-12 text-center">
            <Link href="/docs/guardian-modes" className="text-sm text-accent hover:underline">
              Detailed mode comparison →
            </Link>
          </div>
        </div>
      </div>

      <footer className="border-t border-zinc-800 py-8 px-4">
        <div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus — AI Agent Firewall. Open source (MIT).</div>
      </footer>
    </div>
  );
}
