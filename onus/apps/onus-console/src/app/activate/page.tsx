'use client';

import { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import {
  CheckCircle2, Circle, ChevronRight, ChevronLeft,
  Terminal, Shield, Cpu, Server, Plug, ListChecks, Sparkles,
  Loader2, AlertCircle, ArrowRight, Zap, BookOpen, Settings, FileText
} from 'lucide-react';
import { cn } from '@/lib/utils';

const STEPS = [
  'Welcome', 'Build', 'Guardian', 'Provider', 'Daemon', 'Integrations', 'Validation', 'Complete'
];

type GuardianMode = 'deterministic' | 'local' | 'cloud' | 'disabled' | null;
type StepStatus = 'pending' | 'current' | 'complete';

interface CheckResult {
  label: string;
  status: 'idle' | 'running' | 'pass' | 'fail';
  message?: string;
}

export default function ActivatePage() {
  const [step, setStep] = useState(0);
  const [daemonRunning, setDaemonRunning] = useState<boolean | null>(null);
  const [daemonChecking, setDaemonChecking] = useState(false);
  const [guardianMode, setGuardianMode] = useState<GuardianMode>(null);
  const [providerKey, setProviderKey] = useState('');
  const [providerEndpoint, setProviderEndpoint] = useState('');
  const [providerTested, setProviderTested] = useState(false);
  const [providerTesting, setProviderTesting] = useState(false);
  const [providerStatus, setProviderStatus] = useState<{ ok: boolean; message: string } | null>(null);
  const [integrations, setIntegrations] = useState<string[]>([]);
  const [checkResults, setCheckResults] = useState<CheckResult[]>([]);
  const [runValidating, setRunValidating] = useState(false);

  const checkDaemon = useCallback(async () => {
    setDaemonChecking(true);
    try {
      const res = await fetch('/api/status', { signal: AbortSignal.timeout(3000) });
      if (res.ok) {
        setDaemonRunning(true);
        return true;
      }
    } catch { /* daemon not available */ }
    finally {
      setDaemonChecking(false);
    }
    setDaemonRunning(false);
    return false;
  }, []);

  useEffect(() => { checkDaemon(); }, [checkDaemon]);

  // Auto-advance past build step if daemon already running
  useEffect(() => {
    if (step === 1 && daemonRunning === true) {
      setStep(2);
    }
  }, [daemonRunning, step]);

  const canAdvance = (): boolean => {
    switch (step) {
      case 0: return daemonRunning !== null;
      case 1: return true;
      case 2: return guardianMode !== null;
      case 3:
        if (guardianMode === 'deterministic' || guardianMode === 'disabled') return true;
        if (guardianMode === 'local') return providerEndpoint.length > 0;
        return providerKey.length > 0;
      case 4: return daemonRunning === true;
      case 5: return true;
      case 6: return checkResults.length > 0 && checkResults.every(r => r.status === 'pass') && !runValidating;
      case 7: return true;
      default: return false;
    }
  };

  const isLastStep = step === STEPS.length - 1;
  const isFirstStep = step === 0;

  const next = () => {
    if (!canAdvance()) return;
    if (step < STEPS.length - 1) setStep(s => s + 1);
  };

  const prev = () => {
    if (step > 0) setStep(s => s - 1);
  };

  const testProvider = async () => {
    setProviderTesting(true);
    setProviderTested(false);
    setProviderStatus(null);
    try {
      if (guardianMode === 'deterministic' || guardianMode === 'disabled') {
        setProviderTested(true);
        setProviderStatus({ ok: true, message: 'No external semantic provider is required for this mode.' });
        return;
      }

      setProviderStatus({
        ok: false,
        message: 'Browser provider testing is intentionally disabled. Configure provider credentials in the daemon environment and verify them with `onus doctor`.',
      });
    } finally {
      setProviderTesting(false);
    }
  };

  const runValidation = async () => {
    setRunValidating(true);
    const checks: CheckResult[] = [
      { label: 'Daemon Health Check', status: 'idle' },
      { label: 'Guardian Mode Active', status: 'idle' },
      { label: 'Provider Configuration', status: 'idle' },
      { label: 'Daemon Doctor Diagnostics', status: 'idle' },
      { label: 'Receipt Chain Verification', status: 'idle' },
    ];
    setCheckResults(checks);

    const updateCheck = (index: number, patch: Partial<CheckResult>) => {
      setCheckResults(prev => prev.map((c, j) => j === index ? { ...c, ...patch } : c));
    };

    try {
      updateCheck(0, { status: 'running' });
      let daemonOk = false;
      try {
        const res = await fetch('/api/status', { signal: AbortSignal.timeout(3000) });
        daemonOk = res.ok;
      } catch {
        daemonOk = false;
      }
      setDaemonRunning(daemonOk);
      updateCheck(0, {
        status: daemonOk ? 'pass' : 'fail',
        message: daemonOk ? 'Daemon responded to /api/status' : 'Daemon not reachable at /api/status',
      });

      updateCheck(1, { status: 'running' });
      const modeOk = guardianMode !== null;
      updateCheck(1, {
        status: modeOk ? 'pass' : 'fail',
        message: modeOk ? `Selected mode: ${guardianMode}` : 'No guardian mode selected',
      });

      updateCheck(2, { status: 'running' });
      const providerOk =
        guardianMode === 'deterministic' ||
        guardianMode === 'disabled' ||
        providerTested;
      updateCheck(2, {
        status: providerOk ? 'pass' : 'fail',
        message: providerOk
          ? (guardianMode === 'deterministic' || guardianMode === 'disabled'
              ? 'No external provider required'
              : 'Provider was verified outside the browser console')
          : 'Provider was not verified. Run `onus doctor` with daemon-side credentials.',
      });

      updateCheck(3, { status: 'running' });
      let doctorOk = false;
      try {
        const res = await fetch('/api/doctor', { signal: AbortSignal.timeout(5000) });
        doctorOk = res.ok;
      } catch {
        doctorOk = false;
      }
      updateCheck(3, {
        status: doctorOk ? 'pass' : 'fail',
        message: doctorOk ? 'Daemon doctor endpoint returned success' : 'Daemon doctor endpoint failed or is unavailable',
      });

      updateCheck(4, { status: 'running' });
      let verifyOk = false;
      try {
        const res = await fetch('/api/verify', { signal: AbortSignal.timeout(5000) });
        verifyOk = res.ok;
      } catch {
        verifyOk = false;
      }
      updateCheck(4, {
        status: verifyOk ? 'pass' : 'fail',
        message: verifyOk ? 'Receipt verifier returned success' : 'Receipt verifier failed or no ledger was available',
      });
    } finally {
      setRunValidating(false);
    }
  };

  const toggleIntegration = (name: string) => {
    setIntegrations(prev =>
      prev.includes(name) ? prev.filter(n => n !== name) : [...prev, name]
    );
  };

  return (
    <div className="min-h-screen bg-black text-zinc-100">
      {/* Progress bar */}
      <div className="fixed top-0 left-0 right-0 z-50 bg-black/90 border-b border-zinc-800">
        <div className="max-w-3xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            {STEPS.map((label, i) => (
              <div key={i} className="flex items-center">
                <div className="flex flex-col items-center">
                  <div className={cn(
                    'w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium transition-colors',
                    i < step ? 'bg-green-600 text-white' :
                    i === step ? 'bg-accent text-black' :
                    'bg-zinc-800 text-zinc-500'
                  )}>
                    {i < step ? <CheckCircle2 className="w-4 h-4" /> : i + 1}
                  </div>
                  <span className={cn(
                    'text-[10px] mt-1 hidden sm:block',
                    i === step ? 'text-accent' : i < step ? 'text-green-500' : 'text-zinc-600'
                  )}>
                    {label}
                  </span>
                </div>
                {i < STEPS.length - 1 && (
                  <div className={cn(
                    'h-px w-6 sm:w-12 mx-1',
                    i < step ? 'bg-green-600' : 'bg-zinc-800'
                  )} />
                )}
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="pt-24 pb-16 px-4 max-w-2xl mx-auto">
        {/* Step 0: Welcome */}
        {step === 0 && (
          <div className="text-center space-y-8">
            <div className="w-16 h-16 rounded-full bg-accent flex items-center justify-center mx-auto">
              <Sparkles className="w-8 h-8 text-black" />
            </div>
            <div>
              <h1 className="text-4xl font-bold text-white mb-3">Welcome to Onus</h1>
              <p className="text-zinc-400 text-lg">
                Your AI Agent Firewall. Let&apos;s get you set up in a few steps.
              </p>
            </div>

            <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 text-left space-y-4">
              <h3 className="font-semibold text-white flex items-center gap-2">
                <Server className="w-4 h-4 text-accent" />
                System Detection
              </h3>
              <div className="text-sm text-zinc-400 space-y-2">
                <p>Platform: {typeof navigator !== 'undefined' ? navigator.platform : 'Unknown'}</p>
                <p>User Agent: {typeof navigator !== 'undefined' ? navigator.userAgent : 'Unknown'}</p>
              </div>
              <div className="flex items-center gap-2 text-sm">
                {daemonChecking ? (
                  <><Loader2 className="w-4 h-4 animate-spin text-zinc-500" /><span className="text-zinc-500">Checking daemon...</span></>
                ) : daemonRunning === true ? (
                  <><CheckCircle2 className="w-4 h-4 text-green-500" /><span className="text-green-500">Daemon is running</span></>
                ) : daemonRunning === false ? (
                  <><AlertCircle className="w-4 h-4 text-yellow-500" /><span className="text-yellow-500">Daemon not detected - will need to start it</span></>
                ) : null}
              </div>
            </div>

            <button
              onClick={next}
              disabled={!canAdvance()}
              className="px-8 py-3 bg-accent text-black font-semibold rounded-full hover:bg-orange-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed inline-flex items-center gap-2"
            >
              Get Started <ArrowRight className="w-4 h-4" />
            </button>
          </div>
        )}

        {/* Step 1: Build */}
        {step === 1 && (
          <div className="space-y-8">
            <div>
              <h2 className="text-2xl font-bold text-white mb-2">Build Onus</h2>
              <p className="text-zinc-400">Build the Onus binary from source.</p>
            </div>

            <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 space-y-4">
              <div className="flex items-center gap-2 text-yellow-500">
                <Terminal className="w-5 h-5" />
                <span className="font-medium">Daemon not running</span>
              </div>
              <p className="text-sm text-zinc-400">
                Run this command in your terminal to build and start Onus:
              </p>
              <pre className="bg-black border border-zinc-700 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto">
                {`cd /path/to/onus
cargo build --release
./target/release/onus daemon &`}
              </pre>
              <button
                onClick={checkDaemon}
                className="px-4 py-2 border border-zinc-700 text-zinc-300 rounded-lg font-medium hover:bg-zinc-800 transition-colors inline-flex items-center gap-2 text-sm"
              >
                {daemonChecking ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
                Check Again
              </button>
              {daemonRunning === true && (
                <div className="flex items-center gap-2 text-green-500 text-sm">
                  <CheckCircle2 className="w-4 h-4" /> Daemon detected!
                </div>
              )}
            </div>

            <div className="flex justify-between">
              <button onClick={prev} className="px-4 py-2 text-zinc-400 hover:text-white transition-colors inline-flex items-center gap-1 text-sm">
                <ChevronLeft className="w-4 h-4" /> Back
              </button>
              <button onClick={next} disabled={!canAdvance()} className="px-6 py-2 bg-accent text-black font-semibold rounded-full hover:bg-orange-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm inline-flex items-center gap-1">
                Continue <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Step 2: Guardian Mode */}
        {step === 2 && (
          <div className="space-y-8">
            <div>
              <h2 className="text-2xl font-bold text-white mb-2">Choose Guardian Mode</h2>
              <p className="text-zinc-400">Select how Onus evaluates actions.</p>
            </div>

            <div className="space-y-3">
              {([
                { value: 'deterministic', label: 'Deterministic-only', desc: 'No API key needed. Rules-only mode. Best for restricted environments.', icon: Shield },
                { value: 'local', label: 'Local (Ollama/llama.cpp)', desc: 'Free, private, runs on your hardware. Best for development.', icon: Cpu },
                { value: 'cloud', label: 'Cloud (OpenAI/Anthropic)', desc: 'Most capable semantic analysis. Best for production use.', icon: Zap },
                { value: 'disabled', label: 'Disabled', desc: 'No enforcement. For testing only.', icon: AlertCircle },
              ] as const).map((mode) => {
                const Icon = mode.icon;
                const selected = guardianMode === mode.value;
                return (
                  <button
                    key={mode.value}
                    onClick={() => {
                      setGuardianMode(mode.value as GuardianMode);
                      setProviderTested(false);
                      setProviderStatus(null);
                    }}
                    className={cn(
                      'w-full p-4 rounded-xl border text-left transition-all',
                      selected
                        ? 'bg-accent/10 border-accent'
                        : 'bg-zinc-900/50 border-zinc-800 hover:border-zinc-600'
                    )}
                  >
                    <div className="flex items-start gap-3">
                      <div className={cn('p-2 rounded-lg', selected ? 'bg-accent/20' : 'bg-zinc-800')}>
                        <Icon className={cn('w-5 h-5', selected ? 'text-accent' : 'text-zinc-400')} />
                      </div>
                      <div>
                        <div className="font-medium text-white mb-1">{mode.label}</div>
                        <div className="text-sm text-zinc-400">{mode.desc}</div>
                      </div>
                      {selected && <CheckCircle2 className="w-5 h-5 text-accent ml-auto flex-shrink-0" />}
                    </div>
                  </button>
                );
              })}
            </div>

            <div className="flex justify-between">
              <button onClick={prev} className="px-4 py-2 text-zinc-400 hover:text-white transition-colors inline-flex items-center gap-1 text-sm">
                <ChevronLeft className="w-4 h-4" /> Back
              </button>
              <button onClick={next} disabled={!canAdvance()} className="px-6 py-2 bg-accent text-black font-semibold rounded-full hover:bg-orange-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm inline-flex items-center gap-1">
                Continue <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Step 3: Provider */}
        {step === 3 && (
          <div className="space-y-8">
            <div>
              <h2 className="text-2xl font-bold text-white mb-2">Configure Provider</h2>
              <p className="text-zinc-400">
                {guardianMode === 'local' ? 'Enter your local endpoint URL.' :
                 guardianMode === 'cloud' ? 'Enter your API key.' :
                 'No provider configuration needed for this mode.'}
              </p>
            </div>

            {guardianMode === 'cloud' && (
              <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 space-y-4">
                <label className="text-sm font-medium text-zinc-300">API Key</label>
                <input
                  type="password"
                  value={providerKey}
                  onChange={e => {
                    setProviderKey(e.target.value);
                    setProviderTested(false);
                    setProviderStatus(null);
                  }}
                  placeholder="sk-..."
                  className="w-full px-4 py-3 bg-black border border-zinc-700 rounded-lg text-white placeholder-zinc-600 focus:outline-none focus:ring-2 focus:ring-accent/50"
                />
                <button
                  onClick={testProvider}
                  disabled={!providerKey || providerTesting}
                  className="px-4 py-2 bg-accent text-black font-semibold rounded-lg hover:bg-orange-500 transition-colors disabled:opacity-50 text-sm inline-flex items-center gap-2"
                >
                  {providerTesting ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
                  Test Connection
                </button>
                {providerStatus && (
                  <div className={cn('flex items-center gap-2 text-sm', providerStatus.ok ? 'text-green-500' : 'text-yellow-500')}>
                    {providerStatus.ok ? <CheckCircle2 className="w-4 h-4" /> : <AlertCircle className="w-4 h-4" />}
                    {providerStatus.message}
                  </div>
                )}
              </div>
            )}

            {guardianMode === 'local' && (
              <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 space-y-4">
                <label className="text-sm font-medium text-zinc-300">Endpoint URL</label>
                <input
                  type="text"
                  value={providerEndpoint}
                  onChange={e => {
                    setProviderEndpoint(e.target.value);
                    setProviderTested(false);
                    setProviderStatus(null);
                  }}
                  placeholder="http://localhost:11434"
                  className="w-full px-4 py-3 bg-black border border-zinc-700 rounded-lg text-white placeholder-zinc-600 focus:outline-none focus:ring-2 focus:ring-accent/50"
                />
                <button
                  onClick={testProvider}
                  disabled={!providerEndpoint || providerTesting}
                  className="px-4 py-2 bg-accent text-black font-semibold rounded-lg hover:bg-orange-500 transition-colors disabled:opacity-50 text-sm inline-flex items-center gap-2"
                >
                  {providerTesting ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
                  Test Connection
                </button>
                {providerStatus && (
                  <div className={cn('flex items-center gap-2 text-sm', providerStatus.ok ? 'text-green-500' : 'text-yellow-500')}>
                    {providerStatus.ok ? <CheckCircle2 className="w-4 h-4" /> : <AlertCircle className="w-4 h-4" />}
                    {providerStatus.message}
                  </div>
                )}
              </div>
            )}

            {(guardianMode === 'deterministic' || guardianMode === 'disabled') && (
              <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
                <div className="flex items-center gap-2 text-zinc-400 text-sm">
                  <CheckCircle2 className="w-4 h-4 text-green-500" />
                  No provider needed - this step will be skipped.
                </div>
              </div>
            )}

            <div className="flex justify-between">
              <button onClick={prev} className="px-4 py-2 text-zinc-400 hover:text-white transition-colors inline-flex items-center gap-1 text-sm">
                <ChevronLeft className="w-4 h-4" /> Back
              </button>
              <button onClick={next} disabled={!canAdvance()} className="px-6 py-2 bg-accent text-black font-semibold rounded-full hover:bg-orange-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm inline-flex items-center gap-1">
                Continue <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Step 4: Daemon */}
        {step === 4 && (
          <div className="space-y-8">
            <div>
              <h2 className="text-2xl font-bold text-white mb-2">Start the Daemon</h2>
              <p className="text-zinc-400">The daemon must be running for Onus to govern actions.</p>
            </div>

            <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 space-y-4">
              {daemonRunning === true ? (
                <div className="flex items-center gap-3 text-green-500">
                  <div className="w-3 h-3 rounded-full bg-green-500 animate-pulse" />
                  <span className="font-medium">Daemon is running</span>
                </div>
              ) : (
                <>
                  <div className="flex items-center gap-2 text-yellow-500">
                    <AlertCircle className="w-5 h-5" />
                    <span className="font-medium">Daemon is not running</span>
                  </div>
                  <p className="text-sm text-zinc-400">Run this command:</p>
                  <pre className="bg-black border border-zinc-700 rounded-lg p-4 text-sm text-zinc-300 font-mono">onus daemon &</pre>
                  <button
                    onClick={checkDaemon}
                    className="px-4 py-2 border border-zinc-700 text-zinc-300 rounded-lg font-medium hover:bg-zinc-800 transition-colors text-sm inline-flex items-center gap-2"
                  >
                    {daemonChecking ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
                    Check Status
                  </button>
                </>
              )}
            </div>

            <div className="flex justify-between">
              <button onClick={prev} className="px-4 py-2 text-zinc-400 hover:text-white transition-colors inline-flex items-center gap-1 text-sm">
                <ChevronLeft className="w-4 h-4" /> Back
              </button>
              <button onClick={next} disabled={!canAdvance()} className="px-6 py-2 bg-accent text-black font-semibold rounded-full hover:bg-orange-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm inline-flex items-center gap-1">
                Continue <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Step 5: Integrations */}
        {step === 5 && (
          <div className="space-y-8">
            <div>
              <h2 className="text-2xl font-bold text-white mb-2">Configure Integrations</h2>
              <p className="text-zinc-400">Choose which integrations to enable. You can change these later.</p>
            </div>

            <div className="space-y-3">
              {[
                { name: 'Claude Code CLI', desc: 'MCP hook for Claude Code sessions' },
                { name: 'OpenAI Codex CLI', desc: 'Codex CLI integration via Onus proxy' },
                { name: 'Google Antigravity', desc: 'Antigravity IDE agent integration' },
                { name: 'VS Code Extension', desc: 'Onus extension for VS Code' },
                { name: 'Cursor IDE', desc: 'Cursor MCP hook integration' },
              ].map((int) => (
                <button
                  key={int.name}
                  onClick={() => toggleIntegration(int.name)}
                  className={cn(
                    'w-full p-4 rounded-xl border text-left transition-all',
                    integrations.includes(int.name)
                      ? 'bg-accent/10 border-accent'
                      : 'bg-zinc-900/50 border-zinc-800 hover:border-zinc-600'
                  )}
                >
                  <div className="flex items-center gap-3">
                    <div className={cn(
                      'w-5 h-5 rounded border flex items-center justify-center',
                      integrations.includes(int.name) ? 'bg-accent border-accent' : 'border-zinc-600'
                    )}>
                      {integrations.includes(int.name) && <CheckCircle2 className="w-4 h-4 text-black" />}
                    </div>
                    <div>
                      <div className="font-medium text-white text-sm">{int.name}</div>
                      <div className="text-xs text-zinc-500">{int.desc}</div>
                    </div>
                  </div>
                </button>
              ))}
            </div>

            <p className="text-xs text-zinc-600">
              Integrations can be configured later from the Integrations page in the console.
            </p>

            <div className="flex justify-between">
              <button onClick={prev} className="px-4 py-2 text-zinc-400 hover:text-white transition-colors inline-flex items-center gap-1 text-sm">
                <ChevronLeft className="w-4 h-4" /> Back
              </button>
              <button onClick={next} className="px-6 py-2 bg-accent text-black font-semibold rounded-full hover:bg-orange-500 transition-colors text-sm inline-flex items-center gap-1">
                Continue <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Step 6: Validation */}
        {step === 6 && (
          <div className="space-y-8">
            <div>
              <h2 className="text-2xl font-bold text-white mb-2">Validation</h2>
              <p className="text-zinc-400">Running system checks to verify everything is ready.</p>
            </div>

            <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 space-y-4">
              {checkResults.length === 0 ? (
                <div className="text-center py-6">
                  <button
                    onClick={runValidation}
                    disabled={runValidating}
                    className="px-6 py-3 bg-accent text-black font-semibold rounded-full hover:bg-orange-500 transition-colors disabled:opacity-50 inline-flex items-center gap-2"
                  >
                    {runValidating ? <Loader2 className="w-4 h-4 animate-spin" /> : <PlayIcon />}
                    Run Checks
                  </button>
                </div>
              ) : (
                <div className="space-y-3">
                  {checkResults.map((check, i) => (
                    <div key={i} className="flex items-center gap-3 p-3 rounded-lg bg-black/50">
                      {check.status === 'running' ? (
                        <Loader2 className="w-5 h-5 animate-spin text-accent" />
                      ) : check.status === 'pass' ? (
                        <CheckCircle2 className="w-5 h-5 text-green-500" />
                      ) : check.status === 'fail' ? (
                        <AlertCircle className="w-5 h-5 text-red-500" />
                      ) : (
                        <Circle className="w-5 h-5 text-zinc-600" />
                      )}
                      <div className="flex-1">
                        <div className={cn(
                          'text-sm font-medium',
                          check.status === 'pass' ? 'text-green-400' :
                          check.status === 'fail' ? 'text-red-400' :
                          'text-zinc-400'
                        )}>
                          {check.label}
                        </div>
                        {check.message && (
                          <div className="text-xs text-zinc-500 mt-0.5">{check.message}</div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="flex justify-between">
              <button onClick={prev} className="px-4 py-2 text-zinc-400 hover:text-white transition-colors inline-flex items-center gap-1 text-sm">
                <ChevronLeft className="w-4 h-4" /> Back
              </button>
              <button
                onClick={next}
                disabled={!canAdvance()}
                className="px-6 py-2 bg-accent text-black font-semibold rounded-full hover:bg-orange-500 transition-colors disabled:opacity-50 text-sm inline-flex items-center gap-1"
              >
                Continue <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Step 7: Complete */}
        {step === 7 && (
          <div className="text-center space-y-8">
            <div className="w-20 h-20 rounded-full bg-green-600 flex items-center justify-center mx-auto">
              <CheckCircle2 className="w-10 h-10 text-white" />
            </div>

            <div>
              <h1 className="text-3xl font-bold text-white mb-3">Setup Complete!</h1>
              <p className="text-zinc-400">Your Onus environment is ready to govern AI agents.</p>
            </div>

            <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 text-left space-y-3">
              <h3 className="font-semibold text-white text-sm">Configuration Summary</h3>
              <div className="text-sm text-zinc-400 space-y-2">
                <div className="flex justify-between"><span>Guardian Mode</span><span className="text-white capitalize">{guardianMode}</span></div>
                <div className="flex justify-between"><span>Daemon Status</span><span className="text-green-500">{daemonRunning ? 'Running' : 'Not started'}</span></div>
                <div className="flex justify-between"><span>Integrations</span><span className="text-white">{integrations.length > 0 ? integrations.join(', ') : 'None selected'}</span></div>
                <div className="flex justify-between"><span>Validation</span><span className="text-green-500">{checkResults.every(r => r.status === 'pass') ? 'Passed' : 'With warnings'}</span></div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <Link href="/intake" className="p-4 bg-accent/10 border border-accent rounded-xl text-left hover:bg-accent/20 transition-colors">
                <Terminal className="w-5 h-5 text-accent mb-2" />
                <div className="font-medium text-white text-sm">Create a Task</div>
                <div className="text-xs text-zinc-500">Start your first governed action</div>
              </Link>
              <Link href="/" className="p-4 bg-zinc-900/50 border border-zinc-800 rounded-xl text-left hover:bg-zinc-800/50 transition-colors">
                <LayoutDashboardIcon className="w-5 h-5 text-accent mb-2" />
                <div className="font-medium text-white text-sm">View Dashboard</div>
                <div className="text-xs text-zinc-500">Monitor system activity</div>
              </Link>
              <Link href="/rules" className="p-4 bg-zinc-900/50 border border-zinc-800 rounded-xl text-left hover:bg-zinc-800/50 transition-colors">
                <FileText className="w-5 h-5 text-accent mb-2" />
                <div className="font-medium text-white text-sm">Explore Rules</div>
                <div className="text-xs text-zinc-500">View and manage policies</div>
              </Link>
              <Link href="/docs" className="p-4 bg-zinc-900/50 border border-zinc-800 rounded-xl text-left hover:bg-zinc-800/50 transition-colors">
                <BookOpen className="w-5 h-5 text-accent mb-2" />
                <div className="font-medium text-white text-sm">Documentation</div>
                <div className="text-xs text-zinc-500">Read the full docs</div>
              </Link>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function PlayIcon() {
  return (
    <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polygon points="5 3 19 12 5 21 5 3" />
    </svg>
  );
}

function LayoutDashboardIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="3" width="7" height="9" />
      <rect x="14" y="3" width="7" height="5" />
      <rect x="14" y="12" width="7" height="9" />
      <rect x="3" y="16" width="7" height="5" />
    </svg>
  );
}
