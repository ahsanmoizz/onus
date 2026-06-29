'use client';

import { useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { DashboardLayout } from '@/components/dashboard-layout';
import { Settings, Save, Loader2 } from 'lucide-react';

interface SettingsData {
  guardian_mode?: string;
  approval_frequency?: string;
  retention_days?: number;
  log_level?: string;
}

export default function SettingsPage() {
  const { data: settings } = useQuery({
    queryKey: ['settings'],
    queryFn: () => api.get<SettingsData>('/settings'),
  });

  const [form, setForm] = useState<SettingsData>({});

  const saveMut = useMutation({
    mutationFn: () => api.put('/settings', form),
  });

  return (
    <DashboardLayout>
      <div className="max-w-2xl mx-auto space-y-6">
        <h1 className="text-2xl font-bold text-white">Settings</h1>

        <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-6 space-y-6">
          <div>
            <label className="block text-sm font-medium text-zinc-300 mb-2">Guardian Mode</label>
            <select
              value={form.guardian_mode || settings?.guardian_mode || ''}
              onChange={e => setForm(f => ({ ...f, guardian_mode: e.target.value }))}
              className="w-full bg-black border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100 focus:outline-none focus:border-accent"
            >
              <option value="">Select mode...</option>
              <option value="beginner">Beginner Guardian</option>
              <option value="professional">Professional Reviewer</option>
              <option value="enterprise">Enterprise Strict</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-zinc-300 mb-2">Approval Frequency</label>
            <select
              value={form.approval_frequency || settings?.approval_frequency || ''}
              onChange={e => setForm(f => ({ ...f, approval_frequency: e.target.value }))}
              className="w-full bg-black border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100 focus:outline-none focus:border-accent"
            >
              <option value="">Select...</option>
              <option value="low">Low — only destructive actions</option>
              <option value="medium">Medium — high-risk actions</option>
              <option value="high">High — all non-trivial actions</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-zinc-300 mb-2">Audit Retention (days)</label>
            <input
              type="number"
              value={form.retention_days ?? settings?.retention_days ?? 90}
              onChange={e => setForm(f => ({ ...f, retention_days: parseInt(e.target.value) }))}
              className="w-full bg-black border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100 focus:outline-none focus:border-accent"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-zinc-300 mb-2">Log Level</label>
            <select
              value={form.log_level || settings?.log_level || ''}
              onChange={e => setForm(f => ({ ...f, log_level: e.target.value }))}
              className="w-full bg-black border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100 focus:outline-none focus:border-accent"
            >
              <option value="info">Info</option>
              <option value="debug">Debug</option>
              <option value="warn">Warning</option>
              <option value="error">Error</option>
            </select>
          </div>

          <button
            onClick={() => saveMut.mutate()}
            disabled={saveMut.isPending}
            className="flex items-center gap-2 px-4 py-2 bg-accent text-black rounded-lg text-sm font-medium hover:bg-accent-hover transition-colors disabled:opacity-50"
          >
            {saveMut.isPending ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
            Save Settings
          </button>
          {saveMut.isSuccess && <p className="text-green-400 text-sm">Settings saved.</p>}
          {saveMut.isError && <p className="text-red-400 text-sm">Failed to save settings.</p>}
        </div>
      </div>
    </DashboardLayout>
  );
}
