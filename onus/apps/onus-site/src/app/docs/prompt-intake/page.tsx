import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function PromptIntakePage() {
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
        <h1 className="text-3xl font-bold text-white mb-4">Prompt Intake Guardian</h1>
        <p className="text-zinc-300 mb-8">Converts ambiguous user prompts into structured, bounded task contracts before any action executes.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">How It Works</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">1. Analysis</h3>
            <p className="text-sm text-zinc-400">Analyzed against deterministic rules (Tier 1) and semantic heuristics (Tier 2) to identify actions, targets, and risks.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">2. Classification</h3>
            <p className="text-sm text-zinc-400">Classified as READY, READY_WITH_SAFE_CONTRACT, CLARIFICATION_REQUIRED, or REJECTED_AS_UNSAFE.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">3. Contract Generation</h3>
            <p className="text-sm text-zinc-400">If READY, a structured contract is generated with actions, risk level, change budget, and permissions.</p>
          </div>
        </div>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Outcomes</h2>
        <div className="space-y-4 mb-6">
          <div className="flex items-start gap-3 p-4 bg-green-900/10 border border-green-800/30 rounded-xl">
            <div className="w-2 h-2 rounded-full bg-green-500 mt-2 flex-shrink-0" /><div>
              <h3 className="text-green-400 font-medium mb-1">READY</h3>
              <p className="text-sm text-zinc-400">Safe and unambiguous. Agent proceeds autonomously.</p>
            </div>
          </div>
          <div className="flex items-start gap-3 p-4 bg-blue-900/10 border border-blue-800/30 rounded-xl">
            <div className="w-2 h-2 rounded-full bg-blue-500 mt-2 flex-shrink-0" /><div>
              <h3 className="text-blue-400 font-medium mb-1">READY_WITH_SAFE_CONTRACT</h3>
              <p className="text-sm text-zinc-400">Safe with bounded contract for scope and evidence.</p>
            </div>
          </div>
          <div className="flex items-start gap-3 p-4 bg-yellow-900/10 border border-yellow-800/30 rounded-xl">
            <div className="w-2 h-2 rounded-full bg-yellow-500 mt-2 flex-shrink-0" /><div>
              <h3 className="text-yellow-400 font-medium mb-1">CLARIFICATION_REQUIRED</h3>
              <p className="text-sm text-zinc-400">Ambiguous. Onus returns clarifying questions.</p>
            </div>
          </div>
          <div className="flex items-start gap-3 p-4 bg-red-900/10 border border-red-800/30 rounded-xl">
            <div className="w-2 h-2 rounded-full bg-red-500 mt-2 flex-shrink-0" /><div>
              <h3 className="text-red-400 font-medium mb-1">REJECTED_AS_UNSAFE</h3>
              <p className="text-sm text-zinc-400">Violates deterministic rules. Blocked.</p>
            </div>
          </div>
        </div>
      </main>
    </>
  );
}
