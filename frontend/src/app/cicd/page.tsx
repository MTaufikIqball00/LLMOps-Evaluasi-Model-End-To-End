"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { StatusBadge } from "@/components/StatusBadge";
import { GitBranch, Play, ExternalLink, Clock, GitCommit, CheckCircle2 } from "lucide-react";

export default function CicdPage() {
  const [workflows, setWorkflows] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [triggerForm, setTriggerForm] = useState({
    workflow_name: "Model Evaluation & Benchmark Gates",
    branch: "main",
  });

  const loadWorkflows = async () => {
    try {
      setLoading(true);
      const data = await fetchApi<any[]>("/cicd/workflows");
      setWorkflows(data);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadWorkflows();
  }, []);

  const handleTrigger = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await fetchApi("/cicd/workflows/trigger", {
        method: "POST",
        body: JSON.stringify(triggerForm),
      });
      setIsModalOpen(false);
      loadWorkflows();
    } catch (err: any) {
      alert(`Trigger failed: ${err.message}`);
    }
  };

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <GitBranch className="w-5 h-5 text-indigo-400" />
            CI/CD Pipeline Automation
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            GitHub Actions workflows: automated benchmark regression gates, container builds, and staging canary rollouts.
          </p>
        </div>
        <button
          onClick={() => setIsModalOpen(true)}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold shadow-sm transition"
        >
          <Play className="w-3.5 h-3.5 fill-current" />
          Dispatch Workflow
        </button>
      </div>

      {/* Workflows Table */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/40 overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-slate-950/60 text-slate-400 uppercase font-semibold border-b border-slate-800">
              <tr>
                <th className="py-3 px-4">Workflow Name</th>
                <th className="py-3 px-4">Branch / Commit</th>
                <th className="py-3 px-4">Status</th>
                <th className="py-3 px-4">Trigger</th>
                <th className="py-3 px-4">Duration</th>
                <th className="py-3 px-4">Started</th>
                <th className="py-3 px-4 text-right">Run Link</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              {loading ? (
                <tr>
                  <td colSpan={7} className="py-8 text-center text-slate-500">
                    Loading CI/CD workflow pipeline runs...
                  </td>
                </tr>
              ) : (
                workflows.map((wf) => (
                  <tr key={wf.id} className="hover:bg-slate-800/30 transition">
                    <td className="py-3.5 px-4">
                      <div className="font-semibold text-white">{wf.workflow_name}</div>
                      <div className="text-[11px] text-slate-400">{wf.commit_message}</div>
                    </td>
                    <td className="py-3.5 px-4 font-mono">
                      <div className="text-indigo-400 flex items-center gap-1">
                        <GitBranch className="w-3 h-3 text-slate-500" />
                        {wf.branch}
                      </div>
                      <div className="text-[10px] text-slate-500 flex items-center gap-1 mt-0.5">
                        <GitCommit className="w-3 h-3" />
                        {wf.commit_sha}
                      </div>
                    </td>
                    <td className="py-3.5 px-4">
                      <StatusBadge status={wf.status} />
                    </td>
                    <td className="py-3.5 px-4 font-mono text-slate-400">
                      {wf.trigger_event}
                    </td>
                    <td className="py-3.5 px-4 text-slate-400">
                      <span className="flex items-center gap-1">
                        <Clock className="w-3 h-3 text-slate-500" />
                        {Math.floor(wf.duration_seconds / 60)}m {wf.duration_seconds % 60}s
                      </span>
                    </td>
                    <td className="py-3.5 px-4 text-slate-400 text-[11px]">
                      {new Date(wf.started_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                    </td>
                    <td className="py-3.5 px-4 text-right">
                      <a
                        href={wf.url || "https://github.com"}
                        target="_blank"
                        rel="noreferrer"
                        className="inline-flex items-center gap-1 text-xs text-indigo-400 hover:text-indigo-300 font-medium"
                      >
                        GitHub
                        <ExternalLink className="w-3 h-3" />
                      </a>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Trigger Modal */}
      {isModalOpen && (
        <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl max-w-md w-full p-6 space-y-4 shadow-2xl">
            <h3 className="text-base font-bold text-white flex items-center gap-2">
              <Play className="w-4 h-4 text-indigo-400 fill-current" />
              Dispatch GitHub Actions Workflow
            </h3>
            <form onSubmit={handleTrigger} className="space-y-3 text-xs">
              <div>
                <label className="block text-slate-400 mb-1">Workflow Target</label>
                <select
                  value={triggerForm.workflow_name}
                  onChange={(e) => setTriggerForm({ ...triggerForm, workflow_name: e.target.value })}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="Model Evaluation & Benchmark Gates">Model Evaluation & Benchmark Gates</option>
                  <option value="vLLM Serving Container Build & Security Scan">vLLM Serving Container Build & Security Scan</option>
                  <option value="Canary Rollout to Staging Kubernetes">Canary Rollout to Staging Kubernetes</option>
                </select>
              </div>
              <div>
                <label className="block text-slate-400 mb-1">Target Git Branch</label>
                <input
                  value={triggerForm.branch}
                  onChange={(e) => setTriggerForm({ ...triggerForm, branch: e.target.value })}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white font-mono focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div className="flex justify-end gap-2 pt-3 border-t border-slate-800">
                <button
                  type="button"
                  onClick={() => setIsModalOpen(false)}
                  className="px-3 py-1.5 bg-slate-800 text-slate-300 rounded-lg text-xs"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold"
                >
                  Dispatch Run
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
