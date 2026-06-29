'use client';

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { Database, Loader2, Trash2, Archive, Eye } from 'lucide-react';

interface MemoryItem {
  id: string;
  kind: string;
  summary: string;
  status: string;
  created_at: string;
}

export default function MemoryPage() {
  const queryClient = useQueryClient();
  const { data: memories, isLoading } = useQuery({
    queryKey: ['memories'],
    queryFn: () => api.get<MemoryItem[]>('/memory'),
  });

  const deleteMut = useMutation({
    mutationFn: (id: string) => api.delete(`/memory/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['memories'] }),
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <h1 className="text-2xl font-bold text-white">Memory</h1>
        {isLoading ? (
          <p className="text-zinc-500">Loading memories...</p>
        ) : !memories || memories.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <Database className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No memories stored</p>
          </div>
        ) : (
          <div className="space-y-2">
            {memories.map((m) => (
              <div key={m.id} className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 flex items-center justify-between">
                <div className="flex items-start gap-3">
                  <Database className="w-4 h-4 text-zinc-500 mt-0.5" />
                  <div>
                    <p className="text-sm text-zinc-300">{m.summary}</p>
                    <p className="text-xs text-zinc-500 font-mono mt-1">{m.kind} | {m.status} | {new Date(m.created_at).toLocaleDateString()}</p>
                  </div>
                </div>
                <div className="flex gap-2">
                  <button title="Archive" className="p-1.5 text-zinc-500 hover:text-zinc-300"><Archive className="w-4 h-4" /></button>
                  <button title="Delete" onClick={() => deleteMut.mutate(m.id)} className="p-1.5 text-zinc-500 hover:text-red-400"><Trash2 className="w-4 h-4" /></button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
