"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { StatusBadge } from "@/components/StatusBadge";
import { Server, RotateCcw, Cpu, Layers, HardDrive } from "lucide-react";

export default function DeploymentsPage() {
  const [deployments, setDeployments] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [restartingId, setRestartingId] = useState<string | null>(null);

  const loadDeployments = async () => {
    try {
      setLoading(true);
      const data = await fetchApi<any[]>("/deployments");
      setDeployments(data);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadDeployments();
  }, []);

  const handleRestart = async (id: string) => {
    try {
      setRestartingId(id);
      await fetchApi(`/deployments/${id}/restart`, { method: "POST" });
      await loadDeployments();
    } catch (err: any) {
      alert(`Restart failed: ${err.message}`);
    } finally {
      setRestartingId(null);
    }
  };

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <Server className="w-5 h-5 text-indigo-400" />
            Kubernetes Serving Deployments
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Manage vLLM and TGI inference workloads, replica auto-scaling, rollout health, and hardware utilization.
          </p>
        </div>
        <StatusBadge status="Kubernetes Cluster Active" size="md" />
      </div>

      {/* Deployments Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {loading ? (
          <div className="col-span-3 text-center py-12 text-xs text-slate-500">
            Loading Kubernetes serving deployments...
          </div>
        ) : (
          deployments.map((dep) => (
            <div
              key={dep.id}
              className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 space-y-4 hover:border-slate-700 transition"
            >
              <div className="flex items-start justify-between gap-2">
                <div>
                  <h3 className="font-semibold text-white text-sm">{dep.name}</h3>
                  <div className="text-[11px] font-mono text-indigo-400 mt-0.5">
                    {dep.model_name} ({dep.version})
                  </div>
                </div>
                <StatusBadge status={dep.status} />
              </div>

              <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800 text-xs space-y-2">
                <div className="flex justify-between items-center text-slate-400">
                  <span>Environment:</span>
                  <span className="font-medium text-white">{dep.environment}</span>
                </div>
                <div className="flex justify-between items-center text-slate-400">
                  <span>Target Cluster:</span>
                  <span className="font-mono text-slate-300">{dep.cluster}</span>
                </div>
                <div className="flex justify-between items-center text-slate-400">
                  <span>Pods Ready / Desired:</span>
                  <span className="font-mono text-emerald-400 font-bold">
                    {dep.replicas_ready} / {dep.replicas_desired}
                  </span>
                </div>
                <div className="flex justify-between items-center text-slate-400">
                  <span>CPU Usage:</span>
                  <span className="font-mono text-cyan-300">{dep.cpu_usage || "N/A"}</span>
                </div>
                <div className="flex justify-between items-center text-slate-400">
                  <span>VRAM / Memory:</span>
                  <span className="font-mono text-amber-300">{dep.memory_usage || "N/A"}</span>
                </div>
              </div>

              <div>
                <span className="text-[10px] text-slate-500 uppercase block mb-1">
                  Container Image Tag
                </span>
                <span
                  className="font-mono text-[11px] text-slate-400 bg-slate-950 px-2 py-1 rounded border border-slate-850 block truncate"
                  title={dep.image_tag}
                >
                  {dep.image_tag}
                </span>
              </div>

              <div className="pt-3 border-t border-slate-800 flex justify-between items-center">
                <span className="text-[10px] text-slate-500">
                  Updated: {new Date(dep.updated_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                </span>
                <button
                  onClick={() => handleRestart(dep.id)}
                  disabled={restartingId === dep.id}
                  className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-white text-xs font-semibold transition"
                >
                  <RotateCcw className={`w-3 h-3 ${restartingId === dep.id ? "animate-spin text-indigo-400" : ""}`} />
                  Rollout Restart
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
