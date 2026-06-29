'use client';

import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';
import { ShieldCheck, ArrowRight, CheckCircle, XCircle, Clock } from 'lucide-react';

export default function ApprovalsPage() {
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
          <h1 className="text-4xl font-bold text-white text-center mb-4">Human Approvals</h1>
          <p className="text-zinc-400 text-center max-w-2xl mx-auto mb-12">
            High-risk agent actions are escalated for human review. Approvals are cryptographically bound to the exact action payload — no ambiguity, no bypass.
          </p>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
            <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5 text-center">
              <CheckCircle className="w-8 h-8 text-green-400 mx-auto mb-3" />
              <h3 className="font-semibold text-white text-sm mb-2">Exact Binding</h3>
              <p className="text-xs text-zinc-400">Approval is bound to the exact canonical action payload. A modified payload requires a new approval.</p>
            </div>
            <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5 text-center">
              <Clock className="w-8 h-8 text-accent mx-auto mb-3" />
              <h3 className="font-semibold text-white text-sm mb-2">Time-Limited</h3>
              <p className="text-xs text-zinc-400">Each approval has a configurable expiration window. Expired approvals must be re-requested.</p>
            </div>
            <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5 text-center">
              <XCircle className="w-8 h-8 text-red-400 mx-auto mb-3" />
              <h3 className="font-semibold text-white text-sm mb-2">Full Audit Trail</h3>
              <p className="text-xs text-zinc-400">Every approval, denial, and expiration is recorded in the tamper-evident audit trail.</p>
            </div>
          </div>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <h2 className="text-lg font-semibold text-white mb-4">How It Works</h2>
            <div className="space-y-3">
              {[
                { step: '1', text: 'Agent action triggers a deterministic rule requiring approval' },
                { step: '2', text: 'Onus creates an approval request with the exact action payload hash' },
                { step: '3', text: 'The approval appears in the Console UI and CLI' },
                { step: '4', text: 'Human reviews the action and approves or denies' },
                { step: '5', text: 'Onus executes or blocks the action based on the decision' },
                { step: '6', text: 'Result is recorded in the hash-chained audit trail' },
              ].map((item, i) => (
                <div key={i} className="flex items-start gap-3">
                  <div className="w-6 h-6 rounded-full bg-accent/20 text-accent flex items-center justify-center text-xs font-bold flex-shrink-0">{item.step}</div>
                  <p className="text-sm text-zinc-300">{item.text}</p>
                </div>
              ))}
            </div>
          </div>

          <div className="mt-8 text-center">
            <Link href="/docs" className="text-sm text-accent hover:underline">Learn more in the docs →</Link>
          </div>
        </div>
      </div>

      <footer className="border-t border-zinc-800 py-8 px-4">
        <div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus — AI Agent Firewall. Open source (MIT).</div>
      </footer>
    </div>
  );
}
