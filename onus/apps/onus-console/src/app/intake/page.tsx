'use client';

import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { Terminal, AlertTriangle, CheckCircle, XCircle, HelpCircle, ArrowRight, Loader2 } from 'lucide-react';

interface IntakeResult {
  outcome: 'READY' | 'READY_WITH_SAFE_CONTRACT' | 'CLARIFICATION_REQUIRED' | 'REJECTED_AS_UNSAFE';
  risks?: string[];
  questions?: string[];
  safe_contract?: Record<string, unknown>;
  reason?: string;
  contract_hash?: string;
}

export default function IntakePage() {
  const [prompt, setPrompt] = useState('');
  const [result, setResult] = useState<IntakeResult | null>(null);

  const intakeMutation = useMutation({
    mutationFn: (p: string) => api.post<IntakeResult>('/intake', { prompt: p }),
    onSuccess: (data) => setResult(data),
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (prompt.trim()) {
      setResult(null);
      intakeMutation.mutate(prompt);
    }
  };

  const outcomeConfig: Record<string, { icon: typeof Terminal; label: string; color: string }> = {
    READY: { icon: CheckCircle, label: 'Ready to proceed', color: 'text-green-400' },
    READY_WITH_SAFE_CONTRACT: { icon: CheckCircle, label: 'Ready with safe contract', color: 'text-green-400' },
    CLARIFICATION_REQUIRED: { icon: HelpCircle, label: 'Clarification required', color: 'text-yellow-400' },
    REJECTED_AS_UNSAFE: { icon: XCircle, label: 'Rejected as unsafe', color: 'text-red-400' },
  };

  return (
    <DashboardLayout>
      <div className="max-w-3xl mx-auto space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-white">New Governed Task</h1>
          <p className="text-zinc-400 text-sm mt-1">Submit a task prompt for analysis by the Prompt Intake Guardian</p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4">
            <label className="block text-sm font-medium text-zinc-300 mb-2">Task Prompt</label>
            <textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="Describe the task you want the agent to perform..."
              rows={6}
              className="w-full bg-black border border-zinc-700 rounded-lg p-3 text-zinc-100 text-sm placeholder-zinc-600 focus:outline-none focus:border-accent resize-none"
            />
          </div>
          <button
            type="submit"
            disabled={!prompt.trim() || intakeMutation.isPending}
            className="flex items-center gap-2 px-6 py-2.5 bg-accent text-black rounded-full font-medium text-sm hover:bg-accent-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {intakeMutation.isPending ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Terminal className="w-4 h-4" />
            )}
            Analyze Task
          </button>
        </form>

        {intakeMutation.isError && (
          <div className="bg-red-900/20 border border-red-800/30 rounded-lg p-4 text-red-400 text-sm">
            Failed to analyze task. Is the daemon running?
          </div>
        )}

        {result && (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-6 space-y-4">
            <div className="flex items-center gap-3">
              {(() => {
                const cfg = outcomeConfig[result.outcome];
                const Icon = cfg?.icon || AlertTriangle;
                return <Icon className={`w-6 h-6 ${cfg?.color || 'text-zinc-400'}`} />;
              })()}
              <span className="text-lg font-semibold text-white">
                {outcomeConfig[result.outcome]?.label || result.outcome}
              </span>
            </div>

            {result.reason && (
              <p className="text-zinc-400 text-sm">{result.reason}</p>
            )}

            {result.risks && result.risks.length > 0 && (
              <div>
                <h4 className="text-sm font-medium text-zinc-300 mb-2">Detected Risks</h4>
                <ul className="space-y-1">
                  {result.risks.map((risk, i) => (
                    <li key={i} className="flex items-start gap-2 text-sm text-zinc-400">
                      <AlertTriangle className="w-4 h-4 text-yellow-500 mt-0.5 flex-shrink-0" />
                      {risk}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {result.questions && result.questions.length > 0 && (
              <div>
                <h4 className="text-sm font-medium text-zinc-300 mb-2">Clarification Needed</h4>
                <ul className="space-y-1">
                  {result.questions.map((q, i) => (
                    <li key={i} className="flex items-start gap-2 text-sm text-zinc-400">
                      <HelpCircle className="w-4 h-4 text-accent mt-0.5 flex-shrink-0" />
                      {q}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {result.safe_contract && (
              <div>
                <h4 className="text-sm font-medium text-zinc-300 mb-2">Safe Contract</h4>
                <pre className="bg-black rounded-lg p-3 text-xs text-zinc-400 overflow-x-auto">
                  {JSON.stringify(result.safe_contract, null, 2)}
                </pre>
              </div>
            )}

            {result.contract_hash && (
              <p className="text-xs text-zinc-600 font-mono">Contract Hash: {result.contract_hash}</p>
            )}

            <div className="flex gap-3 pt-2">
              <button className="flex items-center gap-2 px-4 py-2 bg-accent text-black rounded-full text-sm font-medium hover:bg-accent-hover transition-colors">
                Accept Contract <ArrowRight className="w-4 h-4" />
              </button>
              <button className="px-4 py-2 border border-zinc-700 text-zinc-300 rounded-full text-sm hover:bg-zinc-800 transition-colors">
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
