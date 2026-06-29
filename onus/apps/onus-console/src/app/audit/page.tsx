'use client';

import { useQuery } from '@tanstack/react-query';
import { getRecentActions, verifyChain } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { FileText, Shield, CheckCircle, XCircle, AlertTriangle, RefreshCw } from 'lucide-react';
import { useState } from 'react';

const verdictIcons: Record<string, typeof Shield> = {
  allow: CheckCircle, warn: AlertTriangle, block: XCircle, escalate: Shield,
};
const verdictColors: Record<string, string> = {
  allow: 'text-green-400', warn: 'text-yellow-400', block: 'text-red-400', escalate: 'text-orange-400',
};

export default function AuditPage() {
  const [sessionFilter, setSessionFilter] = useState('');
  const { data: actions, isLoading } = useQuery({
    queryKey: ['actions'],
    queryFn: () => getRecentActions(100),
    refetchInterval: 5000,
  });
  const { data: verifyData, refetch: reVerify } = useQuery({
    queryKey: ['verify'],
    queryFn: () => verifyChain(sessionFilter || undefined),
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-white">Audit & Receipts</h1>
            <p className="text-zinc-400 text-sm mt-1">Tamper-evident action log with hash-chain verification</p>
          </div>
          <button onClick={() => reVerify()} className="flex items-center gap-1 text-xs px-3 py-1.5 bg-zinc-800 text-zinc-300 rounded-lg hover:bg-zinc-700 transition-colors">
            <RefreshCw className="w-3 h-3" /> Verify Chain
          </button>
        </div>

        {verifyData && (
          <div className={`rounded-lg border p-4 ${verifyData.status === 'ok' ? 'bg-green-900/20 border-green-800/30 text-green-400' : 'bg-red-900/20 border-red-800/30 text-red-400'}`}>
            <div className="flex items-center gap-2">
              {verifyData.status === 'ok' ? <CheckCircle className="w-5 h-5" /> : <XCircle className="w-5 h-5" />}
              <span className="font-medium">{verifyData.message}</span>
            </div>
            {verifyData.broken_links > 0 && (
              <p className="text-sm mt-1">{verifyData.broken_links} broken link(s) detected</p>
            )}
          </div>
        )}

        {isLoading ? (
          <p className="text-zinc-500">Loading audit trail...</p>
        ) : !actions || actions.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <FileText className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No actions recorded yet</p>
          </div>
        ) : (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-zinc-800">
                    <th className="text-left p-3 text-zinc-400 font-medium">Time</th>
                    <th className="text-left p-3 text-zinc-400 font-medium">Session</th>
                    <th className="text-left p-3 text-zinc-400 font-medium">Action</th>
                    <th className="text-left p-3 text-zinc-400 font-medium">Verdict</th>
                    <th className="text-left p-3 text-zinc-400 font-medium">Rule</th>
                  </tr>
                </thead>
                <tbody>
                  {actions.map((a) => {
                    const Icon = verdictIcons[a.verdict] || Shield;
                    const color = verdictColors[a.verdict] || 'text-zinc-400';
                    return (
                      <tr key={a.id} className="border-b border-zinc-800/50 hover:bg-zinc-800/30">
                        <td className="p-3 text-zinc-500 text-xs">{new Date(a.created_at).toLocaleTimeString()}</td>
                        <td className="p-3 text-zinc-400 font-mono text-xs">{a.session_id?.slice(0, 8)}...</td>
                        <td className="p-3 text-zinc-300">{a.action_type}</td>
                        <td className="p-3"><span className={`flex items-center gap-1 text-xs ${color}`}><Icon className="w-3 h-3" />{a.verdict}</span></td>
                        <td className="p-3 text-zinc-500 text-xs">{a.rule_id || '-'}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
