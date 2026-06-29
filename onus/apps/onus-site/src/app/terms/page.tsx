'use client';

import Link from 'next/link';

export default function TermsPage() {
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
          <h1 className="text-3xl font-bold text-white mb-8">Terms of Service</h1>
          <div className="prose prose-invert prose-sm max-w-none text-zinc-400 space-y-4">
            <p>Onus is open-source software provided under the MIT License. This is the license that governs the software; these terms cover use of the website and published release artifacts.</p>
            <h3 className="text-white font-semibold">Software License</h3>
            <p>Onus is licensed under the MIT License. You may use, copy, modify, merge, publish, distribute, sublicense, and sell copies of the software subject to the license terms.</p>
            <h3 className="text-white font-semibold">Disclaimer</h3>
                        <p>THE SOFTWARE IS PROVIDED &quot;AS IS&quot;, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED. In no event shall the authors be liable for any claim, damages or other liability arising from the use of the software.</p>
            <h3 className="text-white font-semibold">Website Use</h3>
                        <p>This website provides documentation and download links for Onus. No account registration is required. No user data is collected.</p>
          </div>
        </div>
      </div>
      <footer className="border-t border-zinc-800 py-8 px-4"><div className="max-w-5xl mx-auto text-center text-xs text-zinc-600">Onus — AI Agent Firewall. Open source (MIT).</div></footer>
    </div>
  );
}
