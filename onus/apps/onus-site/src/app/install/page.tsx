import Link from 'next/link';
import { ArrowRight, CheckCircle, Copy, Download, Shield, Terminal } from 'lucide-react';
import { BrandLogo } from '@/components/brand-logo';

const windowsInstall = `Invoke-WebRequest -Uri "https://raw.githubusercontent.com/ahsanmoizz/onus/main/onus/install/install-onus.ps1" -OutFile "install-onus.ps1"
powershell -ExecutionPolicy Bypass -File install-onus.ps1
onus doctor`;

const linuxInstall = `curl -fsSL https://raw.githubusercontent.com/ahsanmoizz/onus/main/onus/install/install-onus.sh -o install-onus.sh
bash install-onus.sh
onus doctor`;

const sourceBuild = `git clone https://github.com/ahsanmoizz/onus.git
cd onus/onus
cargo build --release
./target/release/onus doctor`;

function CodeBlock({ label, code }: { label: string; code: string }) {
  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-950">
      <div className="flex items-center justify-between border-b border-zinc-800 px-4 py-2">
        <span className="text-xs font-medium text-zinc-400">{label}</span>
        <span className="inline-flex items-center gap-1 text-xs text-zinc-600">
          <Copy className="h-3 w-3" />
          copy from page
        </span>
      </div>
      <pre className="overflow-x-auto p-4 text-sm leading-6 text-zinc-300"><code>{code}</code></pre>
    </div>
  );
}

export default function InstallPage() {
  return (
    <div className="min-h-screen bg-black text-zinc-100">
      <nav className="fixed inset-x-0 top-0 z-50 border-b border-zinc-800 bg-black/85 backdrop-blur-md">
        <div className="mx-auto flex h-16 max-w-5xl items-center px-4">
          <Link href="/" className="flex items-center" aria-label="Onus home">
            <BrandLogo imageClassName="h-10 w-auto" />
          </Link>
          <div className="ml-auto flex items-center gap-5 text-sm text-zinc-400">
            <Link href="/download" className="hover:text-white">Download</Link>
            <Link href="/docs/quick-start" className="hover:text-white">Quick Start</Link>
            <Link href="/login" className="hover:text-white">Access</Link>
          </div>
        </div>
      </nav>

      <main className="mx-auto max-w-5xl px-4 pb-20 pt-28">
        <div className="mb-10 max-w-3xl">
          <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-accent/20 bg-accent/10 px-3 py-1 text-xs text-accent">
            <Download className="h-3 w-3" />
            Install the Onus CLI
          </div>
          <h1 className="mb-4 text-4xl font-bold text-white">Install Onus like a normal developer tool.</h1>
          <p className="text-lg leading-8 text-zinc-400">
            Install the `onus` binary, run `onus doctor`, start the daemon, then open the local admin console.
            Deterministic mode works offline. LLM providers are optional and must be configured in the daemon environment.
          </p>
        </div>

        <div className="grid gap-6 lg:grid-cols-2">
          <section className="rounded-lg border border-zinc-800 bg-zinc-900/45 p-6">
            <h2 className="mb-3 text-xl font-semibold text-white">Windows PowerShell</h2>
            <CodeBlock label="PowerShell" code={windowsInstall} />
          </section>

          <section className="rounded-lg border border-zinc-800 bg-zinc-900/45 p-6">
            <h2 className="mb-3 text-xl font-semibold text-white">Linux Bash</h2>
            <CodeBlock label="Bash" code={linuxInstall} />
          </section>
        </div>

        <section className="mt-6 rounded-lg border border-zinc-800 bg-zinc-900/45 p-6">
          <h2 className="mb-3 text-xl font-semibold text-white">Build From Source</h2>
          <p className="mb-4 text-sm leading-6 text-zinc-400">
            Use this path when release artifacts are unavailable or when you want to inspect and build the code yourself.
          </p>
          <CodeBlock label="Source build" code={sourceBuild} />
        </section>

        <section className="mt-8 grid gap-4 md:grid-cols-3">
          {[
            ['1', 'Verify', '`onus doctor` checks the binary, rules, configured integrations, and local audit state.'],
            ['2', 'Start', '`onus start` launches the daemon. Use `onus stop` or `onus restart` to manage it.'],
            ['3', 'Open Console', '`onus console --port 3001` serves the local admin panel with token auth.'],
          ].map(([step, title, body]) => (
            <div key={step} className="rounded-lg border border-zinc-800 bg-zinc-900/45 p-5">
              <div className="mb-3 flex h-8 w-8 items-center justify-center rounded-full bg-accent text-sm font-bold text-black">{step}</div>
              <h3 className="mb-2 font-semibold text-white">{title}</h3>
              <p className="text-sm leading-6 text-zinc-500">{body}</p>
            </div>
          ))}
        </section>

        <section className="mt-8 rounded-lg border border-zinc-800 bg-zinc-950 p-6">
          <h2 className="mb-4 flex items-center gap-2 text-xl font-semibold text-white">
            <Shield className="h-5 w-5 text-accent" />
            Production-use checklist
          </h2>
          <div className="grid gap-3 md:grid-cols-2">
            {[
              'Use deterministic-only mode first and confirm policies with `onus rules`.',
              'Run `onus doctor` and resolve failures before connecting agents.',
              'Use `onus setup --claude`, `--codex`, or `--cursor` only for integrations you will actually route through Onus.',
              'Open the local admin console with an unpredictable token: `onus console --token <random>`.',
              'Use Linux + bubblewrap for any L3 workspace claim. Windows is not L3-contained.',
              'Do not put production credentials into agent prompts, audit payloads, or website forms.',
            ].map((item) => (
              <div key={item} className="flex gap-3 text-sm leading-6 text-zinc-400">
                <CheckCircle className="mt-1 h-4 w-4 flex-shrink-0 text-accent" />
                <span>{item}</span>
              </div>
            ))}
          </div>
        </section>

        <div className="mt-8 flex flex-col gap-3 sm:flex-row">
          <Link href="/docs/quick-start" className="inline-flex items-center justify-center gap-2 rounded-full bg-accent px-6 py-3 text-sm font-semibold text-black hover:bg-accent-hover">
            Continue to Quick Start
            <ArrowRight className="h-4 w-4" />
          </Link>
          <Link href="/docs/cli-reference" className="inline-flex items-center justify-center gap-2 rounded-full border border-zinc-700 px-6 py-3 text-sm text-zinc-200 hover:bg-zinc-900">
            <Terminal className="h-4 w-4" />
            CLI Reference
          </Link>
        </div>
      </main>
    </div>
  );
}
