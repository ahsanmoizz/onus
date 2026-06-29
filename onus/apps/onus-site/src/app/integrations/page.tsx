'use client';

import Link from 'next/link';
import { Plug, ArrowRight, Shield, Wifi, WifiOff } from 'lucide-react';

const integrations = [
  { name: 'Claude Code CLI', status: 'Runtime-tested with limits', level: 'L1 BEST-EFFORT', continuity: true, tone: 'verified', desc: 'Version-pinned cooperative hook path. It remains bypassable if the hook is disabled, so it must not be described as containment.' },
  { name: 'Generic MCP Gateway', status: 'Runtime-tested routed path', level: 'L2 ROUTED ONLY', continuity: false, tone: 'verified', desc: 'MCP traffic governed when the client talks through the Onus gateway. Direct-server bypass remains documented.' },
  { name: 'Python Bindings', status: 'Runtime-tested routed path', level: 'L2 ROUTED ONLY', continuity: false, tone: 'verified', desc: 'SDK path for custom agents that explicitly route actions through Onus evaluation and receipts.' },
  { name: 'Shell Wrapper', status: 'Available routed path', level: 'L2 ROUTED ONLY', continuity: false, tone: 'limited', desc: 'Commands are governed only when executed through the Onus-owned wrapper or run command.' },
  { name: 'VS Code Extension', status: 'Best-effort partial', level: 'L1 BEST-EFFORT', continuity: false, tone: 'limited', desc: 'Cooperative editor integration surface. Not a process, filesystem, or credential boundary.' },
  { name: 'OpenAI Codex CLI', status: 'Protocol-only', level: 'UNVERIFIED ADAPTER', continuity: false, tone: 'planned', desc: 'Use only through supported routed protocols until a version-pinned adapter report exists.' },
  { name: 'Cursor IDE', status: 'Protocol-only', level: 'UNVERIFIED ADAPTER', continuity: false, tone: 'planned', desc: 'No standalone runtime-certified Cursor adapter claim should be made from the current evidence.' },
  { name: 'Google Antigravity', status: 'Protocol-only', level: 'UNVERIFIED ADAPTER', continuity: false, tone: 'planned', desc: 'No standalone runtime-certified Antigravity adapter claim should be made from the current evidence.' },
];

export default function IntegrationsPage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-zinc-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <Link href="/" className="flex items-center gap-2">
              <div className="w-7 h-7 rounded-full bg-accent flex items-center justify-center">
                <span className="text-black text-xs font-bold">O</span>
              </div>
              <span className="font-bold text-white text-lg">Onus</span>
            </Link>
          </div>
        </div>
      </nav>

      <div className="pt-24 pb-20 px-4">
        <div className="max-w-5xl mx-auto">
          <h1 className="text-4xl font-bold text-white text-center mb-4">Integrations</h1>
          <p className="text-zinc-400 text-center max-w-2xl mx-auto mb-12">
            Onus integrates with popular AI coding agents and development tools.
          </p>

          <div className="space-y-3">
            {integrations.map((item, i) => (
              <div key={i} className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-5">
                <div className="flex items-start justify-between">
                  <div className="flex items-start gap-4">
                    <div className="w-10 h-10 rounded-lg bg-accent/10 flex items-center justify-center flex-shrink-0">
                      <Plug className="w-5 h-5 text-accent" />
                    </div>
                    <div>
                      <div className="flex items-center gap-2 mb-1">
                        <h3 className="font-medium text-white">{item.name}</h3>
                        <span className={
                          item.tone === 'verified'
                            ? 'text-[10px] px-2 py-0.5 rounded-full bg-green-900/30 text-green-400 border border-green-800/30'
                            : item.tone === 'limited'
                              ? 'text-[10px] px-2 py-0.5 rounded-full bg-yellow-900/30 text-yellow-300 border border-yellow-800/30'
                              : 'text-[10px] px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-400 border border-zinc-700'
                        }>{item.status}</span>
                        <span className="text-[10px] px-2 py-0.5 rounded-full bg-accent/10 text-accent border border-accent/20">{item.level}</span>
                        {item.continuity && <span className="text-[10px] px-2 py-0.5 rounded-full bg-blue-900/30 text-blue-400 border border-blue-800/30">Continuity</span>}
                      </div>
                      <p className="text-sm text-zinc-400">{item.desc}</p>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <footer className="border-t border-zinc-800 py-8 px-4">
        <div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus - AI Agent Firewall. Open source (MIT).</div>
      </footer>
    </div>
  );
}
