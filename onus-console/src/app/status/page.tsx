"use client";

import { useEffect, useState } from "react";

interface HealthStatus {
  daemon: string;
  policy_engine: string;
  claude_hook: string;
  audit_trail: string;
  approval_server: string;
  version: string;
}

export default function StatusPage() {
  const [status, setStatus] = useState<HealthStatus | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchStatus() {
      try {
        const res = await fetch("http://127.0.0.1:9292/api/status").catch(() => null);
        if (res?.ok) {
          setStatus(await res.json());
        } else {
          // Try daemon status endpoint
          const res2 = await fetch("http://127.0.0.1:9191/api/status").catch(() => null);
          if (res2?.ok) setStatus(await res2.json());
        }
      } catch {
        // Offline
      } finally {
        setLoading(false);
      }
    }
    fetchStatus();
  }, []);

  const checks: [string, string | undefined][] = [
    ["Daemon", status?.daemon],
    ["Policy Engine", status?.policy_engine],
    ["Claude Hook", status?.claude_hook],
    ["Audit Trail", status?.audit_trail],
    ["Approval Server", status?.approval_server],
  ];

  return (
    <div className="p-6 max-w-4xl mx-auto space-y-8">
      <div>
        <h1 className="text-2xl font-bold">System Status</h1>
        <p className="text-zinc-500 text-sm mt-1">Onus component health</p>
        {status?.version && (
          <p className="text-xs text-zinc-600 mt-1 font-mono">Version: {status.version}</p>
        )}
      </div>

      {loading ? (
        <div className="text-zinc-600 text-sm">Checking system status...</div>
      ) : !status ? (
        <div className="border border-zinc-800 rounded-xl p-8 text-center text-zinc-600">
          Could not reach any Onus servers. Ensure the daemon is running.
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {checks.map(([name, value]) => (
            <div key={name} className="border border-zinc-800 rounded-xl p-4 bg-zinc-900/50 flex items-center justify-between">
              <span className="font-medium">{name}</span>
              <span className={`text-xs px-2 py-1 rounded-full ${
                value == "ok" || value == "healthy" ? "bg-green-900/50 text-green-400" :
                value ? "bg-yellow-900/50 text-yellow-400" :
                "bg-zinc-800 text-zinc-600"
              }`}>{value ?? "Unknown"}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
