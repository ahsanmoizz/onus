'use client';

import { useQuery } from '@tanstack/react-query';
import { runDoctor } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { Users, CheckCircle, XCircle, AlertTriangle, RefreshCw, Loader2 } from 'lucide-react';

export default function DoctorPage() {
  const { data, isLoading, isError, refetch, isFetching } = useQuery({
    queryKey: ['doctor'],
    queryFn: runDoctor,
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-white">Doctor</h1>
            <p className="text-zinc-400 text-sm mt-1">System health diagnostics</p>
          </div>
          <button onClick={() => refetch()} disabled={isFetching} className="flex items-center gap-1 text-xs px-3 py-1.5 bg-zinc-800 text-zinc-300 rounded-lg hover:bg-zinc-700 disabled:opacity-50">
            {isFetching ? <Loader2 className="w-3 h-3 animate-spin" /> : <RefreshCw className="w-3 h-3" />}
            Run Doctor
          </button>
        </div>

        {isLoading && <p className="text-zinc-500">Running diagnostics...</p>}
        {isError && (
          <div className="bg-red-900/20 border border-red-800/30 rounded-lg p-4 text-red-400 text-sm">
            Doctor check failed. Is the daemon running?
          </div>
        )}

        {data && (
          <div className="space-y-2">
            {data.checks?.map((check, i) => (
              <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  {check.status === 'ok' ? <CheckCircle className="w-5 h-5 text-green-400" /> :
                   check.status === 'warn' ? <AlertTriangle className="w-5 h-5 text-yellow-400" /> :
                   <XCircle className="w-5 h-5 text-red-400" />}
                  <div>
                    <p className="text-sm text-zinc-300">{check.name}</p>
                    <p className="text-xs text-zinc-500">{check.message}</p>
                  </div>
                </div>
                <span className={`text-xs px-2 py-0.5 rounded-full ${
                  check.status === 'ok' ? 'bg-green-900/50 text-green-400' :
                  check.status === 'warn' ? 'bg-yellow-900/50 text-yellow-400' :
                  'bg-red-900/50 text-red-400'
                }`}>{check.status}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
