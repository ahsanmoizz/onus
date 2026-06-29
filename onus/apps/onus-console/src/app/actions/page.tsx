'use client';

import { useQuery } from '@tanstack/react-query';
import { getRecentActions } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { History, Shield, AlertTriangle, CheckCircle, XCircle } from 'lucide-react';

const verdictIcons: Record<string, typeof Shield> = {
  allow: CheckCircle,
  warn: AlertTriangle,
  block: XCircle,
  escalate: Shield,
};

const verdictColors: Record<string, string> = {
  allow: 'text-green-400',
  warn: 'text-yellow-400',
  block: 'text-red-400',
  escalate: 'text-orange-400',
};

export default function ActionsPage() {
  const { data: actions, isLoading } = useQuery({
    queryKey: ['actions'],
    queryFn: () => getRecentActions(50),
    refetchInterval: 3000,
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <h1 className="text-2xl font-bold text-white">Action Stream</h1>
        {isLoading ? (
          <p className="text-zinc-500">Loading...</p>
        ) : !actions || actions.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <History className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No actions recorded yet</p>
          </div>
        ) : (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-zinc-800">
                    <th className="text-left p-3 text-zinc-400 font-medium">Time</th>
                    <th className="text-left p-3 text-zinc-400 font-medium">Type</th>
                    <th className="text-left p-3 text-zinc-400 font-medium">Tool</th>
                    <th className="text-left p-3 text-zinc-400 font-medium">Verdict</th>
                  </tr>
                </thead>
                <tbody>
                  {actions.map((a) => {
                    const Icon = verdictIcons[a.verdict] || Shield;
                    const color = verdictColors[a.verdict] || 'text-zinc-400';
                    return (
                      <tr key={a.id} className="border-b border-zinc-800/50 hover:bg-zinc-800/30">
                        <td className="p-3 text-zinc-400 text-xs">{new Date(a.created_at).toLocaleTimeString()}</td>
                        <td className="p-3 text-zinc-300">{a.action_type}</td>
                        <td className="p-3 text-zinc-500">{a.tool_name || '-'}</td>
                        <td className="p-3">
                          <span className={`flex items-center gap-1 text-xs font-medium ${color}`}>
                            <Icon className="w-3 h-3" /> {a.verdict}
                          </span>
                        </td>
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
