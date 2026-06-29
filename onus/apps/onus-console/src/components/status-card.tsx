import { cn } from '@/lib/utils';
import type { ReactNode } from 'react';

interface StatusCardProps {
  title: string;
  count: number;
  icon: ReactNode;
  variant?: 'success' | 'warning' | 'error' | 'info';
}

export function StatusCard({ title, count, icon, variant = 'info' }: StatusCardProps) {
  const colors = {
    success: 'bg-green-900/20 border-green-800/30 text-green-400',
    warning: 'bg-yellow-900/20 border-yellow-800/30 text-yellow-400',
    error: 'bg-red-900/20 border-red-800/30 text-red-400',
    info: 'bg-blue-900/20 border-blue-800/30 text-blue-400',
  };

  return (
    <div className={cn('rounded-lg border p-4 flex items-center gap-3', colors[variant])}>
      <div className="p-2 rounded-full bg-black/20">{icon}</div>
      <div>
        <p className="text-2xl font-bold">{count}</p>
        <p className="text-xs opacity-80">{title}</p>
      </div>
    </div>
  );
}
