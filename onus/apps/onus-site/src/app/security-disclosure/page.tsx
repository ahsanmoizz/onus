'use client';

import Link from 'next/link';
import { Lock, Shield } from 'lucide-react';

export default function SecurityDisclosurePage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-zinc-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <Link href="/" className="flex items-center gap-2">
              <div className="w-7 h-7 rounded-full bg-accent flex items-center justify-center"><span className="text-black text-xs font-bold">O</span></div>
              <span className="font-bold text-white text-lg">Onus</span>
            </Link>
          </div>
        </div>
      </nav>

      <div className="pt-24 pb-20 px-4">
        <div className="max-w-3xl mx-auto">
          <div className="flex items-center gap-3 mb-6">
            <Lock className="w-8 h-8 text-accent" />
            <h1 className="text-3xl font-bold text-white">Security Disclosure</h1>
          </div>

          <p className="text-zinc-400 mb-8">
            Onus takes security seriously. If you discover a security vulnerability, report it through a private channel and avoid placing secrets or exploit details in public issues.
          </p>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 mb-6">
            <h2 className="text-lg font-semibold text-white mb-4">Reporting a Vulnerability</h2>
            <p className="text-sm text-zinc-400 mb-4">
              Use GitHub private vulnerability reporting for the repository. If private reporting is unavailable, open a minimal public issue requesting a private disclosure channel and omit exploit details until maintainers respond.
            </p>
            <div className="flex items-center gap-3 bg-black rounded-lg p-4">
              <Shield className="w-5 h-5 text-accent" />
              <code className="text-sm text-zinc-300">GitHub Security Advisory / private vulnerability report</code>
            </div>
          </div>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <h2 className="text-lg font-semibold text-white mb-4">Scope</h2>
            <p className="text-sm text-zinc-400">
              We are interested in vulnerabilities affecting the Onus binary, its security invariants, and the integrity of the audit trail. Please do not test against production infrastructure that is not explicitly authorized.
            </p>
          </div>
        </div>
      </div>

      <footer className="border-t border-zinc-800 py-8 px-4">
        <div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus - AI Agent Firewall. Open source (MIT).</div>
      </footer>
    </div>
  );
}
