import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function RunningGovernedTasksPage() {
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
        <h1 className="text-3xl font-bold text-white mb-4">Running Governed Tasks</h1>
        <p className="text-zinc-300 mb-8">Execute AI agent tasks under Onus governance with full audit tracking, action evaluation, and enforcement.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Before a Task</h2>
        <p className="text-zinc-300 mb-4">Analyze the request before giving it to an agent. This creates or proposes a bounded task contract.</p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto mb-4">onus intake --prompt &quot;add input validation to src/main.rs and keep tests enabled&quot; --provider disabled</pre>
        <p className="text-zinc-300 mb-4">A session summary can be viewed later with <code className="text-accent bg-zinc-900 px-1 rounded">onus session &lt;session-id&gt;</code>.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Task Execution</h2>
        <p className="text-zinc-300 mb-4">Use an Onus-routed surface. Direct actions outside Onus are not governed.</p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto mb-4">onus run -- cargo test</pre>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Monitoring</h2>
        <p className="text-zinc-300 mb-4">Check daemon status with <code className="text-accent bg-zinc-900 px-1 rounded">onus status</code>. View actions with <code className="text-accent bg-zinc-900 px-1 rounded">onus log --limit 20</code>. The local console started by <code className="text-accent bg-zinc-900 px-1 rounded">onus console</code> provides the admin dashboard.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Verdicts</h2>
        <p className="text-zinc-300">Each evaluated action receives a verdict: ALLOW (passed all rules), WARN (passed but flagged), DENY (blocked by rule), or ESCALATE (requires human approval).</p>
      </main>
    </>
  );
}
