'use client';

import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { AlertTriangle, Loader2, ArrowLeft } from 'lucide-react';

export default function RollbackPage() {
  const [mode, setMode] = useState<'action' | 'group' | 'session'>('action');
  const [id, setId] = useState('');
  const [reason, setReason] = useState('');

  const rollbackMut = useMutation({
    mutationFn: () => api.post(`/rollback/${mode}`, { [`${mode}_id`]: id, reason }),
  });

  return (
    <DashboardLayout>
      <div className="max-w-2xl mx-auto space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-white">Rollback</h1>
          <p className="text-zinc-400 text-sm mt-1">Roll back actions, groups, or entire sessions</p>
        </div>

        <div className="flex gap-2">
          {(['action', 'group', 'session'] as const).map((m) => (
            <button key={m} onClick={() => setMode(m)} className={`text-xs px-4 py-2 rounded-full border transition-colors ${mode === m ? 'bg-accent text-black border-accent' : 'border-zinc-700 text-zinc-400 hover:text-zinc-200'}`}>
              {m.charAt(0).toUpperCase() + m.slice(1)}
            </button>
          ))}
        </div>

        <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-6 space-y-4">
          <div>
            <label className="block text-sm text-zinc-300 mb-1">{mode === 'action' ? 'Action ID' : mode === 'group' ? 'Group ID' : 'Session ID'}</label>
            <input value={id} onChange={e => setId(e.target.value)} placeholder={`Enter ${mode} ID...`} className="w-full bg-black border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-accent" />
          </div>
          <div>
            <label className="block text-sm text-zinc-300 mb-1">Reason</label>
            <textarea value={reason} onChange={e => setReason(e.target.value)} rows={3} placeholder="Why is this rollback needed?" className="w-full bg-black border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-accent resize-none" />
          </div>
          <button onClick={() => rollbackMut.mutate()} disabled={!id || !reason || rollbackMut.isPending} className="flex items-center gap-2 px-4 py-2 bg-red-600/20 text-red-400 border border-red-800/30 rounded-lg text-sm font-medium hover:bg-red-600/30 transition-colors disabled:opacity-50">
            {rollbackMut.isPending ? <Loader2 className="w-4 h-4 animate-spin" /> : <AlertTriangle className="w-4 h-4" />}
            Rollback {mode.charAt(0).toUpperCase() + mode.slice(1)}
          </button>
          {rollbackMut.isError && <p className="text-red-400 text-sm">Rollback failed. Check the ID and try again.</p>}
          {rollbackMut.isSuccess && <p className="text-green-400 text-sm">Rollback initiated successfully.</p>}
        </div>

        <div className="bg-yellow-900/10 border border-yellow-800/20 rounded-lg p-4 text-yellow-400/80 text-xs">
          <AlertTriangle className="w-4 h-4 inline mr-1" />
          Rollback may have irreversible effects. Verify before proceeding.
        </div>
      </div>
    </DashboardLayout>
  );
}
