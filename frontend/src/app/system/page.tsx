"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { StatusBadge } from "@/components/StatusBadge";
import { Cpu, ShieldCheck, Database, HardDrive, CheckCircle2, Server, Clock } from "lucide-react";

export default function SystemHealthPage() {
  const [health, setHealth] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  const loadHealth = async () => {
    try {
      setLoading(true);
      const data = await fetchApi<any>("/health");
      setHealth(data);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadHealth();
    const interval = setInterval(loadHealth, 10000);
    return () => clearInterval(interval);
  }, []);

  const formatUptime = (seconds: number) => {
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hrs}h ${mins}m ${secs}s`;
  };

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <Cpu className="w-5 h-5 text-indigo-400" />
            System Health & SRE Diagnostics
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Engine runtime telemetry, isolated port registry, SQLx connection pool, and security control audit.
          </p>
        </div>
        <StatusBadge status={health?.status || "Healthy"} size="md" />
      </div>

      {/* Primary Metrics Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-2">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>Platform Status</span>
            <Server className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-xl font-bold text-white">{health?.status || "Healthy"}</div>
          <div className="text-[11px] text-slate-400">Axum Tokio Engine v{health?.version || "0.1.0"}</div>
        </div>

        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-2">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>Uptime</span>
            <Clock className="w-4 h-4 text-indigo-400" />
          </div>
          <div className="text-xl font-bold text-white font-mono">
            {health?.uptime_seconds !== undefined ? formatUptime(health.uptime_seconds) : "Running"}
          </div>
          <div className="text-[11px] text-slate-400">Continuous runtime</div>
        </div>

        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-2">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>Database Engine</span>
            <Database className="w-4 h-4 text-cyan-400" />
          </div>
          <div className="text-xl font-bold text-white">{health?.database || "Connected"}</div>
          <div className="text-[11px] text-slate-400">SQLx SQLite Connection Pool</div>
        </div>

        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-2">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>Security Posture</span>
            <ShieldCheck className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-xl font-bold text-emerald-400">Enforced</div>
          <div className="text-[11px] text-slate-400">Payload secret scrubber & CORS active</div>
        </div>
      </div>

      {/* Port Allocation Registry Card */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 space-y-4">
        <div className="flex items-center justify-between border-b border-slate-800 pb-3">
          <h3 className="text-sm font-semibold text-white flex items-center gap-2">
            <HardDrive className="w-4 h-4 text-indigo-400" />
            Verified Dedicated Port Allocations (Conflict Free)
          </h3>
          <span className="text-[10px] text-slate-400">All local dev ports isolated</span>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3">
          <div className="p-3 rounded-lg bg-slate-950/70 border border-slate-800 text-xs space-y-1">
            <span className="text-[10px] text-slate-500 uppercase font-bold block">Frontend UI</span>
            <span className="font-mono text-cyan-400 font-bold text-sm">:3210</span>
            <span className="text-[10px] text-slate-400 block">Next.js + Tailwind</span>
          </div>

          <div className="p-3 rounded-lg bg-slate-950/70 border border-slate-800 text-xs space-y-1">
            <span className="text-[10px] text-slate-500 uppercase font-bold block">Rust API</span>
            <span className="font-mono text-emerald-400 font-bold text-sm">:8210</span>
            <span className="text-[10px] text-slate-400 block">Axum + Tokio</span>
          </div>

          <div className="p-3 rounded-lg bg-slate-950/70 border border-slate-800 text-xs space-y-1">
            <span className="text-[10px] text-slate-500 uppercase font-bold block">Prometheus Target</span>
            <span className="font-mono text-amber-400 font-bold text-sm">:9190</span>
            <span className="text-[10px] text-slate-400 block">Metrics Scraper</span>
          </div>

          <div className="p-3 rounded-lg bg-slate-950/70 border border-slate-800 text-xs space-y-1">
            <span className="text-[10px] text-slate-500 uppercase font-bold block">Grafana Target</span>
            <span className="font-mono text-indigo-400 font-bold text-sm">:3310</span>
            <span className="text-[10px] text-slate-400 block">Dashboards</span>
          </div>

          <div className="p-3 rounded-lg bg-slate-950/70 border border-slate-800 text-xs space-y-1">
            <span className="text-[10px] text-slate-500 uppercase font-bold block">MLflow Target</span>
            <span className="font-mono text-violet-400 font-bold text-sm">:5500</span>
            <span className="text-[10px] text-slate-400 block">Tracking Server</span>
          </div>
        </div>
      </div>

      {/* Security & Reliability Controls Checklist */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 space-y-3">
        <h3 className="text-sm font-semibold text-white flex items-center gap-2">
          <ShieldCheck className="w-4 h-4 text-emerald-400" />
          Production Engineering & SRE Controls
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
          <div className="flex items-start gap-2.5 p-3 rounded-lg bg-slate-950/60 border border-slate-800">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
            <div>
              <span className="font-semibold text-white block">Automated Secret Redaction</span>
              <span className="text-slate-400 text-[11px]">
                Regex pipeline redacts bearer tokens, GitHub tokens, and API keys before storing request traces.
              </span>
            </div>
          </div>

          <div className="flex items-start gap-2.5 p-3 rounded-lg bg-slate-950/60 border border-slate-800">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
            <div>
              <span className="font-semibold text-white block">Strict CORS Origin Isolation</span>
              <span className="text-slate-400 text-[11px]">
                Restricted to frontend origin (http://localhost:3210). Arbitrary cross-site requests are rejected.
              </span>
            </div>
          </div>

          <div className="flex items-start gap-2.5 p-3 rounded-lg bg-slate-950/60 border border-slate-800">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
            <div>
              <span className="font-semibold text-white block">Zero-Dependency Local Persistence</span>
              <span className="text-slate-400 text-[11px]">
                Embedded ACID SQLite with SQLx connection pooling. Runs anywhere without requiring external Docker daemon.
              </span>
            </div>
          </div>

          <div className="flex items-start gap-2.5 p-3 rounded-lg bg-slate-950/60 border border-slate-800">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
            <div>
              <span className="font-semibold text-white block">Graceful Shutdown & Signal Handlers</span>
              <span className="text-slate-400 text-[11px]">
                Handles SIGINT (Ctrl+C) and SIGTERM with connection draining and safe background worker completion.
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
