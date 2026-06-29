'use client';

import { useQuery } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { Bot, ArrowRightLeft } from 'lucide-react';

interface Agent {
  name: string;
  status: string;
  current_session?: string;
  eligible_for_handoff: boolean;
}

export default function AgentsPage() {
  const { data: agents, isLoading } = useQuery({
    queryKey: ['agents'],
    queryFn: () => api.get<Agent[]>('/agents'),
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-white">Agents & Continuity</h1>
            <p className="text-zinc-400 text-sm mt-1">Cross-agent handoff with complete task state</p>
          </div>
        </div>

        <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 text-zinc-400 text-sm">
          <p>Agent continuity transfers the full task context: task contract, repository state, checkpoint, memory, evidence, corrections, and audit trail — all verified via manifest hash.</p>
        </div>

        {isLoading ? (
          <p className="text-zinc-500">Loading agents...</p>
        ) : !agents || agents.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <Bot className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No agents configured</p>
          </div>
        ) : (
          <div className="space-y-2">
            {agents.map((a) => (
              <div key={a.name} className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <Bot className="w-5 h-5 text-accent" />
                  <div>
                    <p className="text-sm font-medium text-white">{a.name}</p>
                    <p className="text-xs text-zinc-500">
                      {a.status} {a.current_session && `| Session: ${a.current_session.slice(0, 8)}...`}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {a.eligible_for_handoff && (
                    <span className="text-xs flex items-center gap-1 px-2 py-1 bg-accent/10 text-accent rounded-full">
                      <ArrowRightLeft className="w-3 h-3" /> Handoff available
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
