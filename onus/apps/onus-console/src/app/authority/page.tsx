'use client';

import { useQuery, useMutation } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { Key, Loader2, RotateCcw, XCircle } from 'lucide-react';
import { useState } from 'react';

interface AuthorityCapability {
  id: string;
  authority: string;
  session: string;
  expires_at: number;
  status: string;
  payload_hash: string;
}

export default function AuthorityPage() {
  const [authority, setAuthority] = useState<string>('default');

  const { data: caps, isLoading } = useQuery({
    queryKey: ['authority', authority],
    queryFn: () => api.get<AuthorityCapability[]>(`/authority?authority=${authority}`),
  });

  const revokeMut = useMutation({
    mutationFn: (id: string) => api.post(`/authority/revoke`, { capability: id }),
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-white">Authority (L4)</h1>
          <p className="text-zinc-400 text-sm mt-1">Disposable short-lived credentials for controlled operations</p>
        </div>

        <div className="bg-accent/5 border border-accent/10 rounded-lg p-3 text-accent/70 text-xs">
          L4 provides controlled authority with exact payload binding, automatic expiry, and full audit tracking. Never expose underlying credentials.
        </div>

        {isLoading ? (
          <p className="text-zinc-500">Loading capabilities...</p>
        ) : !caps || caps.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <Key className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No active capabilities</p>
          </div>
        ) : (
          <div className="space-y-2">
            {caps.map((c) => (
              <div key={c.id} className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 flex items-center justify-between">
                <div>
                  <p className="text-sm text-zinc-300 font-mono">{c.id.slice(0, 16)}...</p>
                  <p className="text-xs text-zinc-500">Authority: {c.authority} | Session: {c.session?.slice(0, 8)}...</p>
                  <p className="text-xs text-zinc-500">Expires: {new Date(c.expires_at * 1000).toLocaleString()} | Status: {c.status}</p>
                  <p className="text-xs text-zinc-600 font-mono mt-1">Hash: {c.payload_hash?.slice(0, 16)}...</p>
                </div>
                <button onClick={() => revokeMut.mutate(c.id)} className="flex items-center gap-1 text-xs px-3 py-1.5 bg-red-900/20 text-red-400 rounded-lg hover:bg-red-900/40">
                  <XCircle className="w-3 h-3" /> Revoke
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
