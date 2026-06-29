import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function RulesPoliciesPage() {
  return (
    <div className="min-h-screen bg-black">
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
        <Link href="/docs" className="text-sm text-accent hover:underline mb-6 inline-block">&larr; Back to Docs</Link>

        <h1 className="text-3xl font-bold text-white mb-4">Rules &amp; Policies</h1>

        <p className="text-zinc-300 leading-relaxed mb-8">
          Onus uses a two-tier enforcement system to govern AI agent behavior. Tier 1 provides
          deterministic, pattern-based rules that run locally and never fail open. Tier 2 adds
          semantic review when configured for deeper intent analysis. Managed policies bundle
          rules into cryptographically signed packages that can be installed, verified, and revoked.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Tier 1: Deterministic Rules</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Tier 1 rules are pattern-matching rules that execute entirely locally with no external dependencies.
          They evaluate every action proposed by an AI agent against a set of defined patterns and produce a
          deterministic verdict. Because they never contact an external service, they cannot fail open due to
          network or provider issues.
        </p>

        <p className="text-zinc-300 leading-relaxed mb-4">
          Pattern types supported by Tier 1 rules include:
        </p>
        <ul className="list-disc pl-6 text-zinc-300 space-y-2 mb-6">
          <li><strong className="text-white">File paths</strong> &mdash; match operations on specific files, directories, or glob patterns (e.g., block writes to /etc/passwd).</li>
          <li><strong className="text-white">Shell commands</strong> &mdash; match command invocations, argument patterns, or pipelines (e.g., deny rm -rf /).</li>
          <li><strong className="text-white">Network targets</strong> &mdash; match hostnames, IP addresses, or port ranges for outbound connections.</li>
          <li><strong className="text-white">Secret patterns</strong> &mdash; detect API keys, tokens, connection strings, private keys, and JWTs via entropy and regex matching.</li>
        </ul>

        <p className="text-zinc-300 leading-relaxed mb-4">
          Each rule evaluation produces one of four verdicts:
        </p>
        <ul className="list-disc pl-6 text-zinc-300 space-y-2 mb-6">
          <li><strong className="text-white">ALLOW</strong> &mdash; the action is permitted and proceeds normally.</li>
          <li><strong className="text-white">DENY</strong> &mdash; the action is blocked and the agent receives a rejection.</li>
          <li><strong className="text-white">WARN</strong> &mdash; the action is allowed but logged with a warning for review.</li>
          <li><strong className="text-white">ESCALATE</strong> &mdash; the action is paused and queued for human approval.</li>
        </ul>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Tier 2: Semantic Heuristics</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Tier 2 adds provider-based semantic review to evaluate agent intent, detect deceptive behavior,
          and identify potential data exfiltration. This layer operates in cloud or local mode through
          a configured LLM provider. Unlike Tier 1, Tier 2 analysis is heuristic and may produce false
          positives or false negatives.
        </p>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Semantic analysis examines the context of each action, including the surrounding conversation,
          the agent&apos;s stated goals, and the sensitivity of the resources being accessed. Actions that pass
          Tier 1 but exhibit suspicious semantic patterns may be escalated for human review.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Managed Policies</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Managed policies bundle multiple rules into signed packages that can be distributed and verified.
          Each policy is signed with an Ed25519 key pair, ensuring authenticity and integrity. Onus ships
          with a set of default policies covering common security requirements.
        </p>

        <p className="text-zinc-300 leading-relaxed mb-4">
          Current rules and managed-policy CLI commands:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
# List loaded rules
onus rules list

# Install a signed policy file
onus rules install policy.toml

# Verify a policy signature
onus rules verify policy.toml

# Sign a policy with your key
onus rules sign policy.toml --key mykey.pem

# Revoke an installed policy
onus rules revoke policy.toml</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Default Rules</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Onus ships with built-in default rules that activate immediately after installation. These cover
          common destructive patterns, secret exposure prevention, and network access controls. Default
          rules can be extended or overridden by installing managed policies, but the base set ensures
          a minimum security baseline on every Onus installation.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Rule Evaluation Order</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Actions are evaluated against Tier 1 rules first. If a Tier 1 rule produces a DENY or ESCALATE
          verdict, the action is blocked or paused immediately without proceeding to Tier 2. If Tier 1
          produces ALLOW or WARN, the action proceeds to Tier 2 for semantic analysis. An ESCALATE from
          Tier 2 triggers human approval even if Tier 1 allowed the action.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Custom Rules</h2>
        <p className="text-zinc-300 leading-relaxed mb-8">
          Users can define custom Tier 1 rules using Onus&apos;s rule syntax. Custom rules are stored in the
          Onus configuration directory and are applied alongside default rules. Custom rules take precedence
          over defaults when they conflict, allowing granular control over agent behavior.
        </p>
      </main>
    </div>
  );
}
