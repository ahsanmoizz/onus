import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function IntroductionPage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-sm border-b border-zinc-800">
        <div className="max-w-6xl mx-auto px-4 h-14 flex items-center justify-between">
          <Link href="/" className="inline-flex items-center" aria-label="Onus home"><BrandLogo imageClassName="h-9 w-auto" /></Link>
          <div className="flex items-center gap-6 text-sm text-zinc-400">
            <Link href="/product" className="hover:text-white transition-colors">Product</Link>
            <Link href="/install" className="hover:text-white transition-colors">Install</Link>
            <Link href="/docs" className="text-accent transition-colors">Docs</Link>
          </div>
        </div>
      </nav>

      <main className="pt-20 pb-16 px-4 max-w-4xl mx-auto">
        <Link href="/docs" className="text-sm text-zinc-400 hover:text-white transition-colors inline-flex items-center gap-1 mb-8">&larr; Back to Docs</Link>

        <h1 className="text-3xl font-bold text-white mb-4">Introduction</h1>

        <p className="text-zinc-300 leading-relaxed mb-6">
          Onus is an <strong className="text-white">AI Agent Firewall</strong> that sits between AI coding agents and your codebase. It intercepts every action an agent attempts and evaluates it against deterministic rules and semantic policies before the action reaches your filesystem. This means no unapproved file writes, no dangerous shell commands, and no silent data exfiltration.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">The Problem</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Modern AI coding agents are extraordinarily powerful. They can read, write, and execute arbitrary code across your entire project. A single hallucinated command or a prompt injection attack can delete production data, leak credentials, or introduce critical vulnerabilities. Traditional guardrails either do not exist or are trivially bypassed by the agent itself.
        </p>
        <p className="text-zinc-300 leading-relaxed mb-4">
          The core problem is that AI agents operate without accountability. They make decisions autonomously, can be influenced by untrusted context (prompt injection), and leave no reliable audit trail of what they actually did. Onus solves this by introducing a mandatory enforcement layer that evaluates every action before execution and records everything in a tamper-evident audit log.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">The Solution</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Onus combines deterministic and semantic evaluation to govern agent behavior. Every action an agent attempts is intercepted and checked against:
        </p>
        <ul className="list-disc list-inside text-zinc-300 space-y-2 mb-6 ml-4">
          <li><strong className="text-white">Deterministic rules</strong> &mdash; Pattern-based checks that run offline with no external dependencies. These catch dangerous shell commands, disallowed file patterns, and known attack signatures.</li>
          <li><strong className="text-white">Semantic evaluation</strong> &mdash; An LLM-powered analysis (either local or cloud) that assesses the intent and risk of each action. This catches novel attacks and policy violations that pattern matching alone would miss.</li>
          <li><strong className="text-white">Human approvals</strong> &mdash; When an action exceeds a configurable risk threshold, Onus escalates it for human review. The approval binds to the exact payload, so the agent cannot modify the action after approval.</li>
        </ul>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Enforcement Levels L1&ndash;L4</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Onus defines four enforcement levels that map to increasing security guarantees:
        </p>
        <ul className="list-disc list-inside text-zinc-300 space-y-2 mb-6 ml-4">
          <li><strong className="text-white">L1 &mdash; Notarization</strong> &mdash; Actions are logged and hash-chained but not blocked. Best-effort hooks at the version control level provide awareness without enforcement.</li>
          <li><strong className="text-white">L2 &mdash; Evaluation</strong> &mdash; Actions are intercepted by the MCP proxy and evaluated against rules and policies. Every action is logged with a decision outcome. Only actions routed through Onus receive L2 guarantees.</li>
          <li><strong className="text-white">L3 &mdash; Containment</strong> &mdash; Actions execute inside a containerized workspace with filesystem isolation, network policies, and resource limits. The agent cannot access files outside the workspace scope.</li>
          <li><strong className="text-white">L4 &mdash; Authority</strong> &mdash; Onus controls credentials and authority. The agent receives disposable short-lived credentials bound to specific actions with automatic expiry. No persistent secrets are exposed to the agent.</li>
        </ul>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Target Audience</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Onus is built for development teams, platform engineers, and security teams who rely on AI coding agents and need guardrails that actually work. If you are using Claude Code CLI, OpenAI Codex CLI, Cursor, VS Code with AI extensions, or any other agentic coding tool, Onus provides the governance layer your security posture requires.
        </p>

        <div className="border-t border-zinc-800 mt-12 pt-6">
          <p className="text-sm text-zinc-500">
            Ready to get started? See the <Link href="/docs/installation" className="text-accent hover:underline">Installation guide</Link> or jump to the <Link href="/docs/quick-start" className="text-accent hover:underline">Quick Start</Link>.
          </p>
        </div>
      </main>
    </div>
  );
}
