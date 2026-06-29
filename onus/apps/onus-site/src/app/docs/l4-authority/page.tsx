import Link from 'next/link';

export default function L4AuthorityPage() {
  return (
    <div className="min-h-screen bg-black">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-sm border-b border-zinc-800">
        <div className="max-w-6xl mx-auto px-4 h-14 flex items-center justify-between">
          <Link href="/" className="text-white font-bold text-lg">Onus</Link>
          <div className="flex items-center gap-6 text-sm text-zinc-400">
            <Link href="/product" className="hover:text-white transition-colors">Product</Link>
            <Link href="/install" className="hover:text-white transition-colors">Install</Link>
            <Link href="/docs" className="text-accent transition-colors">Docs</Link>
          </div>
        </div>
      </nav>

      <main className="pt-20 pb-16 px-4 max-w-4xl mx-auto">
        <Link href="/docs" className="text-sm text-accent hover:underline mb-6 inline-block">&larr; Back to Docs</Link>

        <h1 className="text-3xl font-bold text-white mb-4">L4 Authority: Disposable Credentials</h1>

        <p className="text-zinc-300 leading-relaxed mb-8">
          Level 4 enforcement provides disposable, short-lived credentials that are scoped to specific
          actions and automatically revoked when no longer needed. L4 ensures that even if credentials
          are compromised, their utility is limited by strict scope boundaries, short time-to-live,
          and automatic revocation. L4 claims require Onus-controlled authority over the credential source.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Disposable Credential Model</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Traditional credential management relies on long-lived tokens with broad permissions. If
          compromised, these credentials grant attackers extended access. L4 replaces this with a
          model where each credential is issued for a single purpose, with a limited lifespan, and
          is automatically destroyed after use. This minimizes the blast radius of any credential
          compromise.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Issuing an Authority</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Authorities are issued with the <code className="text-accent">onus authority issue</code> command. Each authority specifies
          the scope of access and a time-to-live (TTL) in seconds:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
# Issue an S3 read-only credential for one hour
onus authority issue --scope &quot;s3:GetObject&quot; --ttl 3600

# Issue a credential for a specific S3 bucket
onus authority issue --scope &quot;s3:GetObject&quot; --resource &quot;arn:aws:s3:::myapp-assets&quot; --ttl 1800

# Issue a credential for deploying to a specific environment
onus authority issue --scope &quot;deploy:production&quot; --ttl 900</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Scope-Bound Access</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Each authority is bound to a specific action and resource scope. The scope defines exactly
          what the credential can do and which resources it can access. Scopes use a hierarchical
          format with support for wildcards and specific identifiers. An authority issued for
          <code className="text-accent">s3:GetObject</code> on a specific bucket cannot be used for <code className="text-accent">s3:PutObject</code> on any bucket.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Time-to-Live and Expiry</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Every authority has a configurable TTL measured in seconds. After the TTL expires, the
          credential is automatically revoked by the Onus daemon and cannot be used. Short TTLs
          (15-30 minutes) are recommended for most use cases to limit exposure:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
# Issue a credential with a 5-minute TTL for sensitive operations
onus authority issue --scope &quot;admin:restart&quot; --ttl 300

# Issue a credential with a longer TTL for batch jobs
onus authority issue --scope &quot;batch:process&quot; --ttl 86400</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Revoking an Authority</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Authorities can be revoked before their TTL expires using the revoke command. This is
          useful when a task completes early or if suspicious activity is detected:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
# Revoke a specific authority
onus authority revoke &lt;authority-id&gt;</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Automatic Expiry</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          The Onus daemon monitors active authorities and automatically revokes them when their TTL
          expires. This ensures that even if a task crashes or forgets to revoke, credentials do not
          persist indefinitely. Automatic expiry is enforced by the daemon for brokered capabilities
          rather than by trusting the agent to clean up.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">L4 Claim</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          L4 enforcement claims require Onus-controlled authority. This means Onus must have direct
          control over the credential provider (e.g., cloud provider credentials managed by Onus).
          When Onus cannot control the credential source, L4 features are unavailable and Onus reports
          the limitation. L4 is the highest enforcement level and provides the strongest guarantees
          for credential security.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Use Cases</h2>
        <p className="text-zinc-300 leading-relaxed mb-8">
          L4 authorities are ideal for deployments, database migrations, infrastructure changes, and
          any operation that requires privileged access. By issuing disposable credentials for each
          action, organizations can significantly reduce the risk of credential-based attacks and
          maintain a strict least-privilege model for AI agent operations.
        </p>
      </main>
    </div>
  );
}
