'use client';

import Link from 'next/link';
import { motion } from 'framer-motion';
import { Shield, Terminal, FileCheck, RefreshCw, Users, GitBranch, Key, Database, ArrowRight, CheckCircle, AlertTriangle, Zap, Lock, Activity, BookOpen, Download } from 'lucide-react';
import { Entropy } from '@/components/ui/entropy';
import { FallingPattern } from '@/components/ui/falling-pattern';
import { BrandLogo } from '@/components/brand-logo';
import { OnusScrambleLine, RainingOnusHero } from '@/components/ui/modern-animated-hero-section';

function Navbar() {
  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-zinc-800">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          <Link href="/" className="flex items-center" aria-label="Onus home">
            <BrandLogo imageClassName="h-10 w-auto" />
          </Link>
          <div className="hidden md:flex items-center gap-6 text-sm">
            <Link href="/product" className="text-zinc-400 hover:text-white transition-colors">Product</Link>
            <Link href="/docs" className="text-zinc-400 hover:text-white transition-colors">Docs</Link>
            <Link href="/download" className="text-zinc-400 hover:text-white transition-colors">Download</Link>
            <Link href="/install" className="text-zinc-400 hover:text-white transition-colors">Install</Link>
            <Link href="/security" className="text-zinc-400 hover:text-white transition-colors">Security</Link>
            <Link href="/integrations" className="text-zinc-400 hover:text-white transition-colors">Integrations</Link>
          </div>
          <div className="flex items-center gap-3">
            <a href="https://github.com/ahsanmoizz/onus" className="text-sm text-zinc-400 hover:text-white transition-colors">GitHub</a>
            <Link href="/login" className="inline-flex items-center gap-2 rounded-full border border-white/20 px-4 py-2 text-sm font-medium text-zinc-200 transition-colors hover:bg-white/10">
              <Lock className="h-4 w-4" />
              Access
            </Link>
          </div>
        </div>
      </div>
    </nav>
  );
}

function HeroSection() {
  return (
    <section className="relative pt-32 pb-20 px-4 overflow-hidden">
      {/* Background effects */}
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,_rgba(249,115,22,0.08)_0%,_transparent_50%)]" />
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_bottom_left,_rgba(59,130,246,0.05)_0%,_transparent_50%)]" />

      <div className="max-w-5xl mx-auto text-center relative">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
        >
          <div className="inline-flex items-center gap-2 px-3 py-1 bg-accent/10 border border-accent/20 rounded-full text-accent text-xs mb-6">
            <Zap className="w-3 h-3" />
            AI Agent Firewall - v0.1.0
          </div>

          <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold tracking-tight leading-[1.1] text-white mb-6">
            Govern your AI agents{' '}
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-accent to-orange-400">
              with deterministic control
            </span>
          </h1>

          <p className="text-lg sm:text-xl text-zinc-400 max-w-3xl mx-auto mb-10 leading-relaxed">
            Onus is a governance and execution-control layer for AI coding agents.
            It converts unclear requests into bounded task contracts, evaluates agent
            actions, protects sensitive resources, handles exact approvals, verifies
            completion evidence, and preserves accountable execution records.
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center items-center">
            <Link href="/install" className="px-8 py-3.5 bg-accent text-black rounded-full font-semibold hover:bg-accent-hover transition-colors flex items-center gap-2 text-sm">
              <Download className="w-4 h-4" />
              Install Onus
            </Link>
            <Link href="/docs" className="px-8 py-3.5 border border-zinc-700 text-zinc-300 rounded-full font-medium hover:bg-zinc-900 transition-colors flex items-center gap-2 text-sm">
              <BookOpen className="w-4 h-4" />
              Read the Docs
            </Link>
          </div>
        </motion.div>

        {/* Feature highlights */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.3 }}
          className="mt-20 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 text-left"
        >
          {[
            { icon: Terminal, title: 'Prompt Intake', desc: 'Analyzes ambiguous task requests into bounded contracts before any agent acts' },
            { icon: Shield, title: 'Deterministic Rules', desc: 'Always-block rules for destructive operations, secrets, and sensitive paths' },
            { icon: Users, title: 'Human Approvals', desc: 'Escalates high-risk actions to human operators with exact payload binding' },
            { icon: FileCheck, title: 'Verified Completion', desc: 'Requires real test evidence, not user-supplied success booleans' },
          ].map((feature, i) => {
            const Icon = feature.icon;
            return (
              <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5 hover:border-zinc-700 transition-colors">
                <Icon className="w-5 h-5 text-accent mb-3" />
                <h3 className="font-semibold text-white text-sm mb-1">{feature.title}</h3>
                <p className="text-zinc-500 text-xs leading-relaxed">{feature.desc}</p>
              </div>
            );
          })}
        </motion.div>
      </div>
    </section>
  );
}

function HeroSectionV2() {
  return (
    <section className="relative min-h-screen overflow-hidden bg-[#0647f7] px-4 pt-28 pb-16">
      <div className="absolute inset-x-0 top-16 h-px bg-white/20" />
      <div className="absolute inset-y-0 left-[4vw] hidden w-px bg-white/15 md:block" />
      <div className="absolute inset-y-0 right-[4vw] hidden w-px bg-white/15 md:block" />
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_80%_18%,rgba(255,255,255,0.16),transparent_28%),linear-gradient(180deg,rgba(6,71,247,0.08),rgba(2,18,86,0.38))]" />
      <RainingOnusHero />

      <div className="relative mx-auto grid min-h-[calc(100vh-7rem)] max-w-7xl items-center gap-10 lg:grid-cols-[1.08fr_0.92fr]">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          className="max-w-5xl"
        >
          <div className="mb-8 inline-flex items-center gap-2 rounded-full border border-white/25 bg-white/10 px-3 py-1 text-xs font-medium text-white/80 backdrop-blur">
            <Zap className="h-3 w-3 text-[#ffe55c]" />
            AI Agent Firewall - local-first control plane
          </div>

          <div className="mb-8 space-y-1 text-[clamp(3.25rem,8.5vw,8rem)] font-semibold leading-[0.96] tracking-tight">
            {['vague prompts', 'risky writes', 'secret leaks', 'silent approvals'].map((word) => (
              <div key={word} className="text-white/24">{word}</div>
            ))}
            <div className="flex flex-wrap items-baseline gap-x-6">
              <span className="text-white">ONUS</span>
              <span className="text-[#ffe55c]">control</span>
            </div>
            <div className="text-white/24">missing evidence</div>
          </div>

          <h1 className="mb-5 max-w-5xl text-2xl font-semibold leading-tight text-white sm:text-3xl md:text-4xl">
            <OnusScrambleLine />
          </h1>

          <p className="mb-8 max-w-2xl text-base leading-7 text-blue-50/82 sm:text-lg">
            Onus turns vague requests into bounded task contracts, evaluates each routed action,
            binds risky approvals to exact payloads, and rejects completion when evidence is missing.
          </p>

          <div className="flex flex-col gap-3 sm:flex-row">
            <Link href="/install" className="inline-flex items-center justify-center gap-2 rounded-sm bg-[#06123c] px-7 py-3 text-sm font-semibold text-white transition-colors hover:bg-[#020817]">
              <Download className="h-4 w-4" />
              Install Onus
            </Link>
            <Link href="/login" className="inline-flex items-center justify-center gap-2 rounded-sm border border-white/35 px-7 py-3 text-sm font-medium text-white transition-colors hover:bg-white/10">
              <Terminal className="h-4 w-4" />
              Access local console
            </Link>
          </div>

          <div className="mt-10 grid max-w-2xl grid-cols-2 gap-3 sm:grid-cols-4">
            {[
              ['L1', 'Best-effort hooks'],
              ['L2', 'Onus-routed actions'],
              ['L3', 'Linux workspace proof'],
              ['L4', 'Narrow authority proof'],
            ].map(([level, label]) => (
              <div key={level} className="rounded-sm border border-white/20 bg-white/[0.08] p-3 backdrop-blur">
                <div className="text-sm font-semibold text-white">{level}</div>
                <div className="mt-1 text-xs leading-5 text-blue-50/70">{label}</div>
              </div>
            ))}
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, scale: 0.96 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.7, delay: 0.15 }}
          className="relative mx-auto w-full max-w-[500px]"
        >
          <div className="relative overflow-hidden rounded-sm border border-white/20 bg-white/10 p-6 shadow-2xl shadow-blue-950/40 backdrop-blur">
            <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_0%,rgba(255,255,255,0.2),transparent_42%)]" />
            <div className="relative flex min-h-[430px] items-center justify-center">
              <BrandLogo imageClassName="w-full max-w-[360px] opacity-95" />
              <div className="absolute bottom-4 left-4 right-4 space-y-3">
                {[
                  ['Prompt intake', 'READY_WITH_SAFE_CONTRACT'],
                  ['Action policy', 'DENY beats approval'],
                  ['Completion', 'Evidence required'],
                ].map(([label, value]) => (
                  <div key={label} className="flex items-center justify-between rounded-sm border border-white/20 bg-[#06123c]/80 px-3 py-2 text-xs">
                    <span className="text-blue-50/65">{label}</span>
                    <span className="font-mono text-white">{value}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}

function ProblemSection() {
  return (
    <section className="py-20 px-4 border-t border-zinc-800">
      <div className="max-w-5xl mx-auto">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
          <div>
            <h2 className="text-3xl font-bold text-white mb-4">The Problem</h2>
            <p className="text-zinc-400 mb-6 leading-relaxed">
              AI coding agents have unrestricted access to execute commands, read and write files,
              and interact with external services. Without governance:
            </p>
            <ul className="space-y-3">
              {[
                'Agents can delete production files or modify critical configuration',
                'Sensitive credentials and API keys can be leaked',
                'Destructive git operations can destroy hours of work',
                'No audit trail exists for agent actions',
                'Task requirements are vague leading to incorrect or dangerous execution',
              ].map((item, i) => (
                <li key={i} className="flex items-start gap-3 text-sm text-zinc-300">
                  <AlertTriangle className="w-4 h-4 text-zinc-600 mt-0.5 flex-shrink-0" />
                  {item}
                </li>
              ))}
            </ul>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-8">
            <h3 className="font-semibold text-white mb-4">Onus Control Flow</h3>
            <div className="space-y-3">
              {[
                { step: '1', label: 'Prompt Intake Guardian', desc: 'Analyzes task into contract' },
                { step: '2', label: 'Deterministic + Semantic', desc: 'Rules engine evaluates each action' },
                { step: '3', label: 'Human Approvals', desc: 'Escalates high-risk actions with binding' },
                { step: '4', label: 'Completion Verification', desc: 'Tests evidence, not claims' },
                { step: '5', label: 'Receipt Chain', desc: 'Tamper-evident audit trail' },
              ].map((item, i) => (
                <div key={i} className="flex items-start gap-3">
                  <div className="w-7 h-7 rounded-full bg-accent/20 text-accent flex items-center justify-center text-xs font-bold flex-shrink-0">
                    {item.step}
                  </div>
                  <div>
                    <p className="text-sm font-medium text-white">{item.label}</p>
                    <p className="text-xs text-zinc-500">{item.desc}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function ArchitectureSection() {
  return (
    <section className="py-20 px-4 border-t border-zinc-800">
      <div className="max-w-5xl mx-auto">
        <h2 className="text-3xl font-bold text-white text-center mb-4">Deterministic + Semantic Architecture</h2>
        <p className="text-zinc-400 text-center max-w-2xl mx-auto mb-12">
          Onus combines always-on deterministic rules with semantic analysis for a layered defense
        </p>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <Lock className="w-6 h-6 text-green-400 mb-3" />
            <h3 className="font-semibold text-white mb-2">Deterministic Rules</h3>
            <p className="text-zinc-400 text-sm leading-relaxed">Always-block rules that cannot be overridden by an LLM:</p>
            <ul className="mt-3 space-y-1.5">
              {['rm -rf /', 'git push --force', 'Write to /etc/', 'Read .env files', 'Destructive SQL', 'Secret exfiltration'].map((rule, i) => (
                <li key={i} className="text-xs text-zinc-500 font-mono flex items-center gap-2">
                  <span className="w-1 h-1 rounded-full bg-red-500" /> {rule}
                </li>
              ))}
            </ul>
          </div>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <Zap className="w-6 h-6 text-accent mb-3" />
            <h3 className="font-semibold text-white mb-2">Semantic Analysis</h3>
            <p className="text-zinc-400 text-sm leading-relaxed">Provider-based evaluation for contextual risks:</p>
            <ul className="mt-3 space-y-1.5">
              {['Prompt Intake Guardian', 'Risk classification', 'Action scoring', 'Correction generation', 'Evidence verification', 'Safe contract proposals'].map((item, i) => (
                <li key={i} className="text-xs text-zinc-500 flex items-center gap-2">
                  <span className="w-1 h-1 rounded-full bg-accent" /> {item}
                </li>
              ))}
            </ul>
          </div>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <Activity className="w-6 h-6 text-blue-400 mb-3" />
            <h3 className="font-semibold text-white mb-2">L1-L4 Enforcement</h3>
            <p className="text-zinc-400 text-sm leading-relaxed">Progressive enforcement levels:</p>
            <ul className="mt-3 space-y-1.5">
              {['L1: Best-effort hook (cooperative)', 'L2: Onus-routed actions', 'L3: Process/FS/Net containment', 'L4: Controlled authority (disposable creds)'].map((level, i) => (
                <li key={i} className="text-xs text-zinc-500 flex items-center gap-2">
                  <span className="w-1 h-1 rounded-full bg-blue-500" /> {level}
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    </section>
  );
}

function IntegrationsSection() {
  return (
    <section className="py-20 px-4 border-t border-zinc-800">
      <div className="max-w-5xl mx-auto">
        <h2 className="text-3xl font-bold text-white text-center mb-4">Supported Integrations</h2>
        <p className="text-zinc-400 text-center max-w-2xl mx-auto mb-12">
          Onus integrates with popular AI coding agents and IDEs
        </p>
        <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
          {[
            { name: 'Claude Code CLI', status: 'Best-effort', level: 'L1' },
            { name: 'OpenAI Codex CLI', status: 'Protocol route', level: 'L2 routed' },
            { name: 'Generic MCP', status: 'Protocol route', level: 'L2 routed' },
            { name: 'VS Code', status: 'Best-effort', level: 'L1' },
            { name: 'Cursor IDE', status: 'Protocol route', level: 'L2 routed' },
          ].map((item, i) => (
            <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4 text-center hover:border-zinc-700 transition-colors">
              <p className="text-white font-medium text-sm mb-1">{item.name}</p>
              <p className="text-xs text-zinc-500 mb-1">{item.status}</p>
              <span className="text-[10px] px-2 py-0.5 rounded-full bg-accent/10 text-accent">{item.level}</span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function CtaSection() {
  return (
    <section className="py-20 px-4 border-t border-zinc-800">
      <div className="max-w-3xl mx-auto text-center">
        <h2 className="text-3xl font-bold text-white mb-4">Ready to govern your agents?</h2>
        <p className="text-zinc-400 mb-8 max-w-xl mx-auto">
          Install Onus, set up your provider (deterministic-only mode works offline without one), and start protecting your development workflow in minutes.
        </p>
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <Link href="/install" className="px-8 py-3.5 bg-accent text-black rounded-full font-semibold hover:bg-accent-hover transition-colors flex items-center gap-2 justify-center text-sm">
            Get Started
            <ArrowRight className="w-4 h-4" />
          </Link>
          <Link href="/docs" className="px-8 py-3.5 border border-zinc-700 text-zinc-300 rounded-full font-medium hover:bg-zinc-900 transition-colors text-sm">
            Documentation
          </Link>
        </div>
      </div>
    </section>
  );
}

function Footer() {
  return (
    <footer className="border-t border-zinc-800 py-12 px-4">
      <div className="max-w-5xl mx-auto">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8 mb-8">
          <div>
            <h4 className="font-semibold text-white text-sm mb-3">Product</h4>
            <div className="space-y-2 text-sm">
              <Link href="/product" className="block text-zinc-500 hover:text-zinc-300">Overview</Link>
              <Link href="/how-it-works" className="block text-zinc-500 hover:text-zinc-300">How it Works</Link>
              <Link href="/guardian-modes" className="block text-zinc-500 hover:text-zinc-300">Guardian Modes</Link>
              <Link href="/security" className="block text-zinc-500 hover:text-zinc-300">Security</Link>
            </div>
          </div>
          <div>
            <h4 className="font-semibold text-white text-sm mb-3">Documentation</h4>
            <div className="space-y-2 text-sm">
              <Link href="/docs" className="block text-zinc-500 hover:text-zinc-300">Quick Start</Link>
              <Link href="/docs/cli" className="block text-zinc-500 hover:text-zinc-300">CLI Reference</Link>
              <Link href="/docs/integrations" className="block text-zinc-500 hover:text-zinc-300">Integrations</Link>
              <Link href="/docs/troubleshooting" className="block text-zinc-500 hover:text-zinc-300">Troubleshooting</Link>
            </div>
          </div>
          <div>
            <h4 className="font-semibold text-white text-sm mb-3">Community</h4>
            <div className="space-y-2 text-sm">
              <a href="https://github.com/ahsanmoizz/onus" className="block text-zinc-500 hover:text-zinc-300">GitHub</a>
              <Link href="/changelog" className="block text-zinc-500 hover:text-zinc-300">Changelog</Link>
              <Link href="/limitations" className="block text-zinc-500 hover:text-zinc-300">Limitations</Link>
            </div>
          </div>
          <div>
            <h4 className="font-semibold text-white text-sm mb-3">Legal</h4>
            <div className="space-y-2 text-sm">
              <Link href="/privacy" className="block text-zinc-500 hover:text-zinc-300">Privacy</Link>
              <Link href="/terms" className="block text-zinc-500 hover:text-zinc-300">Terms</Link>
              <Link href="/security-disclosure" className="block text-zinc-500 hover:text-zinc-300">Disclosure</Link>
            </div>
          </div>
        </div>
        <div className="border-t border-zinc-800 pt-6 text-center text-xs text-zinc-600">
          Onus - AI Agent Firewall. Open source (MIT).
        </div>
      </div>
    </footer>
  );
}

export default function HomePage() {
  return (
    <div className="min-h-screen">
      <Navbar />
      <HeroSectionV2 />
      <ProblemSection />
      <ArchitectureSection />
      <IntegrationsSection />
      <CtaSection />
      <Footer />
    </div>
  );
}
