'use client';

import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';
import { Cable, ArrowRight, CheckCircle } from 'lucide-react';

export default function McpPage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-zinc-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <Link href="/" className="flex items-center gap-2">
              <BrandLogo imageClassName="h-9 w-auto" />
            </Link>
          </div>
        </div>
      </nav>

      <div className="pt-24 pb-20 px-4">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-4xl font-bold text-white text-center mb-4">MCP Proxy</h1>
          <p className="text-zinc-400 text-center max-w-2xl mx-auto mb-12">
            Onus provides a Model Context Protocol (MCP) proxy that intercepts and evaluates tool calls from any MCP-compatible agent.
          </p>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6 mb-8">
            <h2 className="text-lg font-semibold text-white mb-4">How It Works</h2>
            <div className="space-y-3">
              {[
                'Onus runs an MCP proxy server that agents connect to instead of direct MCP endpoints',
                'Tool call requests are intercepted and evaluated by the Onus rules engine',
                'Approved calls are forwarded, denied calls are blocked with a clear reason',
                'All calls are logged in the tamper-evident audit trail',
                'Supports human approval escalation for high-risk tool calls',
              ].map((item, i) => (
                <div key={i} className="flex items-start gap-2 text-sm text-zinc-300">
                  <CheckCircle className="w-3.5 h-3.5 text-accent mt-0.5 flex-shrink-0" />{item}
                </div>
              ))}
            </div>
          </div>

          <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-6">
            <h2 className="text-lg font-semibold text-white mb-4">Configuration</h2>
            <div className="bg-black rounded-lg p-4">
              <code className="text-sm text-zinc-300">
                onus mcp-proxy start --port 8080<br />
                {'# Then configure your agent to use http://localhost:8080 as MCP endpoint'}
              </code>
            </div>
          </div>

          <div className="mt-8 text-center">
            <Link href="/docs" className="text-sm text-accent hover:underline">Full documentation →</Link>
          </div>
        </div>
      </div>

      <footer className="border-t border-zinc-800 py-8 px-4">
        <div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus — AI Agent Firewall. Open source (MIT).</div>
      </footer>
    </div>
  );
}
