'use client';

import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';
import { RefreshCw, ArrowRight, CheckCircle } from 'lucide-react';

export default function RollbackPage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-zinc-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <Link href="/" className="flex items-center gap-2">
              <BrandLogo imageClassName="h-9 w-auto" />
            </Link>
          </div>
        </div>
      </nav>

      <div className="pt-24 pb-20 px-4">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-4xl font-bold text-white text-center mb-4">Checkpoint & Rollback</h1>
          <p className="text-zinc-400 text-center max-w-2xl mx-auto mb-12">
            Onus can capture workspace state before risky operations and restore it if needed — with full audit trail preservation.
          </p>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 mb-8">
            <h2 className="text-lg font-semibold text-white mb-4">Checkpoint</h2>
            <div className="space-y-3">
              {[
                'Capture workspace state before executing risky actions',
                'Includes file system state, git state, and task contract',
                'Cross-session checkpoints persist across daemon restarts',
                'Selective file inclusion/exclusion patterns',
                'Each checkpoint is recorded in the audit trail',
              ].map((item, i) => (
                <div key={i} className="flex items-start gap-2 text-sm text-zinc-300">
                  <CheckCircle className="w-3.5 h-3.5 text-accent mt-0.5 flex-shrink-0" />
                  {item}
                </div>
              ))}
            </div>
          </div>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 mb-8">
            <h2 className="text-lg font-semibold text-white mb-4">Rollback</h2>
            <div className="space-y-3">
              {[
                'Restore workspace to a previously checkpointed state',
                'Undo individual actions or full sessions',
                'Preserve audit trail — rollback is not deletion',
                'Compensation actions for irreversible operations',
                'Cross-session rollback support',
              ].map((item, i) => (
                <div key={i} className="flex items-start gap-2 text-sm text-zinc-300">
                  <CheckCircle className="w-3.5 h-3.5 text-accent mt-0.5 flex-shrink-0" />
                  {item}
                </div>
              ))}
            </div>
          </div>

          <div className="text-center">
            <Link href="/docs/checkpoint-rollback" className="text-sm text-accent hover:underline">
              Detailed documentation →
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
