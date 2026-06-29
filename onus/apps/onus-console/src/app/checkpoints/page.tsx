'use client';

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getCheckpoints, createCheckpoint, api } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { RefreshCw, Plus, Eye, RotateCcw, Loader2 } from 'lucide-react';
import { useState } from 'react';

export default function CheckpointsPage() {
  const queryClient = useQueryClient();
  const [showCreate, setShowCreate] = useState(false);
  const [sessionId, setSessionId] = useState('');
  const [description, setDescription] = useState('');

  const { data: checkpoints, isLoading } = useQuery({
    queryKey: ['checkpoints'],
    queryFn: getCheckpoints,
  });

  const createMut = useMutation({
    mutationFn: () => createCheckpoint(sessionId, description),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['checkpoints'] });
      setShowCreate(false);
      setSessionId('');
      setDescription('');
    },
  });

  const restoreMut = useMutation({
    mutationFn: (id: string) => api.post('/checkpoints/restore', { id }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['checkpoints'] }),
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-white">Checkpoints</h1>
            <p className="text-zinc-400 text-sm mt-1">Create and restore workspace snapshots</p>
          </div>
          <button onClick={() => setShowCreate(true)} className="flex items-center gap-1 text-xs px-3 py-1.5 bg-accent text-black rounded-full font-medium hover:bg-accent-hover transition-colors">
            <Plus className="w-3 h-3" /> Create Checkpoint
          </button>
        </div>

        {showCreate && (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 space-y-3">
            <h3 className="text-sm font-medium text-zinc-300">New Checkpoint</h3>
            <input value={sessionId} onChange={e => setSessionId(e.target.value)} placeholder="Session ID" className="w-full bg-black border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100 placeholder-zinc-600" />
            <input value={description} onChange={e => setDescription(e.target.value)} placeholder="Description (optional)" className="w-full bg-black border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100 placeholder-zinc-600" />
            <div className="flex gap-2">
              <button onClick={() => createMut.mutate()} disabled={!sessionId || createMut.isPending} className="flex items-center gap-1 px-3 py-1.5 bg-accent text-black rounded-lg text-sm font-medium disabled:opacity-50">
                {createMut.isPending ? <Loader2 className="w-3 h-3 animate-spin" /> : <RefreshCw className="w-3 h-3" />} Create
              </button>
              <button onClick={() => setShowCreate(false)} className="px-3 py-1.5 border border-zinc-700 text-zinc-300 rounded-lg text-sm">Cancel</button>
            </div>
          </div>
        )}

        {isLoading ? (
          <p className="text-zinc-500">Loading checkpoints...</p>
        ) : !checkpoints || checkpoints.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <RefreshCw className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No checkpoints yet</p>
          </div>
        ) : (
          <div className="space-y-2">
            {checkpoints.map((cp) => (
              <div key={cp.id} className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 flex items-center justify-between">
                <div>
                  <p className="text-sm text-zinc-300">{cp.description || 'Checkpoint'}</p>
                  <p className="text-xs text-zinc-500 font-mono mt-1">ID: {cp.id} | Session: {cp.session_id?.slice(0, 8)}...</p>
                  <p className="text-xs text-zinc-600">{new Date(cp.created_at).toLocaleString()}</p>
                </div>
                <div className="flex gap-2">
                  <button className="flex items-center gap-1 text-xs px-3 py-1.5 bg-zinc-800 text-zinc-300 rounded-lg hover:bg-zinc-700"><Eye className="w-3 h-3" /> Inspect</button>
                  <button onClick={() => restoreMut.mutate(cp.id)} className="flex items-center gap-1 text-xs px-3 py-1.5 bg-yellow-900/20 text-yellow-400 rounded-lg hover:bg-yellow-900/40"><RotateCcw className="w-3 h-3" /> Restore</button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
