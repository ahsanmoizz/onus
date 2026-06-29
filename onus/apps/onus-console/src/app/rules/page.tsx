'use client';

import { useQuery } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { FileCheck, Shield, ShieldAlert, ShieldCheck } from 'lucide-react';

interface Rule {
  id: string;
  description: string;
  pattern: string;
  action: 'allow' | 'block' | 'escalate';
  enabled: boolean;
}

export default function RulesPage() {
  const { data: rules, isLoading } = useQuery({
    queryKey: ['rules'],
    queryFn: () => api.get<Rule[]>('/rules'),
  });

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <h1 className="text-2xl font-bold text-white">Rules & Policies</h1>
        {isLoading ? (
          <p className="text-zinc-500">Loading rules...</p>
        ) : !rules || rules.length === 0 ? (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-8 text-center">
            <FileCheck className="w-8 h-8 text-zinc-600 mx-auto mb-2" />
            <p className="text-zinc-500">No rules configured. Run `onus rules init` to create default rules.</p>
          </div>
        ) : (
          <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg overflow-hidden">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-800">
                  <th className="text-left p-3 text-zinc-400 font-medium">Rule</th>
                  <th className="text-left p-3 text-zinc-400 font-medium">Pattern</th>
                  <th className="text-left p-3 text-zinc-400 font-medium">Action</th>
                  <th className="text-left p-3 text-zinc-400 font-medium">Status</th>
                </tr>
              </thead>
              <tbody>
                {rules.map((r) => (
                  <tr key={r.id} className="border-b border-zinc-800/50 hover:bg-zinc-800/30">
                    <td className="p-3 text-zinc-300">{r.description}</td>
                    <td className="p-3 text-zinc-500 font-mono text-xs">{r.pattern}</td>
                    <td className="p-3">
                      <span className={`flex items-center gap-1 text-xs ${r.action === 'block' ? 'text-red-400' : r.action === 'escalate' ? 'text-orange-400' : 'text-green-400'}`}>
                        {r.action === 'block' ? <ShieldAlert className="w-3 h-3" /> : r.action === 'escalate' ? <Shield className="w-3 h-3" /> : <ShieldCheck className="w-3 h-3" />}
                        {r.action}
                      </span>
                    </td>
                    <td className="p-3">
                      <span className={`text-xs px-2 py-0.5 rounded-full ${r.enabled ? 'bg-green-900/50 text-green-400' : 'bg-zinc-800 text-zinc-500'}`}>
                        {r.enabled ? 'Enabled' : 'Disabled'}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </DashboardLayout>
  );
}
