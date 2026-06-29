'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { cn } from '@/lib/utils';
import { BrandLogo } from '@/components/brand-logo';
import {
  LayoutDashboard,
  Terminal,
  ShieldCheck,
  FileCheck,
  History,
  Users,
  Settings,
  AlertTriangle,
  RefreshCw,
  Database,
  FileText,
  GitBranch,
  Key,
  Plug,
  Bot,
  Activity,
  Menu,
  X,
} from 'lucide-react';
import { useState } from 'react';

const navItems = [
  { href: '/', label: 'Overview', icon: LayoutDashboard },
  { href: '/intake', label: 'New Task', icon: Terminal },
  { href: '/sessions', label: 'Sessions', icon: Activity },
  { href: '/approvals', label: 'Approvals', icon: ShieldCheck },
  { href: '/actions', label: 'Action Stream', icon: History },
  { href: '/checkpoints', label: 'Checkpoints', icon: RefreshCw },
  { href: '/rollback', label: 'Rollback', icon: AlertTriangle },
  { href: '/audit', label: 'Audit & Receipts', icon: FileText },
  { href: '/memory', label: 'Memory', icon: Database },
  { href: '/rules', label: 'Rules & Policies', icon: FileCheck },
  { href: '/workspaces', label: 'Workspaces', icon: GitBranch },
  { href: '/authority', label: 'Authority', icon: Key },
  { href: '/integrations', label: 'Integrations', icon: Plug },
  { href: '/agents', label: 'Agents', icon: Bot },
  { href: '/providers', label: 'Providers', icon: Database },
  { href: '/doctor', label: 'Doctor', icon: Users },
  { href: '/settings', label: 'Settings', icon: Settings },
];

export function DashboardLayout({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="flex h-screen overflow-hidden bg-black">
      {sidebarOpen && (
        <div className="fixed inset-0 z-40 bg-black/50 lg:hidden" onClick={() => setSidebarOpen(false)} />
      )}

      <aside className={cn(
        'fixed lg:static inset-y-0 left-0 z-50 w-64 bg-zinc-900/50 border-r border-zinc-800 transform transition-transform duration-200 lg:transform-none overflow-y-auto',
        sidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'
      )}>
        <div className="p-4 border-b border-zinc-800 flex items-center justify-between">
          <Link href="/" className="flex items-center gap-2" aria-label="Onus console home">
            <BrandLogo imageClassName="h-9 w-auto" />
            <span className="rounded-full border border-white/10 px-2 py-0.5 text-[10px] uppercase tracking-[0.22em] text-zinc-500">
              Console
            </span>
          </Link>
          <button className="lg:hidden text-zinc-400" onClick={() => setSidebarOpen(false)}>
            <X className="w-5 h-5" />
          </button>
        </div>
        <nav className="p-2 space-y-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = pathname === item.href;
            return (
              <Link
                key={item.href}
                href={item.href}
                onClick={() => setSidebarOpen(false)}
                className={cn(
                  'flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors',
                  isActive
                    ? 'bg-accent/10 text-accent'
                    : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/50'
                )}
              >
                <Icon className="w-4 h-4 flex-shrink-0" />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>
      </aside>

      <div className="flex-1 flex flex-col min-w-0">
        <header className="h-14 border-b border-zinc-800 flex items-center px-4 gap-4 bg-zinc-900/30">
          <button className="lg:hidden text-zinc-400" onClick={() => setSidebarOpen(true)}>
            <Menu className="w-5 h-5" />
          </button>
          <div className="flex-1" />
          <div className="flex items-center gap-2 text-xs text-zinc-500">
            <span className="inline-block w-2 h-2 rounded-full bg-green-500" />
            Connected
          </div>
        </header>
        <main className="flex-1 overflow-y-auto p-6">
          {children}
        </main>
      </div>
    </div>
  );
}
