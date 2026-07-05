import Link from 'next/link';
import { BrandLogo } from '@/components/brand-logo';

export default function L3WorkspacesPage() {
  return (
    <div className="min-h-screen bg-black">
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
        <Link href="/docs" className="text-sm text-accent hover:underline mb-6 inline-block">&larr; Back to Docs</Link>

        <h1 className="text-3xl font-bold text-white mb-4">L3 Workspaces: Containerized Execution</h1>

        <p className="text-zinc-300 leading-relaxed mb-8">
          Level 3 enforcement provides containerized execution environments for AI agent actions.
          L3 workspaces isolate agents at the process, filesystem, network, and credential level,
          preventing them from affecting the host system beyond defined boundaries. L3 requires real
          container technology&mdash;Docker or Podman on Linux, WSL on Windows.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Containerized Isolation</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          L3 workspaces run agent actions inside containers with strict isolation boundaries. The host
          system is protected from agent actions, and each workspace is independent of others. This
          containment model ensures that even if an agent behaves maliciously or makes a catastrophic
          error, the damage is contained within the workspace.
        </p>

        <p className="text-zinc-300 leading-relaxed mb-4">
          The four containment dimensions are:
        </p>
        <ul className="list-disc pl-6 text-zinc-300 space-y-2 mb-6">
          <li><strong className="text-white">Process isolation</strong> &mdash; agent processes run in separate PID namespaces and cannot see or signal host processes.</li>
          <li><strong className="text-white">Filesystem isolation</strong> &mdash; read-only base image with a writable overlay layer. Changes are ephemeral unless explicitly committed.</li>
          <li><strong className="text-white">Network isolation</strong> &mdash; configurable network policies that control which hosts and ports the agent can reach.</li>
          <li><strong className="text-white">Credential isolation</strong> &mdash; credentials from the host are not available inside the workspace unless explicitly injected.</li>
        </ul>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Creating a Workspace</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Workspaces are created with the <code className="text-accent">onus workspace create</code> command, which accepts parameters for the base image,
          resource limits, and network policy:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
# Create a workspace with Ubuntu and resource limits
onus workspace create --image ubuntu:22.04 --memory 2g --cpu 2

# Create a workspace with network restrictions
onus workspace create --image python:3.11 --block-net --allow-host pypi.org

# Create a workspace with a host directory mount
onus workspace create --image alpine:latest --mount /projects/myapp:/work</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Resource Limits</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Workspaces enforce configurable resource limits to prevent resource exhaustion:
        </p>
        <ul className="list-disc pl-6 text-zinc-300 space-y-2 mb-6">
          <li><strong className="text-white">CPU</strong> &mdash; limit CPU cores and scheduling priority.</li>
          <li><strong className="text-white">Memory</strong> &mdash; set hard and soft memory limits (e.g., <code className="text-accent">--memory 2g</code>).</li>
          <li><strong className="text-white">Disk</strong> &mdash; limit writable overlay size to prevent disk space exhaustion on the host.</li>
          <li><strong className="text-white">Network</strong> &mdash; allow or block specific hosts, ports, and protocols.</li>
        </ul>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Filesystem Isolation</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Workspace filesystems use a read-only base image with a writable overlay layer. The agent
          can read and write within the overlay, but the base image remains pristine. When the workspace
          is destroyed, the overlay is discarded unless explicitly committed. Mounted host directories
          provide controlled access to specific paths on the host system.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Network Policies</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          Network policies define which external hosts the workspace can communicate with. By default,
          workspaces may have network access restricted or blocked entirely. Policies can be set at
          creation time and allow granular control:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
# Block all outbound traffic
onus workspace create --image alpine:latest --block-net

# Allow only specific hosts
onus workspace create --image python:3.11 --allow-host pypi.org --allow-host api.github.com</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Destroying a Workspace</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          When the governed task completes, destroy the workspace to release resources:
        </p>
        <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 font-mono overflow-x-auto my-4">
# List active workspaces
onus workspace list

# Destroy a specific workspace
onus workspace destroy &lt;workspace-id&gt;</pre>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">L3 Claim</h2>
        <p className="text-zinc-300 leading-relaxed mb-4">
          L3 enforcement claims require real container technology (Docker or Podman on Linux, WSL on
          Windows). Onus does not simulate containment. If the required container runtime is not available,
          L3 features are disabled and Onus reports the limitation. The L3 claim is only valid when
          the underlying containment technology is actively enforcing the isolation boundaries.
        </p>

        <h2 className="text-xl font-semibold text-white mt-10 mb-3">Listing Workspaces</h2>
        <p className="text-zinc-300 leading-relaxed mb-8">
          Active workspaces can be listed to show their status, resource usage, and creation time.
          The <code className="text-accent">onus workspace list</code> command displays all workspaces with their IDs, images, and
          current state (running, stopped, or error).
        </p>
      </main>
    </div>
  );
}
