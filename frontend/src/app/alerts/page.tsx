"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { StatusBadge } from "@/components/StatusBadge";
import {
  BellRing,
  Play,
  Plus,
  Send,
  Trash2,
  AlertTriangle,
  CheckCircle2,
} from "lucide-react";

export default function AlertsPage() {
  const [rules, setRules] = useState<any[]>([]);
  const [events, setEvents] = useState<any[]>([]);
  const [notice, setNotice] = useState<string | null>(null);
  const [isCreateOpen, setIsCreateOpen] = useState(false);

  const [newRule, setNewRule] = useState({
    name: "High Error Rate Webhook Alert",
    metric_type: "error_rate_pct",
    operator: "gt",
    threshold: 5.0,
    severity: "High",
    channel_type: "slack",
    webhook_url: "",
  });

  const loadData = async () => {
    try {
      const [r, ev] = await Promise.all([
        fetchApi<any[]>("/alerts/rules"),
        fetchApi<any[]>("/alerts/events"),
      ]);
      setRules(r);
      setEvents(ev);
    } catch (err) {
      console.error(err);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleCreateRule = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await fetchApi("/alerts/rules", {
        method: "POST",
        body: JSON.stringify(newRule),
      });
      setIsCreateOpen(false);
      setNotice("Created new alerting rule.");
      await loadData();
    } catch (err: any) {
      alert(`Failed to create alert rule: ${err.message}`);
    }
  };

  const handleToggleRule = async (rule: any) => {
    const nextEnabled = !(rule.is_enabled === 1 || rule.is_enabled === true);
    try {
      await fetchApi(`/alerts/rules/${rule.id}`, {
        method: "PATCH",
        body: JSON.stringify({ is_enabled: nextEnabled }),
      });
      await loadData();
    } catch (err: any) {
      alert(`Failed to toggle rule: ${err.message}`);
    }
  };

  const handleDeleteRule = async (id: string) => {
    try {
      await fetchApi(`/alerts/rules/${id}`, { method: "DELETE" });
      await loadData();
    } catch (err: any) {
      alert(`Failed to delete rule: ${err.message}`);
    }
  };

  const handleTestWebhook = async (id: string) => {
    try {
      const res = await fetchApi<any>(`/alerts/rules/${id}/test`, {
        method: "POST",
      });
      setNotice(`Webhook probe (${res.channel_type}): ${res.delivery_status}`);
    } catch (err: any) {
      setNotice(`Webhook probe failed: ${err.message}`);
    }
  };

  const handleEvaluateNow = async () => {
    try {
      const triggered = await fetchApi<any[]>("/alerts/evaluate", {
        method: "POST",
      });
      setNotice(
        triggered.length > 0
          ? `Evaluation complete: ${triggered.length} alert rule(s) breached and recorded.`
          : "Evaluation complete: All active rules within nominal thresholds."
      );
      await loadData();
    } catch (err: any) {
      setNotice(`Alert evaluation failed: ${err.message}`);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <BellRing className="w-5 h-5 text-indigo-400" />
            Alerting Rules &amp; Webhook Notifier
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Automated SLA thresholds for Hallucination Index, p95 latency, error rate, and monthly spend limits with Slack/Discord webhooks.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleEvaluateNow}
            className="inline-flex items-center gap-1.5 px-3 py-2 bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 rounded-lg text-xs font-semibold transition"
          >
            <Play className="w-3.5 h-3.5 text-emerald-400 fill-current" />
            Evaluate Rules Now
          </button>
          <button
            onClick={() => setIsCreateOpen(true)}
            className="inline-flex items-center gap-1.5 px-3.5 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold transition"
          >
            <Plus className="w-4 h-4" />
            New Alert Rule
          </button>
        </div>
      </div>

      {notice && (
        <div className="rounded-xl border border-indigo-800/60 bg-indigo-950/30 p-3 text-xs text-indigo-200 flex items-center justify-between">
          <span className="flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 text-indigo-400" />
            {notice}
          </span>
          <button onClick={() => setNotice(null)} className="text-slate-400 hover:text-white">
            Dismiss
          </button>
        </div>
      )}

      {/* Alert Rules Table */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/40 overflow-hidden">
        <div className="p-4 border-b border-slate-800 flex items-center justify-between">
          <h3 className="text-sm font-semibold text-white">Configured SLA &amp; Guardrail Rules</h3>
          <span className="text-xs text-slate-400">{rules.length} active rule definitions</span>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-slate-950/60 text-slate-400 uppercase font-semibold border-b border-slate-800">
              <tr>
                <th className="py-3 px-4">Rule Name</th>
                <th className="py-3 px-4">Metric Condition</th>
                <th className="py-3 px-4">Severity</th>
                <th className="py-3 px-4">Channel</th>
                <th className="py-3 px-4">Enabled</th>
                <th className="py-3 px-4 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              {rules.map((r) => {
                const enabled = r.is_enabled === 1 || r.is_enabled === true;
                return (
                  <tr key={r.id} className="hover:bg-slate-800/20">
                    <td className="py-3 px-4 font-semibold text-white">
                      <div>{r.name}</div>
                      <div className="text-[10px] font-mono text-slate-500">{r.id}</div>
                    </td>
                    <td className="py-3 px-4 font-mono text-cyan-300">
                      {r.metric_type} {r.operator} {r.threshold}
                    </td>
                    <td className="py-3 px-4">
                      <StatusBadge status={r.severity} />
                    </td>
                    <td className="py-3 px-4 font-mono uppercase text-slate-400">
                      {r.channel_type}
                    </td>
                    <td className="py-3 px-4">
                      <button
                        onClick={() => handleToggleRule(r)}
                        className={`px-2 py-0.5 rounded text-[10px] font-semibold border ${
                          enabled
                            ? "bg-emerald-950/60 text-emerald-400 border-emerald-800/60"
                            : "bg-slate-900 text-slate-500 border-slate-800"
                        }`}
                      >
                        {enabled ? "Active" : "Paused"}
                      </button>
                    </td>
                    <td className="py-3 px-4 text-right space-x-1.5">
                      <button
                        onClick={() => handleTestWebhook(r.id)}
                        className="inline-flex items-center gap-1 px-2 py-1 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded text-[11px]"
                      >
                        <Send className="w-3 h-3" />
                        Test Webhook
                      </button>
                      <button
                        onClick={() => handleDeleteRule(r.id)}
                        className="inline-flex items-center p-1.5 bg-slate-900 hover:bg-rose-950/60 text-slate-400 hover:text-rose-400 border border-slate-800 rounded"
                        title="Delete rule"
                      >
                        <Trash2 className="w-3 h-3" />
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* Triggered Alert Events Log */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/40 overflow-hidden">
        <div className="p-4 border-b border-slate-800 flex items-center justify-between">
          <h3 className="text-sm font-semibold text-white flex items-center gap-2">
            <AlertTriangle className="w-4 h-4 text-amber-400" />
            Alert Trigger &amp; Webhook Delivery Log
          </h3>
          <span className="text-xs text-slate-400">{events.length} recorded events</span>
        </div>
        {events.length === 0 ? (
          <div className="p-6 text-xs text-slate-400 text-center">
            No SLA breaches recorded yet. Click &ldquo;Evaluate Rules Now&rdquo; above to run an immediate threshold check.
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs">
              <thead className="bg-slate-950/60 text-slate-400 uppercase font-semibold border-b border-slate-800">
                <tr>
                  <th className="py-3 px-4">Rule</th>
                  <th className="py-3 px-4">Observed vs Threshold</th>
                  <th className="py-3 px-4">Severity</th>
                  <th className="py-3 px-4">Delivery Status</th>
                  <th className="py-3 px-4">Message</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60 text-slate-300">
                {events.map((ev) => (
                  <tr key={ev.id}>
                    <td className="py-3 px-4 font-semibold text-white">{ev.rule_name}</td>
                    <td className="py-3 px-4 font-mono text-amber-300">
                      {Number(ev.observed_value).toFixed(2)} / {ev.threshold}
                    </td>
                    <td className="py-3 px-4">
                      <StatusBadge status={ev.severity} />
                    </td>
                    <td className="py-3 px-4 font-mono text-[11px] text-emerald-300">
                      {ev.delivery_status}
                    </td>
                    <td className="py-3 px-4 text-slate-400 max-w-md truncate">{ev.message}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Create Alert Rule Modal */}
      {isCreateOpen && (
        <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl max-w-md w-full p-6 space-y-4 shadow-2xl">
            <h3 className="text-base font-bold text-white flex items-center gap-2">
              <BellRing className="w-4 h-4 text-indigo-400" />
              Create Alerting Rule
            </h3>
            <form onSubmit={handleCreateRule} className="space-y-3 text-xs">
              <div>
                <label className="block text-slate-400 mb-1">Rule Name</label>
                <input
                  required
                  value={newRule.name}
                  onChange={(e) => setNewRule({ ...newRule, name: e.target.value })}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white"
                />
              </div>

              <div className="grid grid-cols-3 gap-2">
                <div>
                  <label className="block text-slate-400 mb-1">Metric</label>
                  <select
                    value={newRule.metric_type}
                    onChange={(e) => setNewRule({ ...newRule, metric_type: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white"
                  >
                    <option value="hallucination_score">hallucination_score</option>
                    <option value="latency_p95_ms">latency_p95_ms</option>
                    <option value="budget_used_pct">budget_used_pct</option>
                    <option value="error_rate_pct">error_rate_pct</option>
                  </select>
                </div>
                <div>
                  <label className="block text-slate-400 mb-1">Operator</label>
                  <select
                    value={newRule.operator}
                    onChange={(e) => setNewRule({ ...newRule, operator: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white"
                  >
                    <option value="gt">&gt; (gt)</option>
                    <option value="gte">&gt;= (gte)</option>
                    <option value="lt">&lt; (lt)</option>
                    <option value="lte">&lt;= (lte)</option>
                  </select>
                </div>
                <div>
                  <label className="block text-slate-400 mb-1">Threshold</label>
                  <input
                    type="number"
                    step="0.01"
                    required
                    value={newRule.threshold}
                    onChange={(e) =>
                      setNewRule({ ...newRule, threshold: parseFloat(e.target.value) || 0 })
                    }
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white font-mono"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-slate-400 mb-1">Severity</label>
                  <select
                    value={newRule.severity}
                    onChange={(e) => setNewRule({ ...newRule, severity: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white"
                  >
                    <option value="Critical">Critical</option>
                    <option value="High">High</option>
                    <option value="Medium">Medium</option>
                    <option value="Low">Low</option>
                  </select>
                </div>
                <div>
                  <label className="block text-slate-400 mb-1">Webhook Channel</label>
                  <select
                    value={newRule.channel_type}
                    onChange={(e) => setNewRule({ ...newRule, channel_type: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white"
                  >
                    <option value="slack">Slack</option>
                    <option value="discord">Discord</option>
                    <option value="webhook">Generic JSON Webhook</option>
                  </select>
                </div>
              </div>

              <div>
                <label className="block text-slate-400 mb-1">Webhook URL (Optional)</label>
                <input
                  placeholder="https://hooks.slack.com/services/..."
                  value={newRule.webhook_url}
                  onChange={(e) => setNewRule({ ...newRule, webhook_url: e.target.value })}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white font-mono"
                />
              </div>

              <div className="flex justify-end gap-2 pt-3 border-t border-slate-800">
                <button
                  type="button"
                  onClick={() => setIsCreateOpen(false)}
                  className="px-3 py-1.5 bg-slate-800 text-slate-300 rounded-lg text-xs"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-3.5 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold"
                >
                  Save Alert Rule
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
