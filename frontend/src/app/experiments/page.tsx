"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { StatusBadge } from "@/components/StatusBadge";
import { FlaskConical, GitCompare, Plus, ArrowUpDown, Clock } from "lucide-react";

export default function ExperimentsPage() {
  const [experiments, setExperiments] = useState<any[]>([]);
  const [comparison, setComparison] = useState<any>(null);
  const [selectedRuns, setSelectedRuns] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);

  const loadExperiments = async () => {
    try {
      setLoading(true);
      const data = await fetchApi<any[]>("/experiments");
      setExperiments(data);
      if (data.length >= 2 && selectedRuns.length === 0) {
        setSelectedRuns([data[0].id, data[1].id]);
      }
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const loadComparison = async (runIds: string[]) => {
    if (runIds.length === 0) return;
    try {
      const data = await fetchApi<any>(`/experiments/compare?runs=${runIds.join(",")}`);
      setComparison(data);
    } catch (err) {
      console.error(err);
    }
  };

  useEffect(() => {
    loadExperiments();
  }, []);

  useEffect(() => {
    if (selectedRuns.length > 0) {
      loadComparison(selectedRuns);
    }
  }, [selectedRuns]);

  const toggleSelectRun = (id: string) => {
    setSelectedRuns((prev) =>
      prev.includes(id) ? prev.filter((r) => r !== id) : [...prev, id]
    );
  };

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <FlaskConical className="w-5 h-5 text-indigo-400" />
            Experiment Tracking & Diffing
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Discover MLflow / W&B runs, compare hyperparameter sets, and analyze loss & perplexity metrics.
          </p>
        </div>
      </div>

      {/* Experiments List Table */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/40 overflow-hidden">
        <div className="p-4 border-b border-slate-800 flex items-center justify-between">
          <div className="text-xs font-semibold text-white">Registered Experiment Runs</div>
          <div className="text-[11px] text-slate-400">
            Selected for comparison: <span className="text-indigo-400 font-semibold">{selectedRuns.length}</span>
          </div>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-slate-950/60 text-slate-400 uppercase font-semibold border-b border-slate-800">
              <tr>
                <th className="py-3 px-4 w-10">Select</th>
                <th className="py-3 px-4">Run Name & ID</th>
                <th className="py-3 px-4">Framework</th>
                <th className="py-3 px-4">Status</th>
                <th className="py-3 px-4">Key Metrics</th>
                <th className="py-3 px-4">Duration</th>
                <th className="py-3 px-4">Created</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              {loading ? (
                <tr>
                  <td colSpan={7} className="py-8 text-center text-slate-500">
                    Loading experiment runs...
                  </td>
                </tr>
              ) : experiments.map((exp) => {
                const isSelected = selectedRuns.includes(exp.id);
                let parsedMetrics: any = {};
                try {
                  parsedMetrics = JSON.parse(exp.metrics);
                } catch {}

                return (
                  <tr
                    key={exp.id}
                    onClick={() => toggleSelectRun(exp.id)}
                    className={`cursor-pointer transition ${
                      isSelected ? "bg-indigo-950/20" : "hover:bg-slate-800/30"
                    }`}
                  >
                    <td className="py-3.5 px-4">
                      <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={() => {}}
                        className="rounded border-slate-700 bg-slate-900 text-indigo-600 focus:ring-0"
                      />
                    </td>
                    <td className="py-3.5 px-4 font-medium text-white">
                      <div>{exp.name}</div>
                      <div className="text-[11px] font-mono text-slate-500">{exp.id}</div>
                    </td>
                    <td className="py-3.5 px-4">
                      <span className="font-mono text-xs text-indigo-300 bg-indigo-950/50 px-2 py-0.5 rounded border border-indigo-900/50">
                        {exp.framework}
                      </span>
                    </td>
                    <td className="py-3.5 px-4">
                      <StatusBadge status={exp.status} />
                    </td>
                    <td className="py-3.5 px-4 font-mono text-[11px] text-slate-300">
                      {Object.entries(parsedMetrics)
                        .slice(0, 2)
                        .map(([k, v]: any) => `${k}: ${v}`)
                        .join(" | ")}
                    </td>
                    <td className="py-3.5 px-4 text-slate-400">
                      <span className="flex items-center gap-1">
                        <Clock className="w-3 h-3 text-slate-500" />
                        {Math.floor(exp.duration_seconds / 60)}m {exp.duration_seconds % 60}s
                      </span>
                    </td>
                    <td className="py-3.5 px-4 text-slate-400 text-[11px]">
                      {new Date(exp.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* Side-by-Side Run Comparison Diff Card */}
      {comparison && comparison.runs.length > 0 && (
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 space-y-4">
          <div className="flex items-center justify-between border-b border-slate-800 pb-3">
            <h3 className="text-sm font-semibold text-white flex items-center gap-2">
              <GitCompare className="w-4 h-4 text-indigo-400" />
              Side-by-Side Run Hyperparameter & Metric Comparison
            </h3>
            <span className="text-xs text-slate-400">
              Comparing {comparison.runs.length} runs
            </span>
          </div>

          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs">
              <thead className="bg-slate-950/60 text-slate-400 uppercase font-semibold border-b border-slate-800">
                <tr>
                  <th className="py-2.5 px-4 w-48">Parameter / Metric</th>
                  {comparison.runs.map((r: any) => (
                    <th key={r.id} className="py-2.5 px-4 font-mono font-medium text-indigo-300">
                      {r.name}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60">
                {/* Hyperparameters Section */}
                <tr className="bg-slate-950/80 font-semibold text-slate-300">
                  <td colSpan={comparison.runs.length + 1} className="py-2 px-4 text-[11px] uppercase tracking-wider text-indigo-400">
                    Hyperparameters
                  </td>
                </tr>
                {comparison.parameter_keys.map((pKey: string) => (
                  <tr key={pKey} className="hover:bg-slate-800/20">
                    <td className="py-2 px-4 font-medium text-slate-400">{pKey}</td>
                    {comparison.runs.map((r: any) => {
                      let val = "-";
                      try {
                        const parsed = JSON.parse(r.parameters);
                        if (parsed[pKey] !== undefined) val = String(parsed[pKey]);
                      } catch {}
                      return (
                        <td key={r.id} className="py-2 px-4 font-mono text-slate-200">
                          {val}
                        </td>
                      );
                    })}
                  </tr>
                ))}

                {/* Validation Metrics Section */}
                <tr className="bg-slate-950/80 font-semibold text-slate-300">
                  <td colSpan={comparison.runs.length + 1} className="py-2 px-4 text-[11px] uppercase tracking-wider text-emerald-400">
                    Validation Metrics
                  </td>
                </tr>
                {comparison.metric_keys.map((mKey: string) => (
                  <tr key={mKey} className="hover:bg-slate-800/20">
                    <td className="py-2 px-4 font-medium text-slate-400">{mKey}</td>
                    {comparison.runs.map((r: any) => {
                      let val = "-";
                      try {
                        const parsed = JSON.parse(r.metrics);
                        if (parsed[mKey] !== undefined) val = String(parsed[mKey]);
                      } catch {}
                      return (
                        <td key={r.id} className="py-2 px-4 font-mono text-emerald-300 font-semibold">
                          {val}
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
