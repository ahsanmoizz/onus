import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

const steps = [
  {
    title: '1. Install the CLI',
    body: 'Install Onus from the platform script or build from source.',
    code: 'onus --version\nonus doctor',
  },
  {
    title: '2. Start Onus',
    body: 'Start the daemon before using the console or routed integrations.',
    code: 'onus start\nonus status',
  },
  {
    title: '3. Open the local admin console',
    body: 'The admin panel is local. Keep the token private.',
    code: 'onus console --port 3001\n# open the printed URL',
  },
  {
    title: '4. Check a task before an agent starts',
    body: 'Prompt Intake Guardian classifies the request and proposes a safe task contract when possible.',
    code: 'onus intake --prompt "Fix the login bug and keep tests enabled." --provider disabled',
  },
  {
    title: '5. Connect an agent through a routed surface',
    body: 'Only routed actions are governed. Pick the integration you actually use.',
    code: 'onus setup --claude\nonus setup --codex\nonus setup --cursor',
  },
  {
    title: '6. Run and review evidence',
    body: 'After work, inspect audit rows, verify receipts, and use rollback only where supported.',
    code: 'onus log --limit 20\nonus verify\nonus checkpoint list',
  },
];

export default function QuickStartPage() {
  return (
    <div className="min-h-screen bg-black text-zinc-100">
      <nav className="fixed inset-x-0 top-0 z-50 border-b border-zinc-800 bg-black/85 backdrop-blur-sm">
        <div className="mx-auto flex h-14 max-w-6xl items-center justify-between px-4">
          <Link href="/" className="flex items-center" aria-label="Onus home">
            <BrandLogo imageClassName="h-9 w-auto" />
          </Link>
          <div className="flex items-center gap-6 text-sm text-zinc-400">
            <Link href="/install" className="hover:text-white">Install</Link>
            <Link href="/login" className="hover:text-white">Access</Link>
            <Link href="/docs" className="text-accent">Docs</Link>
          </div>
        </div>
      </nav>

      <main className="mx-auto max-w-4xl px-4 pb-16 pt-20">
        <Link href="/docs" className="mb-8 inline-flex items-center gap-1 text-sm text-zinc-400 hover:text-white">&larr; Back to Docs</Link>
        <h1 className="mb-4 text-3xl font-bold text-white">Quick Start</h1>
        <p className="mb-8 leading-7 text-zinc-400">
          This is the clean path for using Onus today: install the CLI, start the daemon,
          open the local console, then route an agent through Onus. Direct actions outside Onus are not governed.
        </p>

        <div className="space-y-6">
          {steps.map((step) => (
            <section key={step.title} className="rounded-lg border border-zinc-800 bg-zinc-900/45 p-5">
              <h2 className="mb-2 text-xl font-semibold text-white">{step.title}</h2>
              <p className="mb-4 text-sm leading-6 text-zinc-400">{step.body}</p>
              <pre className="overflow-x-auto rounded-md border border-zinc-800 bg-black p-4 text-sm leading-6 text-zinc-300"><code>{step.code}</code></pre>
            </section>
          ))}
        </div>

        <section className="mt-8 rounded-lg border border-zinc-800 bg-zinc-950 p-5">
          <h2 className="mb-3 text-lg font-semibold text-white">What is production-like use?</h2>
          <ul className="space-y-2 text-sm leading-6 text-zinc-400">
            <li>Use deterministic policies first; add semantic providers only after provider credentials are configured outside the browser.</li>
            <li>Use the console token and do not expose local dashboard or approval ports publicly.</li>
            <li>Use Linux L3 workspaces before claiming process/filesystem/network containment.</li>
            <li>Use L4 authority only for disposable controlled operations until independently verified for your environment.</li>
            <li>Keep approval and audit receipts, but do not call the local SQLite hash chain immutable.</li>
          </ul>
        </section>

        <div className="mt-8 flex gap-3">
          <Link href="/docs/cli-reference" className="rounded-full bg-accent px-5 py-3 text-sm font-semibold text-black hover:bg-accent-hover">
            CLI Reference
          </Link>
          <Link href="/login" className="rounded-full border border-zinc-700 px-5 py-3 text-sm text-zinc-200 hover:bg-zinc-900">
            Access Console
          </Link>
        </div>
      </main>
    </div>
  );
}
