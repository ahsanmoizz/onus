"use client";

import { useEffect, useState } from "react";

interface Approval {
  action_id: string;
  tool_name: string;
  summary: string;
  payload_hash: string;
  status: string;
  created_at: string;
  expires_at: number;
}

export default function ApprovalsPage() {
  const [approvals, setApprovals] = useState<Approval[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function fetchApprovals() {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("http://127.0.0.1:9191/api/approvals");
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setApprovals(data.entries ?? data ?? []);
    } catch (e) {
      setError("Could not reach the approval server (127.0.0.1:9191). Is the daemon running?");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => { fetchApprovals(); }, []);

  async function handleAction(actionId: string, action: "approve" | "deny") {
    try {
      const res = await fetch(`http://127.0.0.1:9191/api/approvals/${actionId}/${action}`, {
        method: "POST",
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      fetchApprovals();
    } catch {
      setError(`Failed to ${action} action ${actionId.slice(0, 8)}...`);
    }
  }

  const pending = approvals.filter((a) => a.status === "pending");
  const history = approvals.filter((a) => a.status !== "pending");

  return (
    <div className="p-6 max-w-6xl mx-auto space-y-8">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Approvals</h1>
          <p className="text-zinc-500 text-sm mt-1">Manage pending agent action approvals</p>
        </div>
        <button
          onClick={fetchApprovals}
          className="text-sm px-3 py-1.5 rounded-lg border border-zinc-700 bg-zinc-900 hover:bg-zinc-800 transition-colors"
        >Refresh</button>
      </div>

      {error && (
        <div className="border border-red-900/50 bg-red-950/30 rounded-xl p-4 text-sm text-red-400">{error}</div>
      )}

      {loading ? (
        <div className="text-zinc-600 text-sm">Loading approvals...</div>
      ) : pending.length === 0 ? (
        <div className="border border-zinc-800 rounded-xl p-8 text-center text-zinc-600">
          No pending approvals. All clear.
        </div>
      ) : (
        <div className="space-y-3">
          {pending.map((a) => (
            <div key={a.action_id} className="border border-zinc-800 rounded-xl p-4 bg-zinc-900/50">
              <div className="flex items-start justify-between mb-3">
                <div>
                  <div className="font-medium">{a.summary || a.tool_name || "Action"}</div>
                  <div className="text-xs text-zinc-500 mt-0.5 font-mono">{a.action_id}</div>
                </div>
                <span className="text-xs px-2 py-1 rounded-full bg-yellow-900/50 text-yellow-400">Pending</span>
              </div>
              <div className="text-xs text-zinc-500 space-y-1 mb-3">
                <div>Tool: {a.tool_name}</div>
                <div>Hash: <span className="font-mono">{a.payload_hash?.slice(0, 16)}...</span></div>
                <div>Expires: {a.expires_at ? new Date(a.expires_at * 1000).toLocaleString() : "N/A"}</div>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() => handleAction(a.action_id, "approve")}
                  className="text-sm px-4 py-1.5 rounded-lg bg-green-700 hover:bg-green-600 transition-colors font-medium"
                >Approve</button>
                <button
                  onClick={() => handleAction(a.action_id, "deny")}
                  className="text-sm px-4 py-1.5 rounded-lg bg-red-800 hover:bg-red-700 transition-colors font-medium"
                >Deny</button>
              </div>
            </div>
          ))}
        </div>
      )}

      {history.length > 0 && (
        <section>
          <h2 className="text-lg font-semibold mb-3">History ({history.length})</h2>
          <div className="space-y-2">
            {history.map((a) => (
              <div key={a.action_id} className="flex items-center justify-between border border-zinc-800 rounded-lg px-4 py-3 bg-zinc-900/30">
                <div>
                  <div className="text-sm">{a.summary || a.tool_name || a.action_id}</div>
                  <div className="text-xs text-zinc-600 mt-0.5 font-mono">{a.action_id?.slice(0, 12)}...</div>
                </div>
                <span className={`text-xs px-2 py-1 rounded-full ${
                  a.status === "approved" || a.status === "allow" ? "bg-green-900/50 text-green-400" :
                  a.status === "denied" || a.status === "deny" || a.status === "rejected" ? "bg-red-900/50 text-red-400" :
                  "bg-zinc-800 text-zinc-500"
                }`}>{a.status}</span>
              </div>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
