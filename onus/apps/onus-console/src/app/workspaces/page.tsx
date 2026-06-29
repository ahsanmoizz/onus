'use client';

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { GitBranch, Plus, Loader2, Trash2, ExternalLink } from 'lucide-react';
import { useState } from 'react';

interface Workspace {
  id: string;
  session_id: string;
  path: string;
  network_allowed: boolean;
  created_at: string;
  status: string;
}

export default function WorkspacesPage() {
  const queryClient = useQueryClient();
  const [showCreate, setShowCreate] = useState(false);
  const [repoPath, setRepoPath] = useState('');
  const [sessionId, setSessionId] = useState('');

  const { data: workspaces, isLoading } = useQuery({
    queryKey: ['workspaces'],
    queryFn: () => api.get<Workspace[]>('/workspaces'),
  });

  const createMut = useMutation({
    mutationFn: () => api.post('/workspaces', { repo: repoPath || '.', session: sessionId }),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ['workspaces'] }); setShowCreate(false); },
  });

  const destroyMut = useMutation({
    mutationFn: (id: string) => api.delete(`/workspaces/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['workspaces'] }),
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-white">Workspaces</h1>
            <p className="text-zinc-400 text-sm mt-1">L3 containerized execution environments</p>
          </div>
          <button onClick={() => setShowCreate(true)} className="flex items-center gap-1 text-xs px-3 py-1.5 bg-accent text-black rounded-full font-medium hover:bg-accent-hover">
            <Plus className="w-3 h-3" /> Create Workspace
          </button>
        </div>

        <div className="bg-blue-900/10 border border-blue-800/20 rounded-lg p-3 text-blue-400/70 text-xs">
          L3 workspaces require Linux (native or WSL). Containerization via bubblewrap.
        </div>

        {showCreate && (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 space-y-3">
            <input value={sessionId} onChange={e => setSessionId(e.target.value)} placeholder="Session ID" className="w-full bg-black border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100" />
            <input value={repoPath} onChange={e => setRepoPath(e.target.value)} placeholder="Repository path (default: .)" className="w-full bg-black border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100" />
            <div className="flex gap-2">
              <button onClick={() => createMut.mutate()} disabled={!sessionId || createMut.isPending} className="flex items-center gap-1 px-3 py-1.5 bg-accent text-black rounded-lg text-sm font-medium disabled:opacity-50">
                {createMut.isPending && <Loader2 className="w-3 h-3 animate-spin" />}Create
              </button>
              <button onClick={() => setShowCreate(false)} className="px-3 py-1.5 border border-zinc-700 text-zinc-300 rounded-lg text-sm">Cancel</button>
            </div>
          </div>
        )}

        {isLoading ? (
          <p className="text-zinc-500">Loading workspaces...</p>
        ) : !workspaces || workspaces.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <GitBranch className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No workspaces</p>
            <p className="text-zinc-600 text-sm mt-1">Create a workspace to enable L3 containment</p>
          </div>
        ) : (
          <div className="space-y-2">
            {workspaces.map((w) => (
              <div key={w.id} className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 flex items-center justify-between">
                <div>
                  <p className="text-sm text-zinc-300">Session: <code className="font-mono text-xs">{w.session_id?.slice(0, 12)}...</code></p>
                  <p className="text-xs text-zinc-500 mt-1">Path: {w.path}</p>
                  <p className="text-xs text-zinc-500">Status: {w.status} | Network: {w.network_allowed ? 'allowed' : 'blocked'}</p>
                </div>
                <div className="flex gap-2">
                  <button title="Export" className="p-1.5 text-zinc-500 hover:text-zinc-300"><ExternalLink className="w-4 h-4" /></button>
                  <button title="Destroy" onClick={() => destroyMut.mutate(w.id)} className="p-1.5 text-zinc-500 hover:text-red-400"><Trash2 className="w-4 h-4" /></button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
