'use client';

import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';
import { ArrowRight, CheckCircle } from 'lucide-react';

export default function AgentContinuityPage() {
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
          <h1 className="text-4xl font-bold text-white text-center mb-4">Agent Continuity</h1>
          <p className="text-zinc-400 text-center max-w-2xl mx-auto mb-12">
            Switch between AI coding agents without losing task context. Onus preserves and transfers the full task state across agents via verified handoff manifests.
          </p>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-12">
            <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
              <h2 className="text-lg font-semibold text-white mb-4">Handoff Manifest</h2>
              <ul className="space-y-2 text-sm text-zinc-300">
                {['Task contract and intent', 'Repository workspace state', 'Active checkpoint references', 'Collected evidence and corrections', 'Full audit trail excerpt', 'Agent-specific context'].map((item, i) => (
                  <li key={i} className="flex items-start gap-2"><CheckCircle className="w-3.5 h-3.5 text-accent mt-0.5 flex-shrink-0" />{item}</li>
                ))}
              </ul>
            </div>
            <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
              <h2 className="text-lg font-semibold text-white mb-4">Supported Agents</h2>
              <ul className="space-y-2 text-sm text-zinc-300">
                {['Claude Code CLI', 'OpenAI Codex CLI', 'Google Antigravity', 'More coming'].map((item, i) => (
                  <li key={i} className="flex items-start gap-2"><CheckCircle className="w-3.5 h-3.5 text-accent mt-0.5 flex-shrink-0" />{item}</li>
                ))}
              </ul>
            </div>
          </div>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <h2 className="text-lg font-semibold text-white mb-4">Continuity Flow</h2>
            <div className="space-y-3">
              {[
                { step: '1', text: 'Start a governed task with Claude Code — Onus records task contract and repository state' },
                { step: '2', text: 'Create a checkpoint of the workspace and audit trail' },
                { step: '3', text: 'Onus generates a verified handoff manifest signed with the session key' },
                { step: '4', text: 'Switch to Codex CLI — Onus presents the handoff manifest' },
                { step: '5', text: 'Codex receives the full task context including audit trail and evidence' },
                { step: '6', text: 'Same task continues with a unified audit trail across both agents' },
              ].map((item, i) => (
                <div key={i} className="flex items-start gap-3">
                  <div className="w-6 h-6 rounded-full bg-accent/20 text-accent flex items-center justify-center text-xs font-bold flex-shrink-0">{item.step}</div>
                  <p className="text-sm text-zinc-300">{item.text}</p>
                </div>
              ))}
            </div>
          </div>

          <div className="mt-8 text-sm text-zinc-500 text-center">
            Note: Subscriptions, provider quotas, and rate limits remain separate per agent.
          </div>
        </div>
      </div>

      <footer className="border-t border-zinc-800 py-8 px-4">
        <div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus — AI Agent Firewall. Open source (MIT).</div>
      </footer>
    </div>
  );
}
