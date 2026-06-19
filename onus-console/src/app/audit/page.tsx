"use client";

import { useEffect, useState } from "react";

interface AuditEntry {
  id: string;
  action: string;
  tool: string;
  status: string;
  verdict: string;
  timestamp: string;
  hash?: string;
}

export default function AuditPage() {
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchAudit() {
      try {
        const res = await fetch("http://127.0.0.1:9292/api/log?limit=100");
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const data = await res.json();
        setEntries(data.entries ?? data ?? []);
      } catch {
        setError("Could not reach the audit server (127.0.0.1:9292).");
      } finally {
        setLoading(false);
      }
    }
    fetchAudit();
  }, []);

  return (
    <div className="p-6 max-w-6xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Audit Trail</h1>
        <p className="text-zinc-500 text-sm mt-1">Hash-chained action log</p>
      </div>

      {error && (
        <div className="border border-red-900/50 bg-red-950/30 rounded-xl p-4 text-sm text-red-400">{error}</div>
      )}

      {loading ? (
        <div className="text-zinc-600 text-sm">Loading audit trail...</div>
      ) : entries.length === 0 ? (
        <div className="border border-zinc-800 rounded-xl p-8 text-center text-zinc-600">No audit entries found.</div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-zinc-800 text-zinc-500 text-left">
                <th className="pb-2 pr-4">Timestamp</th>
                <th className="pb-2 pr-4">Action</th>
                <th className="pb-2 pr-4">Tool</th>
                <th className="pb-2 pr-4">Verdict</th>
                <th className="pb-2 pr-4">Status</th>
                <th className="pb-2">Hash</th>
              </tr>
            </thead>
            <tbody>
              {entries.map((e) => (
                <tr key={e.id} className="border-b border-zinc-800/50 hover:bg-zinc-900/30">
                  <td className="py-2 pr-4 text-zinc-400 whitespace-nowrap font-mono text-xs">{e.timestamp}</td>
                  <td className="py-2 pr-4 font-medium">{e.action}</td>
                  <td className="py-2 pr-4 text-zinc-400">{e.tool || "—"}</td>
                  <td className="py-2 pr-4">
                    <span className={`text-xs px-2 py-0.5 rounded-full ${
                      e.verdict === "allow" ? "bg-green-900/50 text-green-400" :
                      e.verdict === "deny" ? "bg-red-900/50 text-red-400" :
                      "bg-zinc-800 text-zinc-500"
                    }`}>{e.verdict}</span>
                  </td>
                  <td className="py-2 pr-4 text-zinc-400">{e.status}</td>
                  <td className="py-2 text-zinc-600 font-mono text-xs">{e.hash ? e.hash.slice(0, 12) + "..." : "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
