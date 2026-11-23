"use client";

import React, { useEffect, useState } from "react";
import {
  API_BASE_URL,
  ApiConnectionMode,
  fetchApi,
  getConnectionMode,
  subscribeConnectionMode,
} from "@/lib/api";
import { Cloud, RefreshCw, Server, ShieldCheck } from "lucide-react";

export function Header() {
  const [backendHealth, setBackendHealth] = useState<string>("Checking...");
  const [mode, setMode] = useState<ApiConnectionMode>(getConnectionMode());
  const [isRefreshing, setIsRefreshing] = useState(false);

  const checkHealth = async () => {
    try {
      setIsRefreshing(true);
      const res = await fetchApi<any>("/health");
      const currentMode = getConnectionMode();
      setMode(currentMode);
      if (currentMode === "cloud_fallback") {
        setBackendHealth("Cloud Fallback Mode");
      } else {
        setBackendHealth(res.status === "Healthy" ? "Live Rust API" : res.status);
      }
    } catch {
      setBackendHealth("Offline (Start backend)");
    } finally {
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    const unsub = subscribeConnectionMode((nextMode) => {
      setMode(nextMode);
      if (nextMode === "cloud_fallback") {
        setBackendHealth("Cloud Fallback Mode");
      } else if (nextMode === "live") {
        setBackendHealth("Live Rust API");
      }
    });
    checkHealth();
    const interval = setInterval(checkHealth, 15000);
    return () => {
      unsub();
      clearInterval(interval);
    };
  }, []);

  const isLive = mode === "live";
  const isFallback = mode === "cloud_fallback";

  return (
    <header className="h-16 border-b border-slate-800 bg-slate-950/40 backdrop-blur-md px-6 flex items-center justify-between sticky top-0 z-20">
      <div className="flex items-center gap-3">
        <span className="text-xs font-semibold px-2 py-0.5 rounded bg-emerald-950/80 text-emerald-300 border border-emerald-800/80 flex items-center gap-1.5">
          <ShieldCheck className="w-3 h-3 text-emerald-400" />
          Production Environment
        </span>
        <span className="text-xs text-slate-400">|</span>
        <span className="text-xs text-slate-400 font-mono">
          {isFallback ? "Source: Serverless / Edge Fallback Store" : `API: ${API_BASE_URL}`}
        </span>
      </div>

      <div className="flex items-center gap-4">
        {/* Transparent mode indicator: Live Rust API vs Cloud Fallback Mode */}
        <div className="flex items-center gap-2 text-xs">
          <span className="text-slate-400 flex items-center gap-1">
            {isFallback ? <Cloud className="w-3.5 h-3.5 text-amber-400" /> : <Server className="w-3.5 h-3.5" />}
            Mode:
          </span>
          <span
            className={`font-medium px-2 py-0.5 rounded flex items-center gap-1.5 ${
              isLive
                ? "bg-emerald-950/60 text-emerald-400 border border-emerald-800/50"
                : isFallback
                ? "bg-amber-950/60 text-amber-300 border border-amber-800/50"
                : "bg-slate-900 text-slate-300 border border-slate-700"
            }`}
          >
            <span
              className={`w-1.5 h-1.5 rounded-full ${
                isLive ? "bg-emerald-400" : isFallback ? "bg-amber-400" : "bg-slate-400"
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
