'use client';

import { useQuery } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { Plug, CheckCircle, XCircle, AlertTriangle, ExternalLink } from 'lucide-react';

interface Integration {
  name: string;
  installed: boolean;
  authenticated: boolean;
  configured: boolean;
  enforcement_level: string;
  doctor_status: 'ok' | 'warn' | 'fail';
}

export default function IntegrationsPage() {
  const { data: integrations, isLoading } = useQuery({
    queryKey: ['integrations'],
    queryFn: () => api.get<Integration[]>('/integrations'),
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <h1 className="text-2xl font-bold text-white">Integrations</h1>
        {isLoading ? (
          <p className="text-zinc-500">Loading integrations...</p>
        ) : !integrations || integrations.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <Plug className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No integrations configured</p>
            <p className="text-zinc-600 text-sm mt-1">Set up integrations via `onus setup --&lt;name&gt;`</p>
          </div>
        ) : (
          <div className="space-y-3">
            {integrations.map((item) => (
              <div key={item.name} className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4">
                <div className="flex items-center justify-between mb-3">
                  <h3 className="font-medium text-white">{item.name}</h3>
                  <span className={`text-xs px-2 py-0.5 rounded-full ${item.enforcement_level === 'L2' ? 'bg-blue-900/50 text-blue-400' : 'bg-zinc-800 text-zinc-500'}`}>
                    {item.enforcement_level}
                  </span>
                </div>
                <div className="grid grid-cols-3 gap-4 text-sm">
                  <div className="flex items-center gap-2">
                    {item.installed ? <CheckCircle className="w-4 h-4 text-green-400" /> : <XCircle className="w-4 h-4 text-red-400" />}
                    <span className="text-zinc-400">Installed</span>
                  </div>
                  <div className="flex items-center gap-2">
                    {item.authenticated ? <CheckCircle className="w-4 h-4 text-green-400" /> : <AlertTriangle className="w-4 h-4 text-yellow-400" />}
                    <span className="text-zinc-400">Authenticated</span>
                  </div>
                  <div className="flex items-center gap-2">
                    {item.configured ? <CheckCircle className="w-4 h-4 text-green-400" /> : <XCircle className="w-4 h-4 text-red-400" />}
                    <span className="text-zinc-400">Configured</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
