'use client';

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getApprovals, approveApproval, denyApproval } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { ShieldCheck, CheckCircle, XCircle } from 'lucide-react';
import { useState } from 'react';

export default function ApprovalsPage() {
  const [filter, setFilter] = useState('pending');
  const queryClient = useQueryClient();

  const { data: approvals, isLoading } = useQuery({
    queryKey: ['approvals', filter],
    queryFn: () => getApprovals(filter),
  });

  const approveMutation = useMutation({
    mutationFn: approveApproval,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['approvals'] }),
  });

  const denyMutation = useMutation({
    mutationFn: (id: string) => denyApproval(id, 'Denied by operator'),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['approvals'] }),
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold text-white">Approvals</h1>
          <div className="flex gap-2">
            {['pending', 'approved', 'rejected', 'all'].map((s) => (
              <button
                key={s}
                onClick={() => setFilter(s)}
                className={`text-xs px-3 py-1.5 rounded-full border transition-colors ${
                  filter === s ? 'bg-accent text-black border-accent' : 'border-zinc-700 text-zinc-400 hover:text-zinc-200'
                }`}
              >
                {s.charAt(0).toUpperCase() + s.slice(1)}
              </button>
            ))}
          </div>
        </div>

        {isLoading ? (
          <p className="text-zinc-500">Loading approvals...</p>
        ) : !approvals || approvals.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <ShieldCheck className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No {filter} approvals</p>
          </div>
        ) : (
          <div className="space-y-3">
            {approvals.map((a) => (
              <div key={a.id} className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <span className={`text-xs px-2 py-0.5 rounded-full ${
                      a.status === 'pending' ? 'bg-yellow-900/50 text-yellow-400' :
                      a.status === 'approved' ? 'bg-green-900/50 text-green-400' : 'bg-red-900/50 text-red-400'
                    }`}>{a.status}</span>
                    <code className="text-xs text-zinc-500 font-mono">{a.id.slice(0, 12)}...</code>
                  </div>
                  {a.status === 'pending' && (
                    <div className="flex gap-2">
                      <button onClick={() => approveMutation.mutate(a.id)} className="flex items-center gap-1 text-xs px-3 py-1.5 bg-green-600/20 text-green-400 rounded-lg hover:bg-green-600/30">
                        <CheckCircle className="w-3 h-3" /> Approve
                      </button>
                      <button onClick={() => denyMutation.mutate(a.id)} className="flex items-center gap-1 text-xs px-3 py-1.5 bg-red-600/20 text-red-400 rounded-lg hover:bg-red-600/30">
                        <XCircle className="w-3 h-3" /> Deny
                      </button>
                    </div>
                  )}
                </div>
                <p className="text-xs text-zinc-500">Session: <code className="font-mono">{a.session_id.slice(0, 8)}...</code></p>
                <p className="text-xs text-zinc-500 mt-1">Expires: {new Date(a.expires_at * 1000).toLocaleString()}</p>
              </div>
            ))}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
