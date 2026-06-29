'use client';

import Link from 'next/link';
import { Shield, ArrowRight, FileCheck, Users, Terminal, Lock, Activity, Database, GitBranch } from 'lucide-react';

const steps = [
  { icon: Terminal, title: '1. Prompt Intake Guardian', desc: 'When an agent receives a task, Onus intercepts and analyzes it. Vague requests are clarified, risks are identified, and a bounded task contract is created.', details: ['Analyzes task intent and scope', 'Detects ambiguous or risky requests', 'Creates a bounded task contract', 'Returns READY or REJECTED verdict'] },
  { icon: Shield, title: '2. Deterministic Rules Engine', desc: 'Every agent action is evaluated against always-on deterministic rules before execution. Critical operations are always blocked regardless of the AI model used.', details: ['Always-block rules for destructive ops', 'Path and pattern matching', 'Secret and credential detection', 'Cannot be overridden by LLM'] },
  { icon: Users, title: '3. Human Approval Escalation', desc: 'High-risk actions that pass deterministic rules are escalated for human review. Approvals are cryptographically bound to the exact action payload.', details: ['Exact payload binding', 'Time-limited approval windows', 'Audit trail for every decision', 'Local approval UI'] },
  { icon: FileCheck, title: '4. Completion Verification', desc: 'Onus verifies that claimed completions are backed by real evidence — test results, type checking, and observable state changes.', details: ['Test evidence requirement', 'Type check verification', 'State change confirmation', 'Rejects unsupported claims'] },
  { icon: Database, title: '5. Tamper-Evident Audit Trail', desc: 'Every action, verdict, and approval is recorded in a hash-chained audit trail. The chain can be independently verified.', details: ['Hash chain links each event', 'Independent verification tool', 'Receipt generation for each session', 'Cannot be retroactively modified'] },
  { icon: GitBranch, title: '6. Checkpoint & Rollback', desc: 'Workspace state can be checkpointed before risky operations. If an action causes issues, the workspace can be restored to a known-good state.', details: ['Full workspace snapshots', 'Selective file restoration', 'Cross-session checkpointing', 'Rollback with audit trail'] },
];

export default function HowItWorksPage() {
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
        <div className="max-w-4xl mx-auto">
          <h1 className="text-4xl font-bold text-white text-center mb-4">How It Works</h1>
          <p className="text-zinc-400 text-center max-w-2xl mx-auto mb-16">
            Onus sits between AI coding agents and your development environment, evaluating every action through a layered governance pipeline.
          </p>

          <div className="space-y-12">
            {steps.map((step, i) => {
              const Icon = step.icon;
              return (
                <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
                  <div className="flex items-start gap-4">
                    <div className="w-12 h-12 rounded-xl bg-accent/10 flex items-center justify-center flex-shrink-0">
                      <Icon className="w-6 h-6 text-accent" />
                    </div>
                    <div className="flex-1">
                      <h2 className="text-xl font-semibold text-white mb-2">{step.title}</h2>
                      <p className="text-zinc-400 text-sm leading-relaxed mb-4">{step.desc}</p>
                      <ul className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                        {step.details.map((d, j) => (
                          <li key={j} className="flex items-center gap-2 text-sm text-zinc-500">
                            <span className="w-1 h-1 rounded-full bg-accent" /> {d}
                          </li>
                        ))}
                      </ul>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>

          <div className="mt-12 text-center">
            <Link href="/install" className="inline-flex items-center gap-2 px-8 py-3.5 bg-accent text-black rounded-full font-semibold hover:bg-accent-hover transition-colors text-sm">
              Get Started <ArrowRight className="w-4 h-4" />
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
