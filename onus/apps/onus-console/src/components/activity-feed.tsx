'use client';

import { useQuery } from '@tanstack/react-query';
import { getRecentActions } from '@/lib/api';
import { Shield, AlertTriangle, CheckCircle, XCircle } from 'lucide-react';

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

export function ActivityFeed() {
  const { data: actions, isLoading } = useQuery({
    queryKey: ['actions'],
    queryFn: () => getRecentActions(10),
    refetchInterval: 3000,
  });

  return (
    <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4">
      <h3 className="text-sm font-semibold text-zinc-300 mb-3">Recent Actions</h3>
      {isLoading ? (
        <p className="text-zinc-500 text-sm">Loading...</p>
      ) : !actions || actions.length === 0 ? (
        <p className="text-zinc-500 text-sm">No recent actions</p>
      ) : (
        <div className="space-y-1">
          {actions.map((action) => {
            const Icon = verdictIcons[action.verdict] || Shield;
            const color = verdictColors[action.verdict] || 'text-zinc-400';
            return (
              <div key={action.id} className="flex items-center gap-3 py-2 text-sm border-b border-zinc-800/50 last:border-0">
                <Icon className={`w-3.5 h-3.5 flex-shrink-0 ${color}`} />
                <span className="text-zinc-400 flex-1 truncate">{action.action_type}</span>
                <span className={`text-xs font-medium ${color}`}>{action.verdict}</span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
