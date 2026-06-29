import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function LimitationsPage() {
  return (
    <>
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
        <h1 className="text-3xl font-bold text-white mb-4">Limitations</h1>
        <p className="text-zinc-300 mb-8">Onus is under active development. The following limitations are known and being addressed in planned releases.</p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Platform Limitations</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">No macOS Support</h3>
            <p className="text-sm text-zinc-400">Onus does not support macOS. Only Linux x86_64 and Windows x86_64 are supported. ARM architectures (including Apple Silicon and ARM64 Linux) are not supported.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">L3 Containment is Linux-only</h3>
            <p className="text-sm text-zinc-400">L3 workspace containment requires bubblewrap, which is Linux-only. On Windows, L3 is not available — use L2 or Docker Desktop via WSL2 for container-based isolation.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Cross-platform Consistency</h3>
            <p className="text-sm text-zinc-400">Some features behave differently across Linux and Windows due to underlying OS capabilities (file permissions, process isolation, signal handling).</p>
          </div>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Performance Limitations</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Semantic Analysis Latency</h3>
            <p className="text-sm text-zinc-400">Cloud-based semantic evaluation adds 1-5 seconds per action. Local LLM evaluation is faster but less capable. Deterministic-only mode has near-zero latency.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Memory Context Window</h3>
            <p className="text-sm text-zinc-400">Project memory is limited by available storage and the agent&apos;s context window. Very large memory stores may need manual pruning.</p>
          </div>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Provider Limitations</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Provider Keys Required for Semantic Mode</h3>
            <p className="text-sm text-zinc-400">Semantic (LLM-based) evaluation requires a provider API key. Deterministic mode — which enforces rules, patterns, and policies without any AI — works fully offline and requires no provider key.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Provider-dependent Quality</h3>
            <p className="text-sm text-zinc-400">Semantic evaluation quality depends on the provider model. Smaller local models may produce less accurate risk assessments. Cloud providers may have outages.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">False Positives/Negatives</h3>
            <p className="text-sm text-zinc-400">Semantic heuristics can produce false positives (safe action flagged as risky) or false negatives (risky action allowed). Tier 1 deterministic rules never produce false negatives for known patterns.</p>
          </div>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Integration Limitations</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">VS Code Extension — Partial</h3>
            <p className="text-sm text-zinc-400">The VS Code extension is implemented but has limited testing and no setup/doctor integration. Use CLI or MCP integrations for reliable governance.</p>
          </div>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Security Limitations</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Provider Compromise</h3>
            <p className="text-sm text-zinc-400">If the LLM provider is compromised, semantic evaluation could be manipulated. Deterministic rules provide a backstop for known patterns.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Side Channels</h3>
            <p className="text-sm text-zinc-400">Onus cannot prevent side-channel data exfiltration (timing, power, electromagnetic). These require physical security measures.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Physical Access</h3>
            <p className="text-sm text-zinc-400">An attacker with physical access to the machine can bypass all software enforcement levels.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">No Signed Code Signing Certificates</h3>
            <p className="text-sm text-zinc-400">Onus binaries are not signed with code signing certificates. Users should verify checksums and signatures after download before running the binary.</p>
          </div>
        </div>
      </main>
    </>
  );
}
