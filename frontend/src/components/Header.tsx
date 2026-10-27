"use client";

import React, { useEffect, useState } from "react";
import { API_BASE_URL, fetchApi } from "@/lib/api";
import { RefreshCw, Server, ShieldCheck } from "lucide-react";

export function Header() {
  const [backendHealth, setBackendHealth] = useState<string>("Checking...");
  const [isRefreshing, setIsRefreshing] = useState(false);

  const checkHealth = async () => {
    try {
      setIsRefreshing(true);
      const res = await fetchApi<any>("/health");
      setBackendHealth(res.status === "Healthy" ? "Connected" : res.status);
    } catch {
      setBackendHealth("Offline (Start backend)");
    } finally {
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    checkHealth();
    const interval = setInterval(checkHealth, 15000);
    return () => clearInterval(interval);
  }, []);

  return (
    <header className="h-16 border-b border-slate-800 bg-slate-950/40 backdrop-blur-md px-6 flex items-center justify-between sticky top-0 z-20">
      <div className="flex items-center gap-3">
        <span className="text-xs font-semibold px-2 py-0.5 rounded bg-emerald-950/80 text-emerald-300 border border-emerald-800/80 flex items-center gap-1.5">
          <ShieldCheck className="w-3 h-3 text-emerald-400" />
          Production Environment
        </span>
        <span className="text-xs text-slate-400">|</span>
        <span className="text-xs text-slate-400 font-mono">
          API: {API_BASE_URL}
        </span>
      </div>

      <div className="flex items-center gap-4">
        {/* Backend health status badge */}
        <div className="flex items-center gap-2 text-xs">
          <span className="text-slate-400 flex items-center gap-1">
            <Server className="w-3.5 h-3.5" />
            Backend:
          </span>
          <span
            className={`font-medium px-2 py-0.5 rounded flex items-center gap-1.5 ${
              backendHealth === "Connected"
                ? "bg-emerald-950/60 text-emerald-400 border border-emerald-800/50"
                : "bg-rose-950/60 text-rose-400 border border-rose-800/50"
            }`}
          >
            <span
              className={`w-1.5 h-1.5 rounded-full ${
                backendHealth === "Connected" ? "bg-emerald-400" : "bg-rose-400"
              }`}
            />
            {backendHealth}
          </span>
        </div>

        {/* Reload button */}
        <button
          onClick={checkHealth}
          disabled={isRefreshing}
          className="p-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-400 hover:text-white hover:border-slate-700 transition"
          title="Refresh health & telemetry"
        >
          <RefreshCw
            className={`w-4 h-4 ${isRefreshing ? "animate-spin text-indigo-400" : ""}`}
          />
        </button>
      </div>
    </header>
  );
}
