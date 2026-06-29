import type { Metadata } from 'next';
import { Inter, JetBrains_Mono } from 'next/font/google';
import './globals.css';

const inter = Inter({ subsets: ['latin'], variable: '--font-inter' });
const mono = JetBrains_Mono({ subsets: ['latin'], variable: '--font-mono' });

export const metadata: Metadata = {
  title: 'Onus — AI Agent Firewall',
  description: 'Onus is a governance and execution-control layer for AI coding agents. It converts unclear requests into bounded task contracts, evaluates agent actions, protects sensitive resources, and preserves accountable execution records.',
  keywords: ['AI safety', 'agent firewall', 'LLM governance', 'AI security', 'prompt intake', 'agent guardrails'],
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <body className={`${inter.variable} ${mono.variable} font-sans antialiased bg-black text-zinc-100`}>
        {children}
      </body>
    </html>
  );
}
