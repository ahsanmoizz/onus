import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function ProvidersPage() {
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
        <h1 className="text-3xl font-bold text-white mb-4">Providers</h1>
        <p className="text-zinc-300 mb-8">
          Onus uses providers for semantic evaluation of actions. In Deterministic-only mode, no provider is needed. In Cloud or Local modes, a provider must be configured.
        </p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Supported Providers</h2>
        <div className="space-y-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5">
            <h3 className="text-white font-medium mb-2">OpenAI</h3>
            <p className="text-zinc-400 text-sm mb-3">GPT-4o, GPT-4-turbo, GPT-4. Requires API key.</p>
            <pre className="bg-black border border-zinc-800 rounded-lg p-3 text-sm text-zinc-300 font-mono overflow-x-auto">export OPENAI_API_KEY=sk-your-key-here</pre>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5">
            <h3 className="text-white font-medium mb-2">Anthropic</h3>
            <p className="text-zinc-400 text-sm mb-3">Claude 3 Opus, Sonnet. Requires API key.</p>
            <pre className="bg-black border border-zinc-800 rounded-lg p-3 text-sm text-zinc-300 font-mono overflow-x-auto">export ANTHROPIC_API_KEY=sk-ant-your-key-here</pre>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5">
            <h3 className="text-white font-medium mb-2">Local (Ollama / llama.cpp)</h3>
            <p className="text-zinc-400 text-sm mb-3">Run locally on your hardware. No data leaves your machine.</p>
            <pre className="bg-black border border-zinc-800 rounded-lg p-3 text-sm text-zinc-300 font-mono overflow-x-auto">export OLLAMA_HOST=http://localhost:11434</pre>
          </div>
        </div>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Configuration</h2>
        <p className="text-zinc-300 mb-4">Configure via <code className="text-accent bg-zinc-900 px-1 rounded">onus setup</code>:</p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto mb-4">{`onus setup --guardian-mode cloud --provider openai
onus setup --guardian-mode cloud --provider anthropic
onus setup --guardian-mode local --provider ollama`}</pre>
        <p className="text-zinc-300 mb-4">Verify with <code className="text-accent bg-zinc-900 px-1 rounded">onus doctor</code>.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Provider Fallback</h2>
        <p className="text-zinc-300">When a provider is unavailable, Onus falls back to deterministic-only evaluation. Actions remain governed by Tier 1 rules.</p>
      </main>
    </>
  );
}
