import { cn } from '@/lib/utils';
import type { ReactNode } from 'react';

interface StatCardProps {
  title: string;
  value: string;
  icon: ReactNode;
  status?: 'active' | 'inactive';
}

export function StatCard({ title, value, icon, status }: StatCardProps) {
  return (
    <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4">
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs text-zinc-500 uppercase tracking-wider">{title}</span>
        <div className={cn(
          'p-1.5 rounded-full',
          status === 'active' ? 'bg-green-900/20 text-green-400' :
          status === 'inactive' ? 'bg-red-900/20 text-red-400' :
          'bg-zinc-800 text-zinc-400'
        )}>
          {icon}
        </div>
      </div>
      <p className="text-lg font-semibold text-white">{value}</p>
    </div>
  );
}
