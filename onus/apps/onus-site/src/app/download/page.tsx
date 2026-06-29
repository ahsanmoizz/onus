import Link from 'next/link';
import { ArrowLeft, CheckCircle, Download, FileArchive, Github, Shield, Terminal } from 'lucide-react';

const downloads = [
  {
    title: 'Windows x86_64',
    file: 'onus-windows-x86_64.zip',
    command: 'Invoke-WebRequest -Uri "https://raw.githubusercontent.com/ahsanmoizz/onus/main/onus/install/install-onus.ps1" -OutFile "install-onus.ps1"',
    note: 'Use PowerShell. L3 workspace containment is not available on native Windows.',
  },
  {
    title: 'Linux x86_64',
    file: 'onus-linux-x86_64.tar.gz',
    command: 'curl -fsSL https://raw.githubusercontent.com/ahsanmoizz/onus/main/onus/install/install-onus.sh -o install-onus.sh',
    note: 'Use Linux for L3 workspace containment with bubblewrap.',
  },
  {
    title: 'Source',
    file: 'git clone',
    command: 'git clone https://github.com/ahsanmoizz/onus.git && cd onus/onus && cargo build --release',
    note: 'Best path while release artifacts are being finalized or when you want to inspect the code.',
  },
];

export default function DownloadPage() {
  return (
    <div className="min-h-screen bg-black text-zinc-100">
      <nav className="fixed inset-x-0 top-0 z-50 border-b border-zinc-800 bg-black/85 backdrop-blur-md">
        <div className="mx-auto flex h-16 max-w-5xl items-center justify-between px-4">
          <Link href="/" className="flex items-center gap-2">
            <div className="flex h-7 w-7 items-center justify-center rounded-full bg-accent text-xs font-bold text-black">O</div>
            <span className="font-bold text-white">Onus</span>
          </Link>
          <Link href="/" className="inline-flex items-center gap-1 text-sm text-zinc-400 hover:text-white">
            <ArrowLeft className="h-3 w-3" />
            Back
          </Link>
        </div>
      </nav>

      <main className="mx-auto max-w-5xl px-4 pb-20 pt-28">
        <div className="mb-10 text-center">
          <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-accent/20 bg-accent/10 px-3 py-1 text-xs text-accent">
            <Download className="h-3 w-3" />
            Onus v0.1.0
          </div>
          <h1 className="mb-4 text-4xl font-bold text-white">Download Onus</h1>
          <p className="mx-auto max-w-2xl text-zinc-400">
            Onus is installed as a CLI first. The official site provides downloads and docs;
            the local admin console is launched from the CLI with `onus console`.
          </p>
        </div>

        <div className="grid gap-5 md:grid-cols-3">
          {downloads.map((item) => (
            <section key={item.title} className="rounded-lg border border-zinc-800 bg-zinc-900/45 p-5">
              <FileArchive className="mb-4 h-6 w-6 text-accent" />
              <h2 className="mb-1 text-lg font-semibold text-white">{item.title}</h2>
              <p className="mb-4 font-mono text-xs text-zinc-500">{item.file}</p>
              <pre className="mb-4 overflow-x-auto rounded-md border border-zinc-800 bg-black p-3 text-xs leading-5 text-zinc-300"><code>{item.command}</code></pre>
              <p className="text-sm leading-6 text-zinc-500">{item.note}</p>
            </section>
          ))}
        </div>

        <section className="mt-8 rounded-lg border border-zinc-800 bg-zinc-950 p-6">
          <h2 className="mb-4 flex items-center gap-2 text-xl font-semibold text-white">
            <Shield className="h-5 w-5 text-accent" />
            Verify before use
          </h2>
          <div className="grid gap-4 md:grid-cols-2">
            <div>
              <p className="mb-2 text-sm text-zinc-400">Windows checksum:</p>
              <pre className="overflow-x-auto rounded-md border border-zinc-800 bg-black p-3 text-xs text-zinc-300"><code>Get-FileHash .\onus-windows-x86_64.zip -Algorithm SHA256</code></pre>
            </div>
            <div>
              <p className="mb-2 text-sm text-zinc-400">Linux checksum:</p>
              <pre className="overflow-x-auto rounded-md border border-zinc-800 bg-black p-3 text-xs text-zinc-300"><code>sha256sum onus-linux-x86_64.tar.gz</code></pre>
            </div>
          </div>
          <p className="mt-4 text-sm leading-6 text-zinc-500">
            If a signed release manifest is not available for the exact artifact you downloaded, build from source
            or treat the artifact as unverified. Do not use unverified binaries for production authority or credentials.
          </p>
        </section>

        <section className="mt-8 grid gap-4 md:grid-cols-3">
          {[
            ['Install', 'Run the platform installer or build from source.'],
            ['Check', 'Run `onus doctor` and fix failures.'],
            ['Use', 'Run `onus start`, then `onus console`, then connect a routed integration.'],
          ].map(([title, body]) => (
            <div key={title} className="rounded-lg border border-zinc-800 bg-zinc-900/45 p-5">
              <CheckCircle className="mb-3 h-5 w-5 text-accent" />
              <h3 className="mb-2 font-semibold text-white">{title}</h3>
              <p className="text-sm leading-6 text-zinc-500">{body}</p>
            </div>
          ))}
        </section>

        <div className="mt-8 flex flex-col gap-3 sm:flex-row sm:justify-center">
          <Link href="/install" className="inline-flex items-center justify-center gap-2 rounded-full bg-accent px-6 py-3 text-sm font-semibold text-black hover:bg-accent-hover">
            <Terminal className="h-4 w-4" />
            Installation guide
          </Link>
          <a href="https://github.com/ahsanmoizz/onus/releases" className="inline-flex items-center justify-center gap-2 rounded-full border border-zinc-700 px-6 py-3 text-sm text-zinc-200 hover:bg-zinc-900">
            <Github className="h-4 w-4" />
            GitHub releases
          </a>
        </div>
      </main>
    </div>
  );
}
