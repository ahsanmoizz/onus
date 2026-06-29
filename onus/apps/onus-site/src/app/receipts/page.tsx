'use client';

import Link from 'next/link';
import { FileText, ArrowRight, CheckCircle } from 'lucide-react';

export default function ReceiptsPage() {
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
          <h1 className="text-4xl font-bold text-white text-center mb-4">Receipts & Audit Trail</h1>
          <p className="text-zinc-400 text-center max-w-2xl mx-auto mb-12">
            Every action, verdict, and approval is recorded in a tamper-evident hash-chained audit trail that can be independently verified.
          </p>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
            {[
              { title: 'Hash Chain', desc: 'Each event is linked to the previous event via SHA-256 hash. Modifying any entry breaks the chain.' },
              { title: 'Independent Verification', desc: 'The verify command checks the full chain integrity without requiring the daemon.' },
              { title: 'Receipt Export', desc: 'Export session receipts for external audit or compliance purposes.' },
              { title: 'Cross-Session Links', desc: 'Handoff manifests link receipts across agent sessions for unified audit trails.' },
            ].map((item, i) => (
              <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5">
                <h3 className="font-semibold text-white text-sm mb-2">{item.title}</h3>
                <p className="text-xs text-zinc-400">{item.desc}</p>
              </div>
            ))}
          </div>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <h2 className="text-lg font-semibold text-white mb-4">Receipt Schema</h2>
            <div className="bg-black rounded-lg p-4">
              <pre className="text-xs text-zinc-400 font-mono overflow-x-auto">
{`{
  "session_id": "ses_...",
  "event_index": 42,
  "action": "file.write",
  "verdict": "allow",
  "timestamp": "...",
  "previous_hash": "abc...",
  "hash": "def...",
  "approved_by": null
}`}
              </pre>
            </div>
          </div>
        </div>
      </div>

      <footer className="border-t border-zinc-800 py-8 px-4">
        <div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus — AI Agent Firewall. Open source (MIT).</div>
      </footer>
    </div>
  );
}
