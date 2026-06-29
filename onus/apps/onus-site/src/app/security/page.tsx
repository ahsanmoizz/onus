import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';
import { Shield, Lock, Zap, AlertTriangle, CheckCircle, Server, Key, Users, FileText, GitBranch, Activity, Database } from 'lucide-react';

export default function SecurityPage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-zinc-800">
        <div className="max-w-5xl mx-auto px-4 h-16 flex items-center">
          <Link href="/" className="flex items-center gap-2">
            <BrandLogo imageClassName="h-9 w-auto" />
          </Link>
          <div className="ml-auto text-sm text-zinc-400 space-x-6">
            <Link href="/" className="hover:text-white transition-colors">Home</Link>
            <Link href="/product" className="hover:text-white transition-colors">Product</Link>
            <Link href="/docs" className="hover:text-white transition-colors">Docs</Link>
          </div>
        </div>
      </nav>

      <div className="pt-24 pb-16 px-4 max-w-4xl mx-auto">
        <h1 className="text-4xl font-bold text-white mb-4">Security Model</h1>
        <p className="text-zinc-400 mb-8 max-w-2xl">
          Onus is built with security as the primary design constraint. This page documents the
          trust model, threat coverage, enforcement levels, and residual risks.
        </p>

        {/* Assets */}
        <section className="mb-10">
          <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-2">
            <Database className="w-5 h-5 text-accent" />
            Protected Assets
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {[
              { title: 'Source Code & Configuration', desc: 'All files in the workspace and protected system paths (/etc, ~/.ssh, .env, etc.)' },
              { title: 'Credentials & Secrets', desc: 'API keys, tokens, connection strings, private keys — detected by entropy analysis and pattern matching' },
              { title: 'Git History', desc: 'Branch state, commit history, remote configuration. Destructive git operations are blocked or escalated.' },
              { title: 'External Services', desc: 'Database connections, cloud APIs, package registries — controlled via network policy and L4 disposable credentials' },
              { title: 'Audit Integrity', desc: 'Receipt chain and session audit trail — tamper-evident via SHA-256 Merkle tree hashing' },
              { title: 'Execution Environment', desc: 'Filesystem, network, process namespace — isolated at L3 via bubblewrap containment' },
            ].map((asset, i) => (
              <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4">
                <h3 className="font-semibold text-white text-sm mb-1">{asset.title}</h3>
                <p className="text-zinc-400 text-xs leading-relaxed">{asset.desc}</p>
              </div>
            ))}
          </div>
        </section>

        {/* Trust Boundaries */}
        <section className="mb-10">
          <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-2">
            <Shield className="w-5 h-5 text-accent" />
            Trust Boundaries
          </h2>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <div className="space-y-4">
              {[
                { boundary: 'Agent → Onus', trust: 'Untrusted', detail: 'All agent actions are evaluated before execution. Onus does not trust the agent.' },
                { boundary: 'Onus → Host', trust: 'Controlled', detail: 'Onus runs on the host with user-level permissions. Actions are filtered before reaching the shell.' },
                { boundary: 'Onus → Provider API', trust: 'TLS-protected', detail: 'Semantic analysis data is sent to the configured provider over TLS. Payloads may contain task context.' },
                { boundary: 'Human → Approval Server', trust: 'Local', detail: 'Approval UI runs on localhost:9191 with token auth, CSRF protection, and rate limiting.' },
                { boundary: 'L3 Workspace → Host', trust: 'Isolated', detail: 'bubblewrap provides filesystem/network isolation. Host paths are explicitly excluded from the container.' },
              ].map((item, i) => (
                <div key={i} className="flex items-start gap-3">
                  <div className="w-2 h-2 rounded-full bg-accent mt-1.5 flex-shrink-0" />
                  <div>
                    <p className="text-sm font-medium text-white">{item.boundary}</p>
                    <p className="text-xs text-zinc-500"><span className="text-accent">{item.trust}</span> — {item.detail}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Threat Coverage */}
        <section className="mb-10">
          <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-2">
            <AlertTriangle className="w-5 h-5 text-accent" />
            Threats Covered
          </h2>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-800">
                  <th className="text-left text-zinc-400 font-medium pb-2 pr-4">Threat</th>
                  <th className="text-left text-zinc-400 font-medium pb-2 pr-4">Coverage</th>
                  <th className="text-left text-zinc-400 font-medium pb-2">Mechanism</th>
                </tr>
              </thead>
              <tbody className="text-zinc-300">
                {[
                  ['Destructive file operations', 'BLOCK', 'Tier 1 deterministic patterns (rm -rf, dd, mkfs, etc.)'],
                  ['Secret exfiltration', 'BLOCK / ESCALATE', 'Entropy analysis, pattern matching, JWT/private key detection'],
                  ['Credential leakage to logs', 'BLOCK', 'Content-aware secret detection on file_write actions'],
                  ['Force push / branch deletion', 'BLOCK', 'Git-specific deterministic patterns'],
                  ['Protected path writes', 'BLOCK', 'Protected paths list (/etc, ~/.ssh, .env, etc.)'],
                  ['Mass file deletion', 'ESCALATE', 'Heuristic detection of bulk delete patterns'],
                  ['Large shell payloads', 'ESCALATE', 'Size-based heuristic (>2000 chars suspicious)'],
                  ['Dangerous curl patterns', 'BLOCK', 'Shell pipe-to-shell detection'],
                  ['Database destruction', 'BLOCK / ESCALATE', 'DROP TABLE, DROP DATABASE patterns with allowlist'],
                  ['Unauthorized package install', 'ESCALATE', 'Pattern matching against package manager commands'],
                  ['Implied consent bypass', 'BLOCK', 'LLM-crafted implied consent patterns detected'],
                  ['Receipt tampering', 'DETECT', 'SHA-256 hash chain with Merkle tree root'],
                  ['Provider API key theft', 'BLOCK', 'API key pattern detection on write and shell actions'],
                  ['Tampered policy injection', 'REJECT', 'Ed25519 signature verification on managed policies'],
                ].map((row, i) => (
                  <tr key={i} className="border-b border-zinc-800/50">
                    <td className="py-2 pr-4 text-white">{row[0]}</td>
                    <td className="py-2 pr-4">
                      <span className={`text-xs px-2 py-0.5 rounded-full ${
                        row[1] === 'BLOCK' ? 'bg-red-500/10 text-red-400' :
                        row[1] === 'ESCALATE' ? 'bg-accent/10 text-accent' :
                        row[1] === 'DETECT' ? 'bg-blue-500/10 text-blue-400' :
                        'bg-green-500/10 text-green-400'
                      }`}>
                        {row[1]}
                      </span>
                    </td>
                    <td className="py-2 text-zinc-400 text-xs">{row[2]}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Deterministic Rules */}
        <section className="mb-10">
          <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-2">
            <Lock className="w-5 h-5 text-green-400" />
            Deterministic Rules (Tier 1)
          </h2>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <p className="text-zinc-400 text-sm mb-4">
              Tier 1 rules are always-on, precompiled regex patterns that execute before any semantic evaluation.
              A Tier 1 BLOCK verdict is absolute — it cannot be overridden by an LLM, a policy change, or an
              approval. The only way to resolve a Tier 1 BLOCK is to modify the action to not match the pattern.
            </p>
            <h3 className="font-semibold text-white text-sm mb-2">Key design properties:</h3>
            <ul className="space-y-2 text-sm text-zinc-300">
              {[
                'Uses aho-corasick for fast multi-pattern matching on shell commands',
                'Short-circuits on first BLOCK match — no further evaluation needed',
                'Patterns compiled at startup from bundled and user-provided rule files',
                'Each pattern carries a human-readable correction suggesting the proper alternative',
                'Allowlists exist per rule (e.g., allowed DROP TABLE on specific databases)',
                'Reversibility is classified per rule (reversible / irreversible / compensation_required)',
              ].map((item, i) => (
                <li key={i} className="flex items-start gap-2">
                  <CheckCircle className="w-3.5 h-3.5 text-green-400 mt-0.5 flex-shrink-0" />
                  <span>{item}</span>
                </li>
              ))}
            </ul>
          </div>
        </section>

        {/* Semantic Advice */}
        <section className="mb-10">
          <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-2">
            <Zap className="w-5 h-5 text-accent" />
            Semantic Analysis (Tier 2)
          </h2>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <p className="text-zinc-400 text-sm mb-4">
              Tier 2 evaluation uses a configured provider (OpenAI, Anthropic, or local llama.cpp) to
              analyze actions for contextual risks that pattern matching alone cannot detect. The
              semantic evaluator produces a verdict (ALLOW, BLOCK, ESCALATE) with structured reasoning
              and a correction if the action is rejected.
            </p>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mt-4">
              <div className="bg-black/30 rounded-lg p-4">
                <h4 className="text-xs font-semibold text-white uppercase tracking-wider mb-2">Semantic checks performed</h4>
                <ul className="text-xs text-zinc-400 space-y-1">
                  <li>Action risk classification (low / medium / high / critical)</li>
                  <li>Contextual intent analysis</li>
                  <li>Data destruction risk scoring</li>
                  <li>Cross-file modification conflict detection</li>
                  <li>Network interaction risk evaluation</li>
                </ul>
              </div>
              <div className="bg-black/30 rounded-lg p-4">
                <h4 className="text-xs font-semibold text-white uppercase tracking-wider mb-2">Provider modes</h4>
                <ul className="text-xs text-zinc-400 space-y-1">
                  <li><strong className="text-white">Cloud:</strong> OpenAI / Anthropic API with full analysis</li>
                  <li><strong className="text-white">Local:</strong> llama.cpp on localhost</li>
                  <li><strong className="text-white">Deterministic-only:</strong> No semantic analysis, Tier 1 only</li>
                  <li><strong className="text-white">Disabled:</strong> No governance at all (not recommended)</li>
                </ul>
              </div>
            </div>
          </div>
        </section>

        {/* L1-L4 Enforcement */}
        <section className="mb-10">
          <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-2">
            <Activity className="w-5 h-5 text-blue-400" />
            L1–L4 Enforcement Levels
          </h2>
          <div className="space-y-4">
            {[
              { level: 'L1', name: 'Best-Effort Hook', claim: 'Cooperative agent hook via shell integration', label: 'BEST-EFFORT', color: 'text-yellow-400', bg: 'bg-yellow-500/10' },
              { level: 'L2', name: 'Onus-Routed Actions', claim: 'Actions routed through Onus MCP proxy for deterministic + semantic evaluation', label: 'ENFORCED', color: 'text-accent', bg: 'bg-accent/10' },
              { level: 'L3', name: 'Process / FS / Net Containment', claim: 'bubblewrap container with filesystem isolation, network policy, resource limits, environment filtering', label: 'CONTAINED', color: 'text-blue-400', bg: 'bg-blue-500/10' },
              { level: 'L4', name: 'Controlled Authority', claim: 'Disposable short-lived credentials with exact payload binding, human approval, automatic expiry, and revocation', label: 'AUTHORITY', color: 'text-green-400', bg: 'bg-green-500/10' },
            ].map((item, i) => (
              <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
                <div className="flex items-start justify-between mb-2">
                  <div>
                    <span className={`inline-block text-xs px-2 py-0.5 rounded-full font-mono ${item.bg} ${item.color} mb-1`}>
                      {item.level}
                    </span>
                    <h3 className="font-semibold text-white">{item.name}</h3>
                  </div>
                  <span className={`text-xs px-2 py-0.5 rounded-full font-mono ${item.bg} ${item.color}`}>
                    {item.label}
                  </span>
                </div>
                <p className="text-zinc-400 text-sm">{item.claim}</p>
              </div>
            ))}
          </div>
        </section>

        {/* Bypasses and Residual Risks */}
        <section className="mb-10">
          <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-2">
            <AlertTriangle className="w-5 h-5 text-warning" />
            Bypasses and Residual Risks
          </h2>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <p className="text-zinc-400 text-sm mb-4">
              Onus significantly reduces the risk of autonomous agent damage but cannot eliminate all
              risk. The following bypasses and residual risks are acknowledged:
            </p>
            <div className="space-y-3 text-sm">
              {[
                { risk: 'L1 only mode', desc: 'If the agent does not use the cooperative hook, no governance is applied. Upgrade to L2+ for enforced routing.' },
                { risk: 'Semantic provider compromise', desc: 'If the configured semantic provider API is compromised or returns malicious results, the semantic evaluation layer is untrustworthy. Fall back to deterministic-only mode.' },
                { risk: 'Zero-day exploit in bubblewrap', desc: 'L3 isolation depends on bubblewrap correctness. A kernel or bubblewrap escape would break container boundaries. This is a host OS security dependency.' },
                { risk: 'Approval UI localhost binding', desc: 'The approval server binds to localhost:9191. If an attacker gains local user access, they could interact with the approval UI.' },
                { risk: 'Side-channel leaks', desc: 'Memory contents, receipt data, and action payloads are stored on disk. Full disk encryption is recommended but not enforced by Onus.' },
                { risk: 'Unpatterned destructive commands', desc: 'The deterministic rule set is comprehensive but not exhaustive. New destructive patterns may bypass Tier 1 and require a Tier 2 semantic block.' },
                { risk: 'Prompt injection via intake', desc: 'The prompt intake guardian analyzes task descriptions but is itself an LLM call. Prompt injection against the guardian is a residual risk.' },
                { risk: 'Network egress from L3', desc: 'L3 workspaces can be configured with permissive network policies. Audit and restrict network egress per workspace.' },
              ].map((item, i) => (
                <div key={i} className="flex items-start gap-3">
                  <AlertTriangle className="w-4 h-4 text-warning mt-0.5 flex-shrink-0" />
                  <div>
                    <p className="font-medium text-white">{item.risk}</p>
                    <p className="text-zinc-400 text-xs">{item.desc}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Cryptographic Guarantees */}
        <section className="mb-10">
          <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-2">
            <Key className="w-5 h-5 text-accent" />
            Cryptographic Guarantees
          </h2>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              {[
                { title: 'Receipt Chain', desc: 'Every action record links to the previous via SHA-256 hash of the canonical action payload. The chain is verified on every `onus verify` command.' },
                { title: 'Merkle Tree Root', desc: 'Each session produces a Merkle tree over all action hashes. The root hash can be externally anchored for third-party verification.' },
                { title: 'Ed25519 Policy Signing', desc: 'Managed policies are signed with Ed25519. Onus verifies the signature before loading. Unsigned or tampered policies are rejected.' },
                { title: 'Canonical Payload Hashing', desc: 'Approval and authority operations bind to a SHA-256 hash of the exact canonical action payload. Modified payloads require new approval.' },
                { title: 'Environment Identity', desc: 'Production actions require verified environment identity. The identity is included in approval receipts and authority capabilities.' },
                { title: 'Handoff Manifest Hash', desc: 'Agent handoff manifests include a SHA-256 hash of the serialized state. The receiving agent verifies integrity before continuing.' },
              ].map((item, i) => (
                <div key={i} className="bg-black/30 rounded-lg p-4">
                  <h3 className="font-semibold text-white text-sm mb-1">{item.title}</h3>
                  <p className="text-zinc-400 text-xs">{item.desc}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <div className="mt-8 p-6 bg-accent/5 border border-accent/20 rounded-xl">
          <h2 className="text-lg font-semibold text-white mb-2">Responsible Disclosure</h2>
          <p className="text-sm text-zinc-400 mb-4">
            If you discover a security vulnerability in Onus, please report it privately via
            GitHub Security Advisories or by email. We do not have a bug bounty program but
            will respond to and fix verified security issues promptly.
          </p>
          <a href="https://github.com/ahsanmoizz/onus/security/advisories" className="text-sm px-4 py-2 border border-zinc-700 text-zinc-300 rounded-full font-medium hover:bg-zinc-900 transition-colors inline-block">
            Report a Vulnerability
          </a>
        </div>
      </div>
    </div>
  );
}
