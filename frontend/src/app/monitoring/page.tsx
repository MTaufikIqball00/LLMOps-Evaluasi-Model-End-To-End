"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { StatusBadge } from "@/components/StatusBadge";
import { TimeseriesChart } from "@/components/SimpleChart";
import { MetricCard } from "@/components/MetricCard";
import { Activity, ExternalLink, Radio, Zap, AlertCircle } from "lucide-react";

export default function MonitoringPage() {
  const [metrics, setMetrics] = useState<any>(null);
  const [distribution, setDistribution] = useState<any[]>([]);
  const [promStatus, setPromStatus] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  const loadData = async () => {
    try {
      setLoading(true);
      const [m, dist, prom] = await Promise.all([
        fetchApi<any>("/monitoring/metrics"),
        fetchApi<any[]>("/monitoring/latency-distribution"),
        fetchApi<any>("/integrations/prometheus/test").catch(() => null),
      ]);
      setMetrics(m);
      setDistribution(dist);
      setPromStatus(prom);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 15000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <Activity className="w-5 h-5 text-indigo-400" />
            Production Telemetry & Observability
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Real-time inference throughput, latency percentiles, error budgets, and Prometheus scraper validation.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <StatusBadge
            status={promStatus?.status || "Prometheus Exporter Standalone"}
            size="md"
          />
        </div>
      </div>

      {/* Observability Source Notice */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-4 text-xs flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Radio className="w-4 h-4 text-emerald-400 animate-pulse" />
          <div>
            <span className="font-semibold text-white">Active Scrape Target: </span>
            <span className="text-slate-400 font-mono">http://localhost:8210/api/v1/metrics</span>
          </div>
        </div>
        <div className="flex items-center gap-2 text-slate-400">
          <span>Port 9190 (Prometheus Target)</span>
          <span className="text-slate-600">|</span>
          <span>Port 3310 (Grafana Target)</span>
        </div>
      </div>

      {/* Throughput & Latency Chart Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Latency Percentiles */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h3 className="text-sm font-semibold text-white flex items-center gap-2">
                <Zap className="w-4 h-4 text-amber-400" />
                Response Latency Percentiles (p50 / p95 / p99)
              </h3>
              <p className="text-xs text-slate-400">Target SLA: p95 &lt; 500ms</p>
            </div>
            <span className="text-[10px] font-mono text-emerald-400 bg-emerald-950/60 px-2 py-0.5 rounded border border-emerald-800/60">
              SLA Met (Nominal)
            </span>
          </div>
          {metrics && (
            <TimeseriesChart
              labels={metrics.timestamps}
              series={[
                { name: "p99 Latency", data: metrics.p99_latency_ms, color: "#f43f5e" },
                { name: "p95 Latency", data: metrics.p95_latency_ms, color: "#f59e0b" },
                { name: "p50 Latency", data: metrics.p50_latency_ms, color: "#10b981" },
              ]}
              height={190}
              unit="ms"
            />
          )}
        </div>

        {/* Request Throughput */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h3 className="text-sm font-semibold text-white flex items-center gap-2">
                <Activity className="w-4 h-4 text-indigo-400" />
                Inference Request Throughput (RPM)
              </h3>
              <p className="text-xs text-slate-400">Requests per minute rolling average</p>
            </div>
            <span className="text-[10px] font-mono text-indigo-400 bg-indigo-950/60 px-2 py-0.5 rounded border border-indigo-800/60">
              Live Flow
            </span>
          </div>
          {metrics && (
            <TimeseriesChart
              labels={metrics.timestamps}
              series={[
                { name: "Throughput (RPM)", data: metrics.throughput_rpm, color: "#6366f1" },
              ]}
              height={190}
              unit="rpm"
            />
          )}
        </div>
      </div>

      {/* Latency Distribution & Grafana Integration Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Latency Buckets */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 space-y-4">
          <h3 className="text-sm font-semibold text-white flex items-center gap-2">
            <Zap className="w-4 h-4 text-indigo-400" />
            Latency Distribution Histogram
          </h3>
          <div className="space-y-3">
            {distribution.map((b) => (
              <div key={b.range_label} className="text-xs">
                <div className="flex justify-between items-center mb-1 text-slate-300">
                  <span>{b.range_label}</span>
                  <span className="font-mono text-slate-400">
                    {b.count} reqs ({b.percentage}%)
                  </span>
                </div>
                <div className="h-2 w-full bg-slate-800 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-indigo-500 rounded-full"
                    style={{ width: `${b.percentage}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Grafana Dashboards Card */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 flex flex-col justify-between">
          <div>
            <h3 className="text-sm font-semibold text-white flex items-center gap-2">
              <ExternalLink className="w-4 h-4 text-amber-400" />
              Grafana Observability Suite
            </h3>
            <p className="text-xs text-slate-400 mt-1">
              External enterprise dashboard visualizing GPU VRAM, kernel throughput, and prompt cache hit rates.
            </p>
            <div className="mt-4 p-3 rounded-lg bg-slate-950/60 border border-slate-800 text-xs space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-slate-400">Grafana URL:</span>
                <span className="font-mono text-indigo-400">http://localhost:3310</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-slate-400">Provisioned Dashboard:</span>
                <span className="text-slate-200">LLMOps-Cluster-vLLM-Overview</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-slate-400">Metrics Pipeline:</span>
                <span className="text-emerald-400">Prometheus Port 9190 Active</span>
              </div>
            </div>
          </div>

          <div className="pt-4 border-t border-slate-800 mt-4 flex items-center justify-between">
            <span className="text-xs text-slate-400">Launch external dashboard:</span>
            <a
              href="http://localhost:3310"
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center gap-1.5 px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-white rounded-lg text-xs font-semibold transition"
            >
              Open Grafana (3310)
              <ExternalLink className="w-3.5 h-3.5" />
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
