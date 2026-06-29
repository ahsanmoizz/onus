import Link from 'next/link';

export default function GuardianModesPage() {
  return (
    <div className="min-h-screen">
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
        <Link href="/docs" className="text-sm text-zinc-400 hover:text-white transition-colors inline-flex items-center gap-1 mb-8">&larr; Back to Docs</Link>

        <h1 className="text-3xl font-bold text-white mb-4">Guardian Modes</h1>

        <p className="text-zinc-300 leading-relaxed mb-6">
          Onus offers four guardian modes that control how deeply agent actions are evaluated. The mode you choose determines whether semantic analysis is applied, what provider is used, and what level of security guarantees you receive. All modes apply deterministic pattern-based rules; the difference lies in whether and how semantic LLM evaluation is used.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Deterministic-Only Mode</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          In deterministic-only mode, Onus evaluates actions solely against pattern-based rules and policies. No API key is required, no network calls are made, and the evaluation engine runs entirely offline. This mode is ideal for air-gapped environments, CI/CD pipelines, or teams that want basic guardrails without the cost or latency of LLM evaluation.
        </p>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Deterministic rules can detect dangerous shell commands (<code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">rm -rf /</code>, <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">curl | sh</code>), disallowed file patterns (<code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">.env</code>, <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">id_rsa</code>), and known attack signatures. However, this mode cannot assess intent or detect novel attack patterns.
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">onus setup --guardian-mode deterministic-only</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Local Mode</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Local mode uses an LLM running on your own hardware via Ollama, llama.cpp, or any OpenAI-compatible local endpoint. All semantic analysis stays on your machine, so no data leaves your network. This mode is free, private, and suitable for teams that want semantic evaluation without sending code to a third-party API.
        </p>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Local mode requires the <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">OLLAMA_HOST</code> environment variable (defaults to <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">http://localhost:11434</code>) or a custom endpoint URL. Performance depends on your hardware; smaller models offer faster evaluation but may be less accurate for complex risk assessments.
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">onus setup --guardian-mode local</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Cloud Mode</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Cloud mode sends actions to a cloud LLM provider for semantic evaluation. This provides the most capable analysis, supporting GPT-4o (OpenAI) and Claude 3 models (Anthropic). Cloud mode is recommended when you need the highest accuracy in risk assessment and policy compliance checking.
        </p>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Cloud mode requires an API key set as an environment variable. For OpenAI use <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">OPENAI_API_KEY</code>; for Anthropic use <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">ANTHROPIC_API_KEY</code>. The provider is configured during <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">onus setup</code> and can be changed later.
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">onus setup --guardian-mode cloud</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Disabled Mode</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Disabled mode turns off all enforcement. Actions pass through without evaluation or logging. This mode is useful for development, testing, or when you need to temporarily bypass governance. It is not recommended for production use.
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">onus setup --guardian-mode disabled</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Choosing a Mode</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Consider the following factors when selecting a guardian mode:
        </p>
        <ul className="list-disc list-inside text-zinc-300 space-y-2 mb-6 ml-4">
          <li><strong className="text-white">Security needs:</strong> How critical is the codebase? Production infrastructure requires stronger evaluation.</li>
          <li><strong className="text-white">Budget:</strong> Cloud mode incurs API costs per evaluation. Local mode is free but requires hardware.</li>
          <li><strong className="text-white">Latency:</strong> Deterministic-only mode adds negligible latency. Cloud evaluation adds 1&ndash;3 seconds per action.</li>
          <li><strong className="text-white">Data privacy:</strong> If code cannot leave your network, use deterministic-only or local mode.</li>
          <li><strong className="text-white">Offline requirements:</strong> Air-gapped environments require deterministic-only mode.</li>
        </ul>

        <div className="border-t border-zinc-800 mt-12 pt-6">
          <p className="text-sm text-zinc-500">
            See the <Link href="/docs/providers" className="text-accent hover:underline">Providers</Link> page for detailed configuration of cloud and local LLM backends.
          </p>
        </div>
      </main>
    </div>
  );
}
