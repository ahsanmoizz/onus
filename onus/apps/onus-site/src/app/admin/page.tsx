import Link from 'next/link';
import { Activity, ArrowLeft, CheckCircle, Database, KeyRound, Lock, Server, Shield, Terminal } from 'lucide-react';

const startConsole = `onus start
onus console --port 3001
# Open the URL printed by the command and keep the generated token private.`;

const commonAdmin = [
  ['Overview', 'Daemon status, guardian mode, provider mode, receipt-chain status, and active sessions.'],
  ['Actions', 'Recent governed actions, verdicts, policy reasons, and receipt details from the real audit DB.'],
  ['Approvals', 'Pending human approvals for exact action payloads.'],
  ['Rules', 'Deterministic safety rules and policy status.'],
  ['Sessions', 'Session summaries and replay-oriented audit context.'],
  ['Rollback', 'Checkpoint and rollback controls for supported repository/SQLite paths.'],
  ['Doctor', 'Readiness checks for local integrations and system configuration.'],
  ['Settings', 'Local daemon and console configuration surface.'],
];

export default function AdminPage() {
  return (
    <main className="min-h-screen bg-black px-4 py-8 text-zinc-100">
      <div className="mx-auto max-w-5xl">
        <Link href="/" className="mb-8 inline-flex items-center gap-2 text-sm text-zinc-500 hover:text-zinc-200">
          <ArrowLeft className="h-4 w-4" />
          Back to site
        </Link>

        <section className="mb-10 rounded-lg border border-zinc-800 bg-zinc-950 p-8">
          <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-accent/20 bg-accent/10 px-3 py-1 text-xs text-accent">
            <Shield className="h-3 w-3" />
            Local Admin Console
          </div>
          <h1 className="mb-4 text-4xl font-bold text-white">Onus has a separate admin panel.</h1>
          <p className="max-w-3xl text-lg leading-8 text-zinc-400">
            The official site is the public download/docs website. The admin panel is the local Onus Console,
            served from your machine by the CLI and backed by your local daemon and audit database.
          </p>
        </section>

        <div className="grid gap-6 lg:grid-cols-[0.95fr_1.05fr]">
          <section className="rounded-lg border border-zinc-800 bg-zinc-900/45 p-6">
            <h2 className="mb-4 flex items-center gap-2 text-xl font-semibold text-white">
              <Terminal className="h-5 w-5 text-accent" />
              Start it
            </h2>
            <pre className="overflow-x-auto rounded-md border border-zinc-800 bg-black p-4 text-sm leading-6 text-zinc-300"><code>{startConsole}</code></pre>
            <div className="mt-4 rounded-md border border-zinc-800 bg-black/60 p-4 text-sm leading-6 text-zinc-500">
              The console defaults to port `3001`. Its API calls are routed to the local daemon at `127.0.0.1:9090`.
              Use the generated token or pass your own with `--token`.
            </div>
          </section>

          <section className="rounded-lg border border-zinc-800 bg-zinc-900/45 p-6">
            <h2 className="mb-4 text-xl font-semibold text-white">What it manages</h2>
            <div className="grid gap-3 sm:grid-cols-2">
              {commonAdmin.map(([title, body]) => (
                <div key={title} className="rounded-md border border-zinc-800 bg-black/55 p-4">
                  <h3 className="mb-1 text-sm font-semibold text-white">{title}</h3>
                  <p className="text-xs leading-5 text-zinc-500">{body}</p>
                </div>
              ))}
            </div>
          </section>
        </div>

        <section className="mt-6 grid gap-4 md:grid-cols-4">
          {[
            [Server, 'Daemon-backed', 'The console is not static mock data; it reads local daemon endpoints.'],
            [Database, 'Audit-backed', 'Sessions, actions, and receipts come from the local SQLite audit trail.'],
            [KeyRound, 'Token-gated', 'Use an unpredictable console token for local access.'],
            [Lock, 'Local only', 'Do not expose the console directly to the public internet.'],
          ].map(([Icon, title, body]) => {
            const IconComponent = Icon as typeof Shield;
            return (
              <div key={title as string} className="rounded-lg border border-zinc-800 bg-zinc-900/45 p-5">
                <IconComponent className="mb-3 h-5 w-5 text-accent" />
                <h3 className="mb-2 font-semibold text-white">{title as string}</h3>
                <p className="text-sm leading-6 text-zinc-500">{body as string}</p>
              </div>
            );
          })}
        </section>

        <section className="mt-6 rounded-lg border border-zinc-800 bg-zinc-950 p-6">
          <h2 className="mb-4 flex items-center gap-2 text-xl font-semibold text-white">
            <Activity className="h-5 w-5 text-accent" />
            Admin checklist
          </h2>
          <div className="grid gap-3 md:grid-cols-2">
            {[
              'Run `onus doctor` before demos or production-like use.',
              'Start the daemon with `onus start` before opening the console.',
              'Open console with `onus console --token <random>` when other local processes are untrusted.',
              'Verify receipts with `onus verify` before relying on audit evidence.',
              'Use `onus approvals serve` for dedicated approval review flows.',
              'Keep L1 integrations labeled BEST-EFFORT and L2 claims limited to routed actions.',
            ].map((item) => (
              <div key={item} className="flex gap-3 text-sm leading-6 text-zinc-400">
                <CheckCircle className="mt-1 h-4 w-4 flex-shrink-0 text-accent" />
                <span>{item}</span>
              </div>
            ))}
          </div>
        </section>

        <div className="mt-8 flex gap-3">
          <Link href="/install" className="rounded-full bg-accent px-5 py-3 text-sm font-semibold text-black hover:bg-accent-hover">
            Install Onus
          </Link>
          <Link href="/docs/cli-reference" className="rounded-full border border-zinc-700 px-5 py-3 text-sm text-zinc-200 hover:bg-zinc-900">
            CLI commands
          </Link>
        </div>
      </div>
    </main>
  );
}
