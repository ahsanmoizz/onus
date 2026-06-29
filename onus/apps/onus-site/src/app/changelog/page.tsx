import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';
import { GitCommit, GitBranch, Tag, Calendar } from 'lucide-react';

const releases = [
  {
    version: '0.1.0',
    date: '2026-06-19',
    tag: 'current',
    changes: [
      { type: 'feat', message: 'Memory lifecycle operations — list, inspect, export, delete, archive, retention, incidents' },
      { type: 'feat', message: 'Content-aware secret detection with entropy, JWT, connection strings, private keys' },
      { type: 'feat', message: 'Ed25519 signed managed policies with install/verify/sign/revoke CLI' },
      { type: 'feat', message: 'CSRF, rate limiting, and security headers to dashboard and approval servers' },
      { type: 'feat', message: 'Full CLI workflow for human approval management' },
      { type: 'feat', message: 'L4 authority subsystem — disposable credentials with expiry and revocation' },
      { type: 'feat', message: 'Prompt Intake Guardian — analyzes ambiguous tasks into READY/CLARIFICATION_REQUIRED/REJECTED_AS_UNSAFE' },
      { type: 'feat', message: 'Deterministic rules engine with aho-corasick multi-pattern matching' },
      { type: 'feat', message: 'Semantic analysis support via OpenAI, Anthropic, and local llama.cpp providers' },
      { type: 'feat', message: 'Workspace checkpoint and rollback — per-action, group, and session-level' },
      { type: 'feat', message: 'L3 workspace isolation with bubblewrap (Linux)' },
      { type: 'feat', message: 'Receipt chain with SHA-256 Merkle tree anchoring and verification' },
      { type: 'feat', message: 'Agent handoff with complete serialized state and manifest hash verification' },
      { type: 'feat', message: 'MCP proxy for L2 action routing through Onus evaluation' },
      { type: 'feat', message: 'Session-scoped and project-scoped memory with SQLite backend' },
      { type: 'feat', message: 'IPC daemon with client/server architecture and protocol framing' },
      { type: 'feat', message: 'Audit database with SQLite and receipt query CLI' },
      { type: 'feat', message: 'CLI dashboard with health metrics, session browser, and action viewer' },
      { type: 'feat', message: 'Install scripts for Windows (PowerShell) and Linux/macOS (bash)' },
      { type: 'feat', message: 'VS Code extension for inline governance integration' },
      { type: 'feat', message: 'Python bindings for programmatic access' },
      { type: 'feat', message: 'Agent integrations: Claude Code CLI, OpenAI Codex CLI, Antigravity, Cursor IDE' },
      { type: 'fix', message: 'Secure secret handling — secrets excluded from logs, receipts, prompts, and memory' },
      { type: 'fix', message: 'Rollback compensation operations for irreversible actions with honest irreversibility reporting' },
      { type: 'fix', message: 'WebSocket reconnection and stale connection cleanup in IPC' },
    ],
  },
];

const commitHistory = [
  { hash: '9037cf0', date: '2026-06-19', message: 'feat(cli): memory lifecycle operations — list, inspect, export, delete, archive, retention, incidents' },
  { hash: 'bf31065', date: '2026-06-19', message: 'feat(security): content-aware secret detection with entropy, JWT, connection strings, private keys' },
  { hash: '0f8b527', date: '2026-06-19', message: 'feat(policy): Ed25519 signed managed policies with install/verify/sign/revoke CLI' },
  { hash: 'f3a63b4', date: '2026-06-19', message: 'feat(security): add CSRF, rate limiting, and security headers to dashboard and approval servers' },
  { hash: '11e0bea', date: '2026-06-19', message: 'feat(approvals): full CLI workflow for human approval management' },
];

function ChangelogEntry({ release }: { release: typeof releases[0] }) {
  const typeColors: Record<string, string> = {
    feat: 'bg-green-500/10 text-green-400',
    fix: 'bg-blue-500/10 text-blue-400',
    security: 'bg-red-500/10 text-red-400',
    perf: 'bg-accent/10 text-accent',
    docs: 'bg-zinc-500/10 text-zinc-400',
    refactor: 'bg-purple-500/10 text-purple-400',
    chore: 'bg-zinc-500/10 text-zinc-400',
  };

  return (
    <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 mb-6">
      <div className="flex items-center gap-3 mb-1">
        <Tag className="w-5 h-5 text-accent" />
        <h2 className="text-xl font-bold text-white">v{release.version}</h2>
        <span className="text-xs text-zinc-500 flex items-center gap-1">
          <Calendar className="w-3 h-3" />
          {release.date}
        </span>
        {release.tag === 'current' && (
          <span className="text-[10px] px-2 py-0.5 rounded-full bg-accent/10 text-accent">Latest</span>
        )}
      </div>
      <div className="mt-4 space-y-2">
        {release.changes.map((change, i) => (
          <div key={i} className="flex items-start gap-2 text-sm">
            <span className={`text-xs px-1.5 py-0.5 rounded font-mono uppercase flex-shrink-0 mt-0.5 ${typeColors[change.type] || 'bg-zinc-500/10 text-zinc-400'}`}>
              {change.type}
            </span>
            <span className="text-zinc-300">{change.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export default function ChangelogPage() {
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
          </div>
        </div>
      </nav>

      <div className="pt-24 pb-16 px-4 max-w-3xl mx-auto">
        <h1 className="text-4xl font-bold text-white mb-2">Changelog</h1>
        <p className="text-zinc-400 mb-8">
          Every release of Onus with notable changes, features, and fixes.
        </p>

        {releases.map((release, i) => (
          <ChangelogEntry key={i} release={release} />
        ))}

        <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
          <h2 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
            <GitCommit className="w-4 h-4 text-accent" />
            Recent Commits
          </h2>
          <div className="space-y-2">
            {commitHistory.map((commit, i) => (
              <div key={i} className="flex items-start gap-3 text-sm py-1">
                <code className="text-xs text-accent font-mono flex-shrink-0 w-20">{commit.hash}</code>
                <span className="text-zinc-500 text-xs flex-shrink-0 w-24">{commit.date}</span>
                <span className="text-zinc-300">{commit.message}</span>
              </div>
            ))}
          </div>
          <div className="mt-4 pt-4 border-t border-zinc-800">
            <a
              href="https://github.com/ahsanmoizz/onus/commits/main"
              className="text-sm text-accent hover:underline"
            >
              View full commit history on GitHub &rarr;
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
