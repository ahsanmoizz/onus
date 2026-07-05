import Link from 'next/link';
import { Activity, AlertTriangle, BookOpen, Cpu, Database, FileCheck, GitBranch, Key, Lock, RefreshCw, Server, Settings, Shield, Terminal, Users, Zap } from 'lucide-react';
import { BrandLogo } from '@/components/brand-logo';

const sections = [
  {
    category: 'Getting Started',
    icon: BookOpen,
    items: [
      { title: 'Introduction', href: '/docs/introduction', desc: 'What Onus is and what it can honestly govern today.', icon: BookOpen },
      { title: 'Installation', href: '/docs/installation', desc: 'Install Onus on Windows or Linux, or build from source.', icon: Terminal },
      { title: 'Quick Start', href: '/docs/quick-start', desc: 'Install, doctor, daemon, console, intake, setup, and evidence review.', icon: Zap },
      { title: 'CLI Reference', href: '/docs/cli-reference', desc: 'Current CLI commands from the working binary.', icon: Terminal },
    ],
  },
  {
    category: 'Core Concepts',
    icon: Shield,
    items: [
      { title: 'Guardian Modes', href: '/docs/guardian-modes', desc: 'Beginner, professional, strict, deterministic, local, and provider-disabled operation.', icon: Settings },
      { title: 'Providers', href: '/docs/providers', desc: 'Provider configuration and offline deterministic mode.', icon: Cpu },
      { title: 'Prompt Intake', href: '/docs/prompt-intake', desc: 'Classify prompts before an agent starts work.', icon: Terminal },
      { title: 'Task Contracts', href: '/docs/task-contracts', desc: 'Bounded scopes, protected resources, budgets, and required evidence.', icon: FileCheck },
    ],
  },
  {
    category: 'Use and Manage',
    icon: Zap,
    items: [
      { title: 'Console Access', href: '/login', desc: 'Password/token-gated local dashboard access started with `onus console`.', icon: Server },
      { title: 'Running Governed Tasks', href: '/docs/running-governed-tasks', desc: 'Run actions through Onus-owned or routed execution surfaces.', icon: Terminal },
      { title: 'Approvals', href: '/docs/approvals', desc: 'Human approval workflow with exact payload binding.', icon: Users },
      { title: 'Checkpoint & Rollback', href: '/docs/checkpoint-rollback', desc: 'Supported rollback and honest unsupported states.', icon: RefreshCw },
      { title: 'Memory', href: '/docs/memory', desc: 'Scoped session, project, policy, incident, and preference memory.', icon: Database },
    ],
  },
  {
    category: 'Security and Evidence',
    icon: Lock,
    items: [
      { title: 'Rules & Policies', href: '/docs/rules-policies', desc: 'Deterministic rules, policy memory, and managed policies.', icon: Shield },
      { title: 'MCP L2', href: '/docs/mcp-l2', desc: 'MCP gateway behavior when traffic is routed through Onus.', icon: Server },
      { title: 'L3 Workspaces', href: '/docs/l3-workspaces', desc: 'Linux containment requirements and limitations.', icon: GitBranch },
      { title: 'L4 Authority', href: '/docs/l4-authority', desc: 'Narrow controlled authority proof, not general production authority.', icon: Key },
      { title: 'Receipts & Audit', href: '/docs/receipts-audit', desc: 'Tamper-evident local audit trail and verification.', icon: Database },
      { title: 'Security Model', href: '/docs/security-model', desc: 'Trust boundaries, enforcement levels, bypasses, and residual risks.', icon: Lock },
    ],
  },
  {
    category: 'Integrations and Reference',
    icon: GitBranch,
    items: [
      { title: 'Integrations', href: '/docs/integrations', desc: 'L1 best-effort hooks and L2 routed protocol integrations.', icon: GitBranch },
      { title: 'Agent Handoff', href: '/docs/agent-handoff', desc: 'Cross-agent continuity manifests.', icon: Activity },
      { title: 'Troubleshooting', href: '/docs/troubleshooting', desc: 'Common failures and commands to diagnose them.', icon: AlertTriangle },
      { title: 'Limitations', href: '/docs/limitations', desc: 'Current known limitations and unsupported claims.', icon: AlertTriangle },
    ],
  },
];

function SectionCard({ category, icon: Icon, items }: { category: string; icon: any; items: { title: string; href: string; desc: string; icon: any }[] }) {
  return (
    <section className="rounded-lg border border-zinc-800 bg-zinc-900/45 p-6">
      <div className="mb-4 flex items-center gap-2">
        <Icon className="h-5 w-5 text-accent" />
        <h2 className="text-lg font-semibold text-white">{category}</h2>
      </div>
      <div className="space-y-2">
        {items.map((item) => {
          const ItemIcon = item.icon;
          return (
            <Link key={item.href} href={item.href} className="-mx-3 block rounded-lg p-3 transition-colors hover:bg-zinc-800/50">
              <div className="flex items-start gap-3">
                <ItemIcon className="mt-0.5 h-4 w-4 flex-shrink-0 text-accent" />
                <div>
                  <h3 className="text-sm font-medium text-white">{item.title}</h3>
                  <p className="mt-0.5 text-xs leading-5 text-zinc-500">{item.desc}</p>
                </div>
              </div>
            </Link>
          );
        })}
      </div>
    </section>
  );
}

export default function DocsPage() {
  return (
    <div className="min-h-screen bg-black text-zinc-100">
      <nav className="fixed inset-x-0 top-0 z-50 border-b border-zinc-800 bg-black/85 backdrop-blur-md">
        <div className="mx-auto flex h-16 max-w-6xl items-center px-4">
          <Link href="/" className="flex items-center" aria-label="Onus home">
            <BrandLogo imageClassName="h-9 w-auto" />
          </Link>
          <div className="ml-auto space-x-6 text-sm text-zinc-400">
            <Link href="/download" className="hover:text-white">Download</Link>
            <Link href="/install" className="hover:text-white">Install</Link>
            <Link href="/login" className="hover:text-white">Access</Link>
          </div>
        </div>
      </nav>

      <main className="mx-auto max-w-6xl px-4 pb-16 pt-24">
        <h1 className="mb-2 text-4xl font-bold text-white">Documentation</h1>
        <p className="mb-10 max-w-2xl text-zinc-400">
          Install, configure, run, govern, audit, and manage Onus without blurring what is verified,
          best-effort, experimental, or unsupported.
        </p>

        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          {sections.map((section) => (
            <SectionCard key={section.category} {...section} />
          ))}
        </div>

        <section className="mt-12 rounded-lg border border-zinc-800 bg-zinc-950 p-6">
          <h2 className="mb-3 text-lg font-semibold text-white">Need a clean first run?</h2>
          <p className="mb-4 text-sm leading-6 text-zinc-400">
            Start with install and quick start, then open the admin console. Do not connect production credentials
            or claim L3/L4 behavior until the required runtime checks pass in your environment.
          </p>
          <div className="flex gap-3">
            <Link href="/install" className="rounded-full bg-accent px-4 py-2 text-sm font-semibold text-black hover:bg-accent-hover">Install</Link>
            <Link href="/docs/quick-start" className="rounded-full border border-zinc-700 px-4 py-2 text-sm text-zinc-200 hover:bg-zinc-900">Quick Start</Link>
          </div>
        </section>
      </main>
    </div>
  );
}
