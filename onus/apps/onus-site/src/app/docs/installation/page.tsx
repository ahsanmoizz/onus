import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function InstallationPage() {
  return (
    <div className="min-h-screen">
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-sm border-b border-zinc-800">
        <div className="max-w-6xl mx-auto px-4 h-14 flex items-center justify-between">
          <Link href="/" className="inline-flex items-center" aria-label="Onus home"><BrandLogo imageClassName="h-9 w-auto" /></Link>
          <div className="flex items-center gap-6 text-sm text-zinc-400">
            <Link href="/product" className="hover:text-white transition-colors">Product</Link>
            <Link href="/install" className="hover:text-white transition-colors">Install</Link>
            <Link href="/docs" className="text-accent transition-colors">Docs</Link>
          </div>
        </div>
      </nav>

      <main className="pt-20 pb-16 px-4 max-w-4xl mx-auto">
        <Link href="/docs" className="text-sm text-zinc-400 hover:text-white transition-colors inline-flex items-center gap-1 mb-8">&larr; Back to Docs</Link>

        <h1 className="text-3xl font-bold text-white mb-4">Installation</h1>

        <p className="text-zinc-300 leading-relaxed mb-6">
          Onus runs on Linux x86_64 and Windows x86_64 (native PowerShell or WSL2 if you want L3 containment). The recommended installation method is the download script, which handles dependencies, platform detection, and PATH setup automatically. You can also build from source if you prefer to verify the binary yourself.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">System Requirements</h2>
        <ul className="list-disc list-inside text-zinc-300 space-y-2 mb-6 ml-4">
          <li><strong className="text-white">Operating System:</strong> Linux x86_64 (any modern distribution), or Windows 10/11 x86_64</li>
          <li><strong className="text-white">Build:</strong> <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">cargo build --release</code> (requires Rust, only for building from source)</li>
          <li><strong className="text-white">Disk:</strong> Approximately 50 MB for the Onus binary</li>
          <li><strong className="text-white">Memory:</strong> 256 MB minimum, 1 GB recommended when using local LLM evaluation</li>
          <li><strong className="text-white">Network:</strong> Outbound HTTPS access to the provider API endpoint if using Cloud or Local evaluation modes. Deterministic-only mode works fully offline.</li>
        </ul>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Quick Install (Recommended)</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Run the install script from your terminal. On Linux x86_64:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4"># Download the installer script
curl -fsSL https://github.com/ahsanmoizz/onus/releases/latest/download/install.sh -o install-onus.sh

# Review the script (always verify before running)
less install-onus.sh

# Run the installer
bash install-onus.sh</pre>
        <p className="text-zinc-300 leading-relaxed mb-4">
          On Windows (PowerShell):
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4"># Download the installer script
Invoke-WebRequest -Uri &quot;https://github.com/ahsanmoizz/onus/releases/latest/download/install.ps1&quot; -OutFile &quot;install-onus.ps1&quot;

# Review the script (always verify before running)
notepad install-onus.ps1

# Run the installer
powershell -ExecutionPolicy Bypass -File install-onus.ps1</pre>
        <p className="text-zinc-300 leading-relaxed mb-4">
          The script detects your platform and architecture, downloads the correct binary, verifies the checksum, and installs it to <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">~/.local/bin</code> (Linux) or <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">%LOCALAPPDATA%\onus</code> (Windows).
        </p>
        <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-4 mb-6">
          <p className="text-sm text-zinc-400">
            <strong className="text-white">Note for L3 containment on Windows:</strong> L3 workspace containment requires bubblewrap, which is Linux-only. Windows users who need L3 enforcement should use Onus via WSL2 with Docker or podman.
          </p>
        </div>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Build from Source</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          If you prefer to build from source, install Rust and clone the repository:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4"># Install Rust: https://rustup.rs
curl --proto &#39;=https&#39; --tlsv1.2 -sSf https://sh.rustup.rs | sh

git clone https://github.com/ahsanmoizz/onus.git
cd onus
cargo build --release</pre>
        <p className="text-zinc-300 leading-relaxed mb-4">
          The compiled binary will be at <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">target/release/onus</code>. Copy it to a directory in your PATH:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">cp target/release/onus ~/.local/bin/onus    # Linux
# or on Windows copy to %LOCALAPPDATA%\onus\</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Verify the Installation</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          After installation, verify that Onus is working correctly:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">onus doctor</pre>
        <p className="text-zinc-300 leading-relaxed mb-4">
          The <code className="text-zinc-200 bg-zinc-800 px-1.5 py-0.5 rounded text-sm font-mono">onus doctor</code> command checks that the binary is correctly installed, the daemon can start, and all required directories and configuration files are in place. If any issues are found, it will provide guidance for resolving them.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Next Steps</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Once Onus is installed, proceed to the <Link href="/docs/quick-start" className="text-accent hover:underline">Quick Start</Link> guide to configure your first guardian mode and provider, or read about <Link href="/docs/guardian-modes" className="text-accent hover:underline">Guardian Modes</Link> to understand the available enforcement options.
        </p>

        <div className="border-t border-zinc-800 mt-12 pt-6">
          <p className="text-sm text-zinc-500">
            Having trouble? Check the <Link href="/docs/troubleshooting" className="text-accent hover:underline">Troubleshooting guide</Link> or open a <a href="https://github.com/ahsanmoizz/onus/issues" className="text-accent hover:underline">GitHub issue</a>.
          </p>
        </div>
      </main>
    </div>
  );
}
