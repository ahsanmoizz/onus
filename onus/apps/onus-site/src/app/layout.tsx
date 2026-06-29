import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Onus — AI Agent Firewall',
  description: 'Onus is a governance and execution-control layer for AI coding agents. It converts unclear requests into bounded task contracts, evaluates agent actions, protects sensitive resources, and preserves accountable execution records.',
  keywords: ['AI safety', 'agent firewall', 'LLM governance', 'AI security', 'prompt intake', 'agent guardrails'],
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <body className="font-sans antialiased bg-black text-zinc-100">
        {children}
      </body>
    </html>
  );
}
