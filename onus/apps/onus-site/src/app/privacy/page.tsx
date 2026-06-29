'use client';

import Link from 'next/link';

export default function PrivacyPage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-zinc-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <Link href="/" className="flex items-center gap-2">
              <div className="w-7 h-7 rounded-full bg-accent flex items-center justify-center"><span className="text-black text-xs font-bold">O</span></div>
              <span className="font-bold text-white text-lg">Onus</span>
            </Link>
          </div>
        </div>
      </nav>
      <div className="pt-24 pb-20 px-4">
        <div className="max-w-3xl mx-auto">
          <h1 className="text-3xl font-bold text-white mb-8">Privacy Policy</h1>
          <div className="prose prose-invert prose-sm max-w-none text-zinc-400 space-y-4">
            <p>Onus operates entirely locally. No telemetry, analytics, or user data is sent to external servers unless explicitly configured by the user.</p>
            <h3 className="text-white font-semibold">Data Collection</h3>
            <p>Onus does not collect or transmit personal data, usage statistics, or error reports. All audit data, configuration, and logs remain on the local machine.</p>
            <h3 className="text-white font-semibold">Provider Configuration</h3>
            <p>When configured with an LLM provider for semantic analysis, the provider endpoint is user-specified and data is sent only as required for analysis. Provider keys are stored locally and never transmitted to Onus servers.</p>
            <h3 className="text-white font-semibold">Updates</h3>
            <p>Onus checks for updates by querying GitHub releases. This is the only outbound network request and can be disabled.</p>
            <h3 className="text-white font-semibold">Third-Party Code</h3>
                        <p>Onus uses open source dependencies listed in the project&apos;s Cargo.lock file. No third-party analytics or tracking libraries are included.</p>
          </div>
        </div>
      </div>
      <footer className="border-t border-zinc-800 py-8 px-4"><div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus — AI Agent Firewall. Open source (MIT).</div></footer>
    </div>
  );
}
