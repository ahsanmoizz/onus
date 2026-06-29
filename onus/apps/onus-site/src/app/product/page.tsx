import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';
import { Shield, Terminal, Users, FileCheck, RefreshCw, Lock, Zap, Activity, Database, GitBranch, Key, ArrowRight } from 'lucide-react';

const features = [
  {
    icon: Terminal,
    title: 'Prompt Intake Guardian',
    desc: 'Analyzes ambiguous task requests before any agent acts. Produces READY, CLARIFICATION_REQUIRED, or REJECTED_AS_UNSAFE outcomes with structured safe contracts. Supports OpenAI, Anthropic, local, or deterministic-only modes.',
  },
  {
    icon: Shield,
    title: 'Deterministic Rules Engine',
    desc: 'Always-block rules for destructive operations, secret exfiltration, protected paths, and dangerous git operations. Uses multi-pattern matching (aho-corasick) for Tier 1 shell rules. Cannot be overridden by an LLM under any circumstances.',
  },
  {
    icon: Zap,
    title: 'Semantic Analysis',
    desc: 'Provider-based evaluation for contextual risks, action scoring, and correction generation. Supports OpenAI (cloud), Anthropic (cloud), llama.cpp (local), and deterministic-only modes. Each evaluation produces structured verdicts with corrections.',
  },
  {
    icon: Users,
    title: 'Human Approval Workflow',
    desc: 'Escalates high-risk actions with exact payload binding, task contract hash, policy version, and environment identity verification. Runs a local HTTP approval server with token-based auth, CSRF protection, and rate limiting.',
  },
  {
    icon: FileCheck,
    title: 'Completion Verification',
    desc: 'Requires real test evidence: exit codes, lint results, type checks, build output, and security scans. Not user-supplied success booleans. Progressively verifies from exit codes through full test suite execution.',
  },
  {
    icon: RefreshCw,
    title: 'Checkpoints & Rollback',
    desc: 'Create workspace checkpoints before risky operations. Roll back individual actions, groups of actions, or entire sessions to initial state. Compensation operations for irreversible effects. Honest irreversibility reporting.',
  },
  {
    icon: GitBranch,
    title: 'L3 Workspaces',
    desc: 'Containerized execution with filesystem isolation, network policy, resource limits, and environment filtering. Uses bubblewrap on Linux. Falls closed on unsupported platforms. Includes protected host paths and boundary verification.',
  },
  {
    icon: Key,
    title: 'L4 Authority',
    desc: 'Disposable short-lived credentials for controlled operations. Human-approved capabilities with exact payload binding, automatic expiry, and revocation. Hash-chained receipts for every authority operation.',
  },
  {
    icon: Database,
    title: 'Receipt Chain',
    desc: 'Tamper-evident audit trail with hash-chained actions. Every action links to the previous via SHA-256 with Merkle tree root anchoring. Chain verification detects any tampering. Supports external anchoring for third-party verification.',
  },
  {
    icon: Activity,
    title: 'Agent Handoff',
    desc: 'Cross-agent continuity with complete serialized state: task contract, checkpoint manifest, session/project memory, policy context, and verified audit trail. The receiving agent can verify integrity via manifest hash before continuing.',
  },
];

export default function ProductPage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-zinc-800">
        <div className="max-w-5xl mx-auto px-4 h-16 flex items-center">
          <Link href="/" className="flex items-center gap-2">
            <BrandLogo imageClassName="h-9 w-auto" />
          </Link>
          <div className="ml-auto text-sm text-zinc-400 space-x-6">
            <Link href="/" className="hover:text-white transition-colors">Home</Link>
            <Link href="/docs" className="hover:text-white transition-colors">Docs</Link>
            <Link href="/install" className="hover:text-white transition-colors">Install</Link>
          </div>
        </div>
      </nav>

      <div className="pt-24 pb-16 px-4 max-w-5xl mx-auto">
        <h1 className="text-4xl font-bold text-white mb-4 text-center">Product</h1>
        <p className="text-zinc-400 text-center max-w-2xl mx-auto mb-12">
          Onus is a governance and execution-control layer for AI coding agents.
          It governs agent actions through deterministic rules, semantic analysis, human approvals, and verified completion.
          Below is every capability the product ships with.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {features.map((feature, i) => {
            const Icon = feature.icon;
            return (
              <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 hover:border-zinc-700 transition-colors">
                <Icon className="w-6 h-6 text-accent mb-3" />
                <h3 className="font-semibold text-white mb-2">{feature.title}</h3>
                <p className="text-zinc-400 text-sm leading-relaxed">{feature.desc}</p>
              </div>
            );
          })}
        </div>

        <div className="mt-12 text-center">
          <Link href="/install" className="inline-flex items-center gap-2 px-8 py-3.5 bg-accent text-black rounded-full font-semibold hover:bg-accent-hover transition-colors text-sm">
            Install Onus <ArrowRight className="w-4 h-4" />
          </Link>
        </div>
      </div>
    </div>
  );
}
