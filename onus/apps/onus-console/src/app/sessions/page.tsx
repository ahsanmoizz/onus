'use client';

import { useQuery } from '@tanstack/react-query';
import { getSessions } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { Activity } from 'lucide-react';

export default function SessionsPage() {
  const { data: sessions, isLoading } = useQuery({
    queryKey: ['sessions'],
    queryFn: getSessions,
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <h1 className="text-2xl font-bold text-white">Sessions</h1>
        {isLoading ? (
          <p className="text-zinc-500">Loading sessions...</p>
        ) : !sessions || sessions.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <Activity className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No sessions yet</p>
            <p className="text-zinc-600 text-sm mt-1">Start a governed task to create a session</p>
          </div>
        ) : (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg overflow-hidden">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-800">
                  <th className="text-left p-3 text-zinc-400 font-medium">Session</th>
                  <th className="text-left p-3 text-zinc-400 font-medium">Agent</th>
                  <th className="text-left p-3 text-zinc-400 font-medium">Task</th>
                  <th className="text-left p-3 text-zinc-400 font-medium">Status</th>
                  <th className="text-right p-3 text-zinc-400 font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {sessions.map((s) => (
                  <tr key={s.id} className="border-b border-zinc-800/50 hover:bg-zinc-800/30">
                    <td className="p-3 text-zinc-300 font-mono text-xs">{s.id.slice(0, 8)}...</td>
                    <td className="p-3 text-zinc-300">{s.agent_name}</td>
                    <td className="p-3 text-zinc-400 max-w-[200px] truncate">{s.task_description}</td>
                    <td className="p-3">
                      <span className={`text-xs px-2 py-0.5 rounded-full ${s.status === 'active' ? 'bg-green-900/50 text-green-400' : 'bg-zinc-800 text-zinc-400'}`}>{s.status}</span>
                    </td>
                    <td className="p-3 text-right text-zinc-500">{s.total_actions}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
