import Link from 'next/link';

export default function ReceiptsAuditPage() {
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

        <h1 className="text-3xl font-bold text-white mb-4">Receipts &amp; Audit Trail</h1>

        <p className="text-zinc-300 leading-relaxed mb-8">
          Every action governed by Onus produces a tamper-evident receipt that is recorded in an
          append-only audit log. Receipts form a Merkle chain, where each receipt contains the hash
          of the previous receipt. This structure makes it possible to detect any tampering with the
          audit history, providing strong evidence of what happened, when, and what decision was made.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Receipt Structure</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Each receipt captures the complete evaluation result for a single action:
        </p>
        <ul className="list-disc pl-6 text-zinc-300 space-y-2 mb-6">
          <li><strong className="text-white">Action hash</strong> &mdash; a cryptographic hash of the canonical action payload, ensuring the action is bound to the receipt.</li>
          <li><strong className="text-white">Verdict</strong> &mdash; the evaluation result (ALLOW, DENY, WARN, or ESCALATE).</li>
          <li><strong className="text-white">Rule match</strong> &mdash; which rule or rules produced the verdict.</li>
          <li><strong className="text-white">Timestamp</strong> &mdash; when the action was evaluated (in UTC).</li>
          <li><strong className="text-white">Previous receipt hash</strong> &mdash; links each receipt to its predecessor, forming the chain.</li>
          <li><strong className="text-white">Receipt signature</strong> &mdash; signed by the Onus daemon for non-repudiation.</li>
        </ul>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Merkle Chain Integrity</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          The audit log is structured as a Merkle chain, where each receipt references the hash of the
          receipt before it. Modifying, deleting, or reordering any receipt in the chain breaks the
          hash links, making tampering detectable. The chain root hash is periodically committed,
          providing a snapshot that can be verified at any time.
        </p>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Chain structure visualization:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
Receipt[1] — hash( action, verdict, rule, ts, prev=null )
    |
    v
Receipt[2] — hash( action, verdict, rule, ts, prev=hash(Receipt[1]) )
    |
    v
Receipt[3] — hash( action, verdict, rule, ts, prev=hash(Receipt[2]) )</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Verifying Chain Integrity</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Onus provides a verify command to check the integrity of the receipt chain:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
# Verify the full receipt chain
onus verify

# Verify the chain from a specific receipt
onus verify --from &lt;receipt-id&gt;</pre>
        <p className="text-zinc-300 leading-relaxed mb-4">
          If any receipt has been tampered with, the verify command will report the exact location
          and nature of the integrity failure.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Viewing the Audit Log</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          The <code className="text-accent">onus log</code> command provides a human-readable view of the audit trail with filtering options:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
# View all log entries
onus log

# Filter by session
onus log --session &lt;session-id&gt;

# Filter by verdict
onus log --verdict DENY

# Filter by rule
onus log --rule &quot;block-rm-rf&quot;

# Filter by time range
onus log --since &quot;2025-01-01&quot; --until &quot;2025-01-31&quot;</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Append-Only Semantics</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Receipts are append-only. Once written, a receipt cannot be modified or deleted through
          normal Onus operations. Combined with Merkle chaining, this provides strong tamper evidence.
          However, hash chaining alone does not equal immutability&mdash;see the Security Invariants
          documentation for a detailed discussion of what Onus does and does not guarantee.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Exporting Receipts</h2>
        <p className="text-zinc-300 leading-relaxed mb-8">
          Receipts can be exported in JSON format for external processing, compliance reporting, or
          archival. The export includes the full receipt data and chain metadata:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
# Export all receipts in the current session
onus log --session &lt;session-id&gt; --format json --output receipts.json</pre>
      </main>
    </div>
  );
}
