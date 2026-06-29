import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';
import { AlertTriangle, CheckCircle, Lightbulb, Shield, Terminal, Lock, GitBranch, Key, Activity, Server } from 'lucide-react';

const limitations = [
  {
    category: 'Enforcement Coverage',
    icon: Shield,
    items: [
      {
        limitation: 'L1 (best-effort hook) only works when the agent cooperates',
        detail: 'If an agent is not configured to use the Onus hook, or deliberately bypasses it, no governance is applied. Upgrade to L2+ for enforced routing through the MCP proxy.',
        status: 'acknowledged',
        plan: 'Enforcement is intended to move from cooperative to mandatory via L2 agent integrations and L3 workspace containment.',
      },
      {
        limitation: 'Semantic analysis is provider-dependent',
        detail: 'The quality and correctness of semantic evaluation depends on the configured provider model. A compromised or malfunctioning provider could produce incorrect verdicts.',
        status: 'mitigated',
        plan: 'Deterministic-only mode bypasses semantic analysis entirely. Tier 1 BLOCK verdicts are final and override any semantic ALLOW.',
      },
      {
        limitation: 'No real-time kernel-level enforcement',
        detail: 'Onus operates at the userspace level. It evaluates actions before they reach the shell but does not intercept syscalls. A malicious process spawned outside the Onus daemon is not governed.',
        status: 'acknowledged',
        plan: 'L3 bubblewrap containment provides process-level isolation. Full kernel-level enforcement is not planned.',
      },
    ],
  },
  {
    category: 'Platform Support',
    icon: Terminal,
    items: [
      {
        limitation: 'L3 workspaces require Linux with bubblewrap',
        detail: 'Workspace isolation (bubblewrap) is only available on Linux. On Windows, L3 workspace creation falls back closed with a clear error message.',
        status: 'known',
        plan: 'WSL support on Windows provides a Linux environment for L3 workspaces.',
      },
      {
        limitation: 'Windows named pipe IPC has path length and permission constraints',
        detail: 'Windows IPC uses named pipes (\\.\pipe\onus). Long path names, restricted pipe permissions, or Windows security software may interfere with communication.',
        status: 'known',
        plan: 'Configurable pipe name and fallback to TCP localhost for environments where named pipes are constrained.',
      },
    ],
  },
  {
    category: 'Approval Workflow',
    icon: Lock,
    items: [
      {
        limitation: 'Approval server binds to localhost only',
        detail: 'The approval HTTP server binds to 127.0.0.1:9191 by default. Remote approval is not supported — an attacker with local user access could interact with the approval UI.',
        status: 'mitigated',
        plan: 'Token-based auth, CSRF protection, and rate limiting are implemented. Remote approval via TLS with client certificates is a potential future enhancement.',
      },
      {
        limitation: 'No multi-user approval workflows',
        detail: 'The current approval system is designed for a single human operator. Multi-user workflows (two-person rule, approval quorums) are not supported.',
        status: 'acknowledged',
        plan: 'Multi-user approval with role-based routing is planned for a future release.',
      },
    ],
  },
  {
    category: 'Receipt Chain',
    icon: Activity,
    items: [
      {
        limitation: 'Hash chaining provides tamper evidence, not tamper prevention',
        detail: 'The SHA-256 receipt chain detects tampering via chain verification, but does not prevent modification of the underlying storage. Chain integrity is only as strong as the storage layer.',
        status: 'mitigated',
        plan: 'External anchoring (e.g., to a transparency log or blockchain) would provide third-party tamper evidence. This is a phase 2 enhancement.',
      },
      {
        limitation: 'Merkle tree root is stored locally',
        detail: 'The session Merkle tree root is stored in the local SQLite audit database. An attacker with filesystem access could modify both the receipts and the root simultaneously.',
        status: 'acknowledged',
        plan: 'External anchoring and periodic root publication are planned to decouple root integrity from local storage.',
      },
    ],
  },
  {
    category: 'Secret Detection',
    icon: Key,
    items: [
      {
        limitation: 'Entropy-based detection produces false positives',
        detail: 'High-entropy string detection flags random UUIDs, base64-encoded data, and cryptographic material as potential secrets. Threshold tuning is required per workspace.',
        status: 'mitigated',
        plan: 'Allowlists per rule and confidence thresholds reduce false positives. Pattern-specific detection (JWT, private keys) is more precise.',
      },
      {
        limitation: 'Secrets in images and binary files are not scanned',
        detail: 'Content-aware secret detection operates on text content. Secrets embedded in images, compiled binaries, or other non-text formats are not detected.',
        status: 'acknowledged',
        plan: 'Binary file scanning is a potential future enhancement.',
      },
    ],
  },
  {
    category: 'Performance & Scale',
    icon: Server,
    items: [
      {
        limitation: 'Semantic analysis adds latency to every evaluated action',
        detail: 'Cloud provider API calls for semantic analysis add 200-2000ms per action depending on provider and model. This is noticeable during rapid agent action sequences.',
        status: 'known',
        plan: 'Local provider (llama.cpp) reduces latency. Deterministic-only mode eliminates it entirely. Caching of repeated action evaluations is under consideration.',
      },
      {
        limitation: 'Receipt chain growth is unbounded per session',
        detail: 'Every action generates a receipt record. Long-running sessions with thousands of actions produce large receipt chains and audit database growth.',
        status: 'known',
        plan: 'Session archiving, receipt pruning policies, and configurable retention windows are planned.',
      },
    ],
  },
];

function StatusBadge({ status }: { status: string }) {
  const styles: Record<string, string> = {
    acknowledged: 'bg-yellow-500/10 text-yellow-400',
    mitigated: 'bg-green-500/10 text-green-400',
    known: 'bg-blue-500/10 text-blue-400',
  };
  return (
    <span className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${styles[status] || 'bg-zinc-500/10 text-zinc-400'}`}>
      {status}
    </span>
  );
}

export default function LimitationsPage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-zinc-800">
        <div className="max-w-4xl mx-auto px-4 h-16 flex items-center">
          <Link href="/" className="flex items-center gap-2">
            <BrandLogo imageClassName="h-9 w-auto" />
          </Link>
          <div className="ml-auto text-sm text-zinc-400 space-x-6">
            <Link href="/" className="hover:text-white transition-colors">Home</Link>
            <Link href="/docs" className="hover:text-white transition-colors">Docs</Link>
            <Link href="/security" className="hover:text-white transition-colors">Security</Link>
          </div>
        </div>
      </nav>

      <div className="pt-24 pb-16 px-4 max-w-4xl mx-auto">
        <h1 className="text-4xl font-bold text-white mb-2">Limitations</h1>
        <p className="text-zinc-400 mb-8 max-w-2xl">
          Onus is under active development. This page documents current limitations honestly.
          Each item includes the current status and plans for improvement.
        </p>

        <div className="space-y-8">
          {limitations.map((group, i) => {
            const GroupIcon = group.icon;
            return (
              <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
                <div className="flex items-center gap-2 mb-4">
                  <GroupIcon className="w-5 h-5 text-accent" />
                  <h2 className="text-lg font-semibold text-white">{group.category}</h2>
                </div>
                <div className="space-y-6">
                  {group.items.map((item, j) => (
                    <div key={j} className="border-l-2 border-zinc-700 pl-4">
                      <div className="flex items-start gap-2 mb-2">
                        <AlertTriangle className="w-4 h-4 text-warning mt-0.5 flex-shrink-0" />
                        <h3 className="font-medium text-white text-sm">{item.limitation}</h3>
                      </div>
                      <p className="text-zinc-400 text-sm ml-6 mb-2">{item.detail}</p>
                      <div className="ml-6 flex items-start gap-2 text-xs">
                        <StatusBadge status={item.status} />
                        <span className="text-zinc-500 flex items-start gap-1">
                          <Lightbulb className="w-3 h-3 mt-0.5 flex-shrink-0" />
                          <span>{item.plan}</span>
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>

        <div className="mt-8 p-6 bg-accent/5 border border-accent/20 rounded-xl">
          <h2 className="text-lg font-semibold text-white mb-2 flex items-center gap-2">
            <Lightbulb className="w-4 h-4 text-accent" />
            Tracking & Roadmap
          </h2>
          <p className="text-sm text-zinc-400 mb-4">
            These limitations are tracked as GitHub issues. Feature requests and bug reports
            help prioritize improvements. Check the roadmap for planned work on each area.
          </p>
          <div className="flex gap-3">
            <a href="https://github.com/ahsanmoizz/onus/issues" className="text-sm px-4 py-2 border border-zinc-700 text-zinc-300 rounded-full font-medium hover:bg-zinc-900 transition-colors">
              GitHub Issues
            </a>
            <Link href="/docs" className="text-sm px-4 py-2 border border-zinc-700 text-zinc-300 rounded-full font-medium hover:bg-zinc-900 transition-colors">
              Documentation
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}
