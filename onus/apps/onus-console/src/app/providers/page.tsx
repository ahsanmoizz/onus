'use client';

import { useQuery } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { Database, CheckCircle, XCircle } from 'lucide-react';

interface Provider {
  name: string;
  mode: string;
  configured: boolean;
  connected: boolean;
  model?: string;
  endpoint?: string;
}

export default function ProvidersPage() {
  const { data: providers, isLoading } = useQuery({
    queryKey: ['providers'],
    queryFn: () => api.get<Provider[]>('/providers'),
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <h1 className="text-2xl font-bold text-white">Providers</h1>
        {isLoading ? (
          <p className="text-zinc-500">Loading providers...</p>
        ) : !providers || providers.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <Database className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No providers configured</p>
            <p className="text-zinc-600 text-sm mt-1">Configure a semantic provider via environment variables</p>
          </div>
        ) : (
          <div className="space-y-3">
            {providers.map((p) => (
              <div key={p.name} className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <h3 className="font-medium text-white">{p.name}</h3>
                  <span className={`text-xs px-2 py-0.5 rounded-full ${p.mode === 'deterministic' ? 'bg-zinc-800 text-zinc-400' : 'bg-blue-900/50 text-blue-400'}`}>
                    {p.mode}
                  </span>
                </div>
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div className="flex items-center gap-2 text-zinc-400">
                    {p.configured ? <CheckCircle className="w-4 h-4 text-green-400" /> : <XCircle className="w-4 h-4 text-red-400" />}
                    Configured
                  </div>
                  <div className="flex items-center gap-2 text-zinc-400">
                    {p.connected ? <CheckCircle className="w-4 h-4 text-green-400" /> : <XCircle className="w-4 h-4 text-red-400" />}
                    Connected
                  </div>
                </div>
                {(p.model || p.endpoint) && (
                  <div className="mt-2 text-xs text-zinc-500">
                    {p.model && <p>Model: {p.model}</p>}
                    {p.endpoint && <p>Endpoint: {p.endpoint}</p>}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
