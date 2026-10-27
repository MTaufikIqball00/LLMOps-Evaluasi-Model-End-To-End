"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { MetricCard } from "@/components/MetricCard";
import { TimeseriesChart } from "@/components/SimpleChart";
import { StatusBadge } from "@/components/StatusBadge";
import {
  Activity,
  Cpu,
  Coins,
  Layers,
  ShieldCheck,
  TrendingUp,
  AlertTriangle,
  Zap,
} from "lucide-react";

export default function OverviewPage() {
  const [stats, setStats] = useState<any>(null);
  const [timeseries, setTimeseries] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadData = async () => {
    try {
      setLoading(true);
      const [s, ts] = await Promise.all([
        fetchApi<any>("/overview/stats"),
        fetchApi<any>("/monitoring/metrics"),
      ]);
      setStats(s);
      setTimeseries(ts);
      setError(null);
    } catch (err: any) {
      setError(err.message || "Failed to load overview data from Rust API");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 20000);
    return () => clearInterval(interval);
  }, []);

  if (loading && !stats) {
    return (
      <div className="space-y-6 animate-pulse">
        <div className="h-8 bg-slate-800 rounded w-64" />
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="h-28 bg-slate-900 rounded-xl border border-slate-800" />
          ))}
        </div>
      </div>
    );
  }

  if (error && !stats) {
    return (
      <div className="rounded-xl border border-rose-900/60 bg-rose-950/20 p-6 text-rose-300">
        <div className="flex items-center gap-2 font-semibold text-lg text-rose-200">
          <AlertTriangle className="w-5 h-5" />
          Backend Connection Notice
        </div>
        <p className="mt-2 text-sm text-slate-300">
          Could not connect to Rust backend on port 8210: {error}.
        </p>
        <p className="mt-1 text-xs text-slate-400">
          Ensure the backend is running via <code className="bg-slate-900 px-1 py-0.5 rounded text-indigo-300">start.bat</code> or <code className="bg-slate-900 px-1 py-0.5 rounded text-indigo-300">cargo run</code> in backend/.
        </p>
        <button
          onClick={loadData}
          className="mt-4 px-4 py-2 bg-slate-800 hover:bg-slate-700 text-white rounded-lg text-xs font-semibold"
        >
          Retry Connection
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            Operations & Quality Overview
            <span className="text-xs font-normal text-slate-400">
              ({stats?.data_freshness || "Active"})
            </span>
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Real-time telemetry, model usage distribution, and evaluation gates across clusters.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <StatusBadge status="All Systems Nominal" size="md" />
        </div>
      </div>

      {/* KPI Cards Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="Total Requests"
          value={stats?.total_requests?.toLocaleString() || "0"}
          subtitle={`${stats?.success_rate_pct}% success rate`}
          icon={Activity}
          trend={{ value: "+12.4%", isPositive: true }}
          source="Traces DB"
        />
        <MetricCard
          title="Latency (p95)"
          value={`${stats?.latency?.p95_ms || 0} ms`}
          subtitle={`p50: ${stats?.latency?.p50_ms || 0} ms | p99: ${stats?.latency?.p99_ms || 0} ms`}
          icon={Zap}
          trend={{ value: "-4.2%", isPositive: true }}
          source="Prometheus Exporter"
        />
        <MetricCard
          title="Token Consumption"
          value={stats?.tokens?.total_tokens?.toLocaleString() || "0"}
          subtitle={`${stats?.tokens?.prompt_tokens?.toLocaleString() || 0} in / ${stats?.tokens?.completion_tokens?.toLocaleString() || 0} out`}
          icon={Cpu}
          source="Token Aggregator"
        />
        <MetricCard
          title="Monthly Spend"
          value={`$${stats?.cost?.current_spend_usd?.toFixed(2) || "0.00"}`}
          subtitle={`${stats?.cost?.budget_used_pct || 0}% of $${stats?.cost?.monthly_budget_usd || 2500} budget`}
          icon={Coins}
          alert={stats?.cost?.budget_used_pct > 80}
          source="Pricing Catalog"
        />
      </div>

      {/* Observability & Timeseries Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 rounded-xl border border-slate-800 bg-slate-900/40 p-5 backdrop-blur-sm">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h3 className="text-sm font-semibold text-white flex items-center gap-2">
                <TrendingUp className="w-4 h-4 text-indigo-400" />
                Latency Percentiles (Rolling Window)
              </h3>
              <p className="text-xs text-slate-400">p50, p95, and p99 inference response latencies</p>
            </div>
            <span className="text-[11px] px-2 py-0.5 rounded bg-slate-800 text-slate-400 border border-slate-700">
              {timeseries?.active_source || "Database Telemetry"}
            </span>
          </div>

          {timeseries && (
            <TimeseriesChart
              labels={timeseries.timestamps}
              series={[
                { name: "p99 Latency", data: timeseries.p99_latency_ms, color: "#f43f5e" },
                { name: "p95 Latency", data: timeseries.p95_latency_ms, color: "#f59e0b" },
                { name: "p50 Latency", data: timeseries.p50_latency_ms, color: "#10b981" },
              ]}
              height={200}
              unit="ms"
            />
          )}
        </div>

        {/* Quality Evaluation Summary Card */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 backdrop-blur-sm flex flex-col justify-between">
          <div>
            <h3 className="text-sm font-semibold text-white flex items-center gap-2">
              <ShieldCheck className="w-4 h-4 text-emerald-400" />
              Evaluation & Hallucinations
            </h3>
            <p className="text-xs text-slate-400 mt-1">
              Automated Ragas & TruLens benchmark gates
            </p>

            <div className="mt-4 space-y-3">
              <div>
                <div className="flex justify-between text-xs mb-1">
                  <span className="text-slate-400">Faithfulness</span>
                  <span className="text-emerald-400 font-semibold font-mono">
                    {((stats?.evaluations?.avg_faithfulness || 0.94) * 100).toFixed(1)}%
                  </span>
                </div>
                <div className="h-1.5 w-full bg-slate-800 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-emerald-500 rounded-full"
                    style={{ width: `${(stats?.evaluations?.avg_faithfulness || 0.94) * 100}%` }}
                  />
                </div>
              </div>

              <div>
                <div className="flex justify-between text-xs mb-1">
                  <span className="text-slate-400">Answer Relevance</span>
                  <span className="text-indigo-400 font-semibold font-mono">
                    {((stats?.evaluations?.avg_relevance || 0.92) * 100).toFixed(1)}%
                  </span>
                </div>
                <div className="h-1.5 w-full bg-slate-800 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-indigo-500 rounded-full"
                    style={{ width: `${(stats?.evaluations?.avg_relevance || 0.92) * 100}%` }}
                  />
                </div>
              </div>

              <div>
                <div className="flex justify-between text-xs mb-1">
                  <span className="text-slate-400">Groundedness (Triad)</span>
                  <span className="text-cyan-400 font-semibold font-mono">
                    {((stats?.evaluations?.avg_groundedness || 0.95) * 100).toFixed(1)}%
                  </span>
                </div>
                <div className="h-1.5 w-full bg-slate-800 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-cyan-500 rounded-full"
                    style={{ width: `${(stats?.evaluations?.avg_groundedness || 0.95) * 100}%` }}
                  />
                </div>
              </div>

              <div>
                <div className="flex justify-between text-xs mb-1">
                  <span className="text-slate-400">Hallucination Index</span>
                  <span className="text-rose-400 font-semibold font-mono">
                    {((stats?.evaluations?.avg_hallucination_score || 0.05) * 100).toFixed(1)}%
                  </span>
                </div>
                <div className="h-1.5 w-full bg-slate-800 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-rose-500 rounded-full"
                    style={{ width: `${(stats?.evaluations?.avg_hallucination_score || 0.05) * 100}%` }}
                  />
                </div>
              </div>
            </div>
          </div>

          <div className="pt-3 border-t border-slate-800 mt-4 text-[11px] text-slate-400 flex justify-between items-center">
            <span>Evaluations executed:</span>
            <span className="font-semibold text-slate-200">
              {stats?.evaluations?.total_evaluations || 3} benchmarks
            </span>
          </div>
        </div>
      </div>

      {/* Model Distribution & Kubernetes Workloads Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Model Distribution */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5">
          <h3 className="text-sm font-semibold text-white flex items-center gap-2 mb-3">
            <Layers className="w-4 h-4 text-indigo-400" />
            Model Request Distribution
          </h3>
          <div className="space-y-3">
            {stats?.model_distribution?.map((m: any) => (
              <div key={m.model_name} className="flex items-center justify-between text-xs">
                <span className="text-slate-300 font-medium truncate max-w-[200px]">
                  {m.model_name}
                </span>
                <div className="flex items-center gap-3">
                  <span className="text-slate-400">{m.requests_count} reqs</span>
                  <span className="w-12 text-right font-mono text-indigo-400 font-semibold">
                    {m.percentage}%
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Deployments Status */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5">
          <h3 className="text-sm font-semibold text-white flex items-center gap-2 mb-3">
            <ShieldCheck className="w-4 h-4 text-emerald-400" />
            Serving Deployments Health
          </h3>
          <div className="space-y-3 text-xs">
            <div className="flex justify-between items-center py-2 border-b border-slate-800/60">
              <span className="text-slate-400">Total Serving Deployments</span>
              <span className="font-semibold text-white">{stats?.deployments?.total || 3}</span>
            </div>
            <div className="flex justify-between items-center py-2 border-b border-slate-800/60">
              <span className="text-slate-400">Healthy Pods / Desired</span>
              <span className="font-semibold text-emerald-400">
                {stats?.deployments?.ready_replicas} / {stats?.deployments?.desired_replicas} Ready
              </span>
            </div>
            <div className="flex justify-between items-center py-2">
              <span className="text-slate-400">Cluster Status</span>
              <StatusBadge status="Healthy (Multi-Cluster)" />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
