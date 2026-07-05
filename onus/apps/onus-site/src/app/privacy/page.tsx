'use client';

import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function PrivacyPage() {
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
        <div className="max-w-3xl mx-auto">
          <h1 className="text-3xl font-bold text-white mb-8">Privacy Policy</h1>
          <div className="prose prose-invert prose-sm max-w-none text-zinc-400 space-y-4">
            <p>Onus keeps audit data, receipts, local console state, and configuration on the user&apos;s machine. Semantic review may call the managed Onus gateway when enabled.</p>
            <h3 className="text-white font-semibold">Data Collection</h3>
            <p>Onus does not collect or transmit personal data, usage statistics, or error reports. All audit data, configuration, and logs remain on the local machine.</p>
            <h3 className="text-white font-semibold">Provider Configuration</h3>
            <p>For managed semantic review, the user&apos;s machine sends redacted review payloads to the Onus gateway. The raw model-provider key stays on the VPS and is never distributed to users.</p>
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
