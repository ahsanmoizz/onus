"use client";

import { useEffect, useState } from "react";

interface Approval {
  action_id: string;
  tool_name: string;
  summary: string;
  status: string;
  created_at: string;
}

interface AuditEntry {
  id: string;
  action: string;
  status: string;
  timestamp: string;
}

export default function DashboardPage() {
  const [approvals, setApprovals] = useState<Approval[]>([]);
  const [auditLog, setAuditLog] = useState<AuditEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchData() {
      try {
        const [approvalsRes, auditRes] = await Promise.all([
          fetch("http://127.0.0.1:9191/api/approvals").catch(() => null),
          fetch("http://127.0.0.1:9292/api/log?limit=20").catch(() => null),
        ]);
        if (approvalsRes?.ok) {
          const data = await approvalsRes.json();
          setApprovals(data.entries ?? data ?? []);
        }
        if (auditRes?.ok) {
          const data = await auditRes.json();
          setAuditLog(data.entries ?? data ?? []);
        }
      } catch {
        // Servers not reachable — show offline state
      } finally {
        setLoading(false);
      }
    }
    fetchData();
  }, []);

  const pendingCount = approvals.filter((a) => a.status === "pending").length;

  return (
    <div className="p-6 max-w-6xl mx-auto space-y-8">
      <div>
        <h1 className="text-2xl font-bold">Dashboard</h1>
        <p className="text-zinc-500 text-sm mt-1">Onus AI Agent Firewall — real-time overview</p>
      </div>

      {/* Status cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
          <div className="text-zinc-500 text-xs uppercase tracking-wide">Pending Approvals</div>
          <div className="text-3xl font-bold mt-1">{loading ? "—" : pendingCount}</div>
        </div>
        <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
          <div className="text-zinc-500 text-xs uppercase tracking-wide">Approval Server</div>
          <div className={`text-sm font-medium mt-1 ${approvals.length > 0 || loading ? "text-green-400" : "text-zinc-600"}`}>
            {loading ? "Checking..." : approvals.length > 0 || pendingCount > 0 ? "Online" : "Unreachable"}
          </div>
        </div>
        <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
          <div className="text-zinc-500 text-xs uppercase tracking-wide">Audit Server</div>
          <div className={`text-sm font-medium mt-1 ${auditLog.length > 0 || loading ? "text-green-400" : "text-zinc-600"}`}>
            {loading ? "Checking..." : auditLog.length > 0 ? "Online" : "Unreachable"}
          </div>
        </div>
      </div>

      {/* Recent approvals */}
      <section>
        <h2 className="text-lg font-semibold mb-3">Pending Approvals</h2>
        {loading ? (
          <div className="text-zinc-600 text-sm">Loading...</div>
        ) : approvals.length === 0 ? (
          <div className="text-zinc-600 text-sm border border-zinc-800 rounded-xl p-6 text-center">
            No approvals to display. Connect to the Onus approval server at port 9191.
          </div>
        ) : (
          <div className="space-y-2">
            {approvals.slice(0, 10).map((a) => (
              <div key={a.action_id} className="flex items-center justify-between border border-zinc-800 rounded-lg px-4 py-3 bg-zinc-900/50">
                <div>
                  <div className="text-sm font-medium">{a.summary || a.tool_name || a.action_id}</div>
                  <div className="text-xs text-zinc-500 mt-0.5">{a.tool_name} — {a.created_at}</div>
                </div>
                <span className={`text-xs px-2 py-1 rounded-full ${
                  a.status === "pending" ? "bg-yellow-900/50 text-yellow-400" : "bg-zinc-800 text-zinc-500"
                }`}>{a.status}</span>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Recent audit entries */}
      <section>
        <h2 className="text-lg font-semibold mb-3">Recent Audit Trail</h2>
        {loading ? (
          <div className="text-zinc-600 text-sm">Loading...</div>
        ) : auditLog.length === 0 ? (
          <div className="text-zinc-600 text-sm border border-zinc-800 rounded-xl p-6 text-center">
            No audit entries. Connect to the Onus audit server at port 9292.
          </div>
        ) : (
          <div className="space-y-2">
            {auditLog.slice(0, 10).map((e) => (
              <div key={e.id} className="flex items-center justify-between border border-zinc-800 rounded-lg px-4 py-3 bg-zinc-900/50">
                <div>
                  <div className="text-sm font-medium">{e.action}</div>
                  <div className="text-xs text-zinc-500 mt-0.5">{e.timestamp}</div>
                </div>
                <span className={`text-xs px-2 py-1 rounded-full ${
                  e.status === "allow" ? "bg-green-900/50 text-green-400" :
                  e.status === "deny" ? "bg-red-900/50 text-red-400" :
                  "bg-zinc-800 text-zinc-500"
                }`}>{e.status}</span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
