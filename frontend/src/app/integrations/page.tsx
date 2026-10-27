"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { StatusBadge } from "@/components/StatusBadge";
import { Radio, RefreshCw, CheckCircle, AlertTriangle, ExternalLink, Settings, Zap } from "lucide-react";

export default function IntegrationsPage() {
  const [integrations, setIntegrations] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [testingService, setTestingService] = useState<string | null>(null);

  const loadIntegrations = async () => {
    try {
      setLoading(true);
      const data = await fetchApi<any[]>("/integrations");
      setIntegrations(data);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadIntegrations();
  }, []);

  const handleTestConnection = async (serviceName: string) => {
    try {
      setTestingService(serviceName);
      const res = await fetchApi<any>(`/integrations/${serviceName}/test`, {
        method: "POST",
      });
      setIntegrations((prev) =>
        prev.map((item) => (item.service_name === serviceName ? res : item))
      );
    } catch (err: any) {
      alert(`Test failed: ${err.message}`);
    } finally {
      setTestingService(null);
    }
  };

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <Radio className="w-5 h-5 text-indigo-400" />
            Integrations & Service Adapters
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Real-time connection verification for MLflow, W&B, Prometheus, Grafana, Ragas, TruLens, Kubernetes, and GitHub Actions.
          </p>
        </div>
        <button
          onClick={loadIntegrations}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-slate-900 hover:bg-slate-850 border border-slate-800 text-slate-300 rounded-lg text-xs transition"
        >
          <RefreshCw className="w-3.5 h-3.5" />
          Probe All Adapters
        </button>
      </div>

      {/* Integration Notice */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-4 text-xs text-slate-300">
        <strong className="text-white">Active Extensible Architecture: </strong>
        Services are probed with active HTTP & system checks. Offline or unconfigured external services fail gracefully to local database records with clear source labeling, ensuring full local zero-setup operability.
      </div>

      {/* Integrations Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {loading ? (
          <div className="col-span-2 text-center py-12 text-xs text-slate-500">
            Probing integration adapters...
          </div>
        ) : (
          integrations.map((item) => {
            const isTesting = testingService === item.service_name;

            return (
              <div
                key={item.service_name}
                className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 space-y-4 hover:border-slate-700 transition flex flex-col justify-between"
              >
                <div>
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <h3 className="font-semibold text-white text-sm">{item.display_name}</h3>
                      <span className="text-[10px] uppercase font-bold text-slate-500 tracking-wider">
                        {item.category}
                      </span>
                    </div>
                    <StatusBadge status={item.status} />
                  </div>

                  <p className="text-xs text-slate-300 mt-3 leading-relaxed">
                    {item.message}
                  </p>

                  <div className="mt-4 p-3 rounded-lg bg-slate-950/70 border border-slate-850 text-xs space-y-1.5">
                    <div className="flex justify-between items-center text-slate-400">
                      <span>Endpoint:</span>
                      <span className="font-mono text-indigo-300 truncate max-w-[240px]" title={item.endpoint_url}>
                        {item.endpoint_url || "Not configured"}
                      </span>
                    </div>
                    <div className="flex justify-between items-center text-slate-400">
                      <span>Probe Latency:</span>
                      <span className="font-mono text-emerald-400 font-semibold">
                        {item.latency_ms !== null && item.latency_ms !== undefined
                          ? `${item.latency_ms.toFixed(1)} ms`
                          : "N/A (Offline)"}
                      </span>
                    </div>
                  </div>
                </div>

                <div className="pt-3 border-t border-slate-800 flex items-center justify-between text-xs">
                  <span className="text-[10px] text-slate-500">
                    Checked: {item.last_checked ? new Date(item.last_checked).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) : "Recently"}
                  </span>
                  <button
                    onClick={() => handleTestConnection(item.service_name)}
                    disabled={isTesting}
                    className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-white font-semibold transition"
                  >
                    <RefreshCw className={`w-3.5 h-3.5 ${isTesting ? "animate-spin text-indigo-400" : ""}`} />
                    Test Connection
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
