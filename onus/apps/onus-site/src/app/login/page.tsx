'use client';

import { useMemo, useState } from 'react';
import Link from 'next/link';
import { ArrowLeft, ExternalLink, KeyRound, Lock, Terminal } from 'lucide-react';
import { Entropy } from '@/components/ui/entropy';
import { FallingPattern } from '@/components/ui/falling-pattern';

export default function LoginPage() {
  const [consoleUrl, setConsoleUrl] = useState('http://127.0.0.1:3001');
  const [sessionToken, setSessionToken] = useState('');

  const normalizedUrl = useMemo(() => {
    try {
      const parsed = new URL(consoleUrl);
      return parsed.toString().replace(/\/$/, '');
    } catch {
      return '';
    }
  }, [consoleUrl]);

  const openConsole = () => {
    if (!normalizedUrl) return;
    window.location.assign(normalizedUrl);
  };

  return (
    <main className="relative min-h-screen overflow-hidden bg-black px-4 py-8 text-zinc-100">
      <FallingPattern className="absolute inset-0 opacity-35" color="rgba(255,255,255,0.16)" />
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_18%_18%,rgba(249,115,22,0.18),transparent_30%),linear-gradient(180deg,rgba(0,0,0,0.15),#000_88%)]" />

      <div className="relative mx-auto flex min-h-[calc(100vh-4rem)] max-w-6xl items-center">
        <div className="grid w-full gap-10 lg:grid-cols-[0.95fr_1.05fr] lg:items-center">
          <section className="hidden lg:block">
            <div className="relative overflow-hidden rounded-lg border border-white/10 bg-zinc-950/80 p-8 shadow-2xl shadow-black/50">
              <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_10%,rgba(249,115,22,0.14),transparent_44%)]" />
              <div className="relative flex min-h-[520px] items-center justify-center">
                <Entropy size={390} />
                <div className="absolute bottom-6 left-6 right-6 rounded-lg border border-white/10 bg-black/75 p-4">
                  <div className="mb-3 flex items-center gap-2 text-sm font-medium text-white">
                    <Lock className="h-4 w-4 text-accent" />
                    Local control plane
                  </div>
                  <p className="text-sm leading-6 text-zinc-400">
                    This page does not authenticate against a cloud service. It opens your local Onus console,
                    where access depends on the daemon and its configured session token.
                  </p>
                </div>
              </div>
            </div>
          </section>

          <section className="mx-auto w-full max-w-md">
            <Link href="/" className="mb-8 inline-flex items-center gap-2 text-sm text-zinc-500 transition-colors hover:text-zinc-200">
              <ArrowLeft className="h-4 w-4" />
              Back to Onus
            </Link>

            <div className="rounded-lg border border-white/10 bg-zinc-950/85 p-6 shadow-2xl shadow-black/50 backdrop-blur">
              <div className="mb-8">
                <div className="mb-4 inline-flex h-11 w-11 items-center justify-center rounded-full bg-accent text-black">
                  <Terminal className="h-5 w-5" />
                </div>
                <h1 className="text-3xl font-bold text-white">Open local console</h1>
                <p className="mt-3 text-sm leading-6 text-zinc-400">
                  Use this entry point for an Onus console already running on your machine.
                  No provider keys or raw secrets are sent from this website.
                </p>
              </div>

              <div className="space-y-5">
                <label className="block">
                  <span className="mb-2 block text-sm font-medium text-zinc-300">Console URL</span>
                  <input
                    value={consoleUrl}
                    onChange={(event) => setConsoleUrl(event.target.value)}
                    className="w-full rounded-md border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-accent"
                    placeholder="http://127.0.0.1:3001"
                  />
                </label>

                <label className="block">
                  <span className="mb-2 block text-sm font-medium text-zinc-300">Session token</span>
                  <div className="flex items-center rounded-md border border-zinc-800 bg-black px-4 py-3 focus-within:border-accent">
                    <KeyRound className="mr-3 h-4 w-4 text-zinc-600" />
                    <input
                      value={sessionToken}
                      onChange={(event) => setSessionToken(event.target.value)}
                      type="password"
                      className="w-full bg-transparent text-sm text-white outline-none"
                      placeholder="Paste token in the console when prompted"
                    />
                  </div>
                  <p className="mt-2 text-xs leading-5 text-zinc-600">
                    Token value is held only in this page state and is not appended to the URL.
                  </p>
                </label>

                <button
                  onClick={openConsole}
                  disabled={!normalizedUrl}
                  className="inline-flex w-full items-center justify-center gap-2 rounded-full bg-accent px-5 py-3 text-sm font-semibold text-black transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
                >
                  Open console
                  <ExternalLink className="h-4 w-4" />
                </button>

                <div className="rounded-md border border-zinc-800 bg-black/70 p-4 text-xs leading-6 text-zinc-500">
                  Start the console locally with the repository scripts or the installed `onus console` command,
                  then return here if you want a bookmarked entry point.
                </div>
              </div>
            </div>
          </section>
        </div>
      </div>
    </main>
  );
}
