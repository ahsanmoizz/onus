import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function TroubleshootingPage() {
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
        <h1 className="text-3xl font-bold text-white mb-4">Troubleshooting</h1>
        <p className="text-zinc-300 mb-8">Common issues, root causes, and solutions for Onus deployments.</p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Daemon Issues</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Daemon Won&apos;t Start</h3>
            <p className="text-sm text-zinc-400"><strong className="text-zinc-300">Cause:</strong> Port conflict (default 9090), or missing binary.</p>
            <p className="text-sm text-zinc-400"><strong className="text-zinc-300">Fix:</strong> Check port with <code className="text-accent bg-zinc-900 px-1 rounded">lsof -i :9090</code> on Linux or <code className="text-accent bg-zinc-900 px-1 rounded">netstat -ano | findstr :9090</code> on Windows. Ensure the binary exists with <code className="text-accent bg-zinc-900 px-1 rounded">onus --version</code>, then retry <code className="text-accent bg-zinc-900 px-1 rounded">onus start</code>.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Daemon Running but Unreachable</h3>
            <p className="text-sm text-zinc-400"><strong className="text-zinc-300">Fix:</strong> Verify with <code className="text-accent bg-zinc-900 px-1 rounded">curl http://127.0.0.1:9090/api/status</code>. Check firewall rules. Ensure daemon is listening on the correct interface.</p>
          </div>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Provider Issues</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Connection Failed</h3>
            <p className="text-sm text-zinc-400"><strong className="text-zinc-300">Causes:</strong> Wrong API key, expired key, network proxy, rate limiting.</p>
            <p className="text-sm text-zinc-400"><strong className="text-zinc-300">Fix:</strong> Verify the API key is set in the environment. Check <code className="text-accent bg-zinc-900 px-1 rounded">onus doctor</code> for provider diagnostics. For local providers, verify the endpoint is running.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Rate Limiting</h3>
            <p className="text-sm text-zinc-400">Cloud providers have rate limits. Onus retries automatically with exponential backoff. For high-volume use, consider local providers to avoid rate limits.</p>
          </div>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Build Issues</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Rust Compilation Errors</h3>
            <p className="text-sm text-zinc-400"><strong className="text-zinc-300">Cause:</strong> Rust version below 1.75.0, or missing system dependencies.</p>
            <p className="text-sm text-zinc-400"><strong className="text-zinc-300">Fix:</strong> Update Rust: <code className="text-accent bg-zinc-900 px-1 rounded">rustup update stable</code>. Install build deps: build-essential, pkg-config, libssl-dev on Linux.</p>
          </div>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Integration Issues</h2>
        <div className="space-y-4 mb-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4">
            <h3 className="text-white font-medium mb-1">Integration Not Detected</h3>
            <p className="text-sm text-zinc-400"><strong className="text-zinc-300">Fix:</strong> Run <code className="text-accent bg-zinc-900 px-1 rounded">onus doctor</code> to check integration status. Ensure the integration is properly installed. For VS Code, check the extension is enabled. For MCP hooks, verify the config file is in the correct location.</p>
          </div>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">General Approach</h2>
        <p className="text-zinc-300">When troubleshooting, follow this order: (1) Check daemon status with <code className="text-accent bg-zinc-900 px-1 rounded">onus status</code>, (2) Run <code className="text-accent bg-zinc-900 px-1 rounded">onus doctor</code>, (3) Check the audit log with <code className="text-accent bg-zinc-900 px-1 rounded">onus log --limit 20</code>, (4) verify receipts with <code className="text-accent bg-zinc-900 px-1 rounded">onus verify</code>, (5) restart with <code className="text-accent bg-zinc-900 px-1 rounded">onus restart</code> if needed.</p>
      </main>
    </>
  );
}
