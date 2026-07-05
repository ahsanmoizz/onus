import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function ProvidersPage() {
  const managedPowerShell = `$env:ONUS_SEMANTIC_PROVIDER="cloud"
$env:ONUS_SEMANTIC_ENDPOINT="https://YOUR-ONUS-GATEWAY/v1/chat/completions"
$env:ONUS_SEMANTIC_MODEL="onus-managed"
$env:ONUS_SEMANTIC_API_KEY="ONUS_CLIENT_TOKEN"
$env:ONUS_SEMANTIC_FALLBACK="fail_closed"
$env:ONUS_SEMANTIC_FAIL_CLOSED_CRITICAL="1"
$env:ONUS_SEMANTIC_PRIVACY_MODE="strict"
$env:ONUS_SEMANTIC_REDACT="1"`;

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
          Onus semantic review uses a vendor-neutral provider architecture. For public distribution, end users should
          route through the managed Onus gateway. The VPS holds the provider key; the user receives only an Onus client token.
        </p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Default: Managed Onus Gateway</h2>
        <p className="text-zinc-300 mb-4">
          The gateway exposes an OpenAI-compatible endpoint to the Onus CLI while keeping the raw model-provider credential
          server-side. This is the path for users who should not configure model keys.
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto mb-8"><code>{managedPowerShell}</code></pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Provider Types</h2>
        <div className="space-y-6">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5">
            <h3 className="text-white font-medium mb-2">Managed Gateway</h3>
            <p className="text-zinc-400 text-sm">Recommended for normal users. The Onus operator pays and controls provider access.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5">
            <h3 className="text-white font-medium mb-2">Disabled / Deterministic</h3>
            <p className="text-zinc-400 text-sm">No model call is made. Deterministic rules still block known unsafe actions.</p>
          </div>
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5">
            <h3 className="text-white font-medium mb-2">Local</h3>
            <p className="text-zinc-400 text-sm">Local adapter mode for a model you operate yourself. Configure only when available and tested.</p>
          </div>
        </div>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Verification</h2>
        <p className="text-zinc-300 mb-4">Verify your configured mode with <code className="text-accent bg-zinc-900 px-1 rounded">onus doctor</code> before routing agents through Onus.</p>
        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Provider Fallback</h2>
        <p className="text-zinc-300">Deterministic denials always remain authoritative. Do not treat an unavailable provider as approval.</p>
      </main>
    </>
  );
}
