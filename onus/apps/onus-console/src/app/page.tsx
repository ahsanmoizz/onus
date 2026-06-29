'use client';

import { useQuery } from '@tanstack/react-query';
import { getStatus, runDoctor, getSessions, verifyChain } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { StatusCard } from '@/components/status-card';
import { StatCard } from '@/components/stat-card';
import { ActivityFeed } from '@/components/activity-feed';
import { Shield, Activity, AlertTriangle, CheckCircle, Server, Database, FileCheck } from 'lucide-react';

export default function DashboardPage() {
  const { data: status } = useQuery({
    queryKey: ['status'],
    queryFn: getStatus,
    refetchInterval: 5000,
  });

  const { data: doctor } = useQuery({
    queryKey: ['doctor'],
    queryFn: runDoctor,
  });

  const { data: sessions } = useQuery({
    queryKey: ['sessions'],
    queryFn: getSessions,
  });

  const { data: verify } = useQuery({
    queryKey: ['verify'],
    queryFn: () => verifyChain(),
  });

  const okChecks = doctor?.checks?.filter(c => c.status === 'ok').length ?? 0;
  const warnChecks = doctor?.checks?.filter(c => c.status === 'warn').length ?? 0;
  const failChecks = doctor?.checks?.filter(c => c.status === 'fail').length ?? 0;

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-white">Overview</h1>
          <p className="text-zinc-400 text-sm mt-1">Onus AI Agent Firewall — System Status</p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <StatCard
            title="Daemon"
            value={status?.daemon === 'RUNNING' ? 'Running' : 'Stopped'}
            icon={<Server className="w-4 h-4" />}
            status={status?.daemon === 'RUNNING' ? 'active' : 'inactive'}
          />
          <StatCard
            title="Guardian Mode"
            value={status?.guardian_mode ?? 'Unknown'}
            icon={<Shield className="w-4 h-4" />}
          />
          <StatCard
            title="Provider"
            value={status?.provider_mode ?? 'Unknown'}
            icon={<Database className="w-4 h-4" />}
          />
          <StatCard
            title="Chain Integrity"
            value={verify?.status === 'ok' ? 'Verified' : 'Unchecked'}
            icon={<FileCheck className="w-4 h-4" />}
            status={verify?.status === 'ok' ? 'active' : 'inactive'}
          />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <StatusCard title="Passed" count={okChecks} icon={<CheckCircle className="w-4 h-4" />} variant="success" />
          <StatusCard title="Warnings" count={warnChecks} icon={<AlertTriangle className="w-4 h-4" />} variant="warning" />
          <StatusCard title="Failures" count={failChecks} icon={<Activity className="w-4 h-4" />} variant="error" />
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <ActivityFeed />
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4">
            <h3 className="text-sm font-semibold text-zinc-300 mb-3">Active Sessions</h3>
            {!sessions || sessions.length === 0 ? (
              <p className="text-zinc-500 text-sm">No active sessions</p>
            ) : (
              <div className="space-y-2">
                {sessions.slice(0, 5).map((s) => (
                  <div key={s.id} className="flex items-center justify-between py-2 border-b border-zinc-800 last:border-0">
                    <div className="min-w-0">
                      <p className="text-sm text-zinc-300 truncate">{s.task_description || s.id}</p>
                      <p className="text-xs text-zinc-500">{s.agent_name}</p>
                    </div>
                    <span className={`text-xs px-2 py-0.5 rounded-full ${
                      s.status === 'active' ? 'bg-green-900/50 text-green-400' : 'bg-zinc-800 text-zinc-400'
                    }`}>{s.status}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}
