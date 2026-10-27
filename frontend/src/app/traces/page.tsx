"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { StatusBadge } from "@/components/StatusBadge";
import { ListTree, ShieldAlert, Clock, Coins, Search, FileCode, CheckCircle, AlertCircle } from "lucide-react";

export default function TracesPage() {
  const [traces, setTraces] = useState<any[]>([]);
  const [selectedTrace, setSelectedTrace] = useState<any>(null);
  const [modelFilter, setModelFilter] = useState<string>("all");
  const [loading, setLoading] = useState(true);

  const loadTraces = async () => {
    try {
      setLoading(true);
      const url = modelFilter === "all" ? "/traces" : `/traces?model=${modelFilter}`;
      const data = await fetchApi<any[]>(url);
      setTraces(data);
      if (data.length > 0 && !selectedTrace) {
        setSelectedTrace(data[0]);
      }
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadTraces();
  }, [modelFilter]);

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <ListTree className="w-5 h-5 text-indigo-400" />
            Request Traces & Debugger
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Deep span inspection, prompt/completion payload scrubbers, token breakdown, and error analysis.
          </p>
        </div>
        <div className="flex items-center gap-3">
          <select
            value={modelFilter}
            onChange={(e) => setModelFilter(e.target.value)}
            className="bg-slate-900 border border-slate-800 text-xs text-slate-300 rounded-lg px-3 py-1.5 focus:outline-none focus:border-indigo-500"
          >
            <option value="all">All Models</option>
            <option value="Llama-3.1-70B-Instruct">Llama-3.1-70B-Instruct</option>
            <option value="Mistral-Large-2">Mistral-Large-2</option>
            <option value="Qwen-2.5-Coder-32B">Qwen-2.5-Coder-32B</option>
          </select>
        </div>
      </div>

      {/* Secret Redaction Banner */}
      <div className="rounded-xl border border-indigo-900/60 bg-indigo-950/20 p-4 text-xs flex items-center gap-3">
        <ShieldAlert className="w-4 h-4 text-indigo-400 shrink-0" />
        <div className="text-slate-300">
          <strong className="text-white">Automated Secret Scrubbing Active: </strong>
          Bearer tokens, API keys matching <code className="text-indigo-300 bg-slate-900 px-1 py-0.5 rounded">sk-*</code> or <code className="text-indigo-300 bg-slate-900 px-1 py-0.5 rounded">ghp_*</code> are automatically redacted to <code className="text-amber-300">[REDACTED_SECRET]</code> before persistence.
        </div>
      </div>

      {/* Main Grid: Traces List & Detail Viewer */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left Column: Traces List (5 cols) */}
        <div className="lg:col-span-5 rounded-xl border border-slate-800 bg-slate-900/40 overflow-hidden flex flex-col h-[650px]">
          <div className="p-3.5 border-b border-slate-800 flex items-center justify-between text-xs text-slate-400">
            <span>Production Requests</span>
            <span>{traces.length} traces</span>
          </div>

          <div className="flex-1 overflow-y-auto divide-y divide-slate-800/60">
            {loading ? (
              <div className="p-8 text-center text-xs text-slate-500">Loading traces...</div>
            ) : traces.map((tr) => {
              const isSelected = selectedTrace?.id === tr.id;
              const isError = tr.status_code !== 200;

              return (
                <div
                  key={tr.id}
                  onClick={() => setSelectedTrace(tr)}
                  className={`p-3.5 cursor-pointer text-xs transition ${
                    isSelected ? "bg-indigo-950/30 border-l-2 border-indigo-500" : "hover:bg-slate-800/20"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-white truncate max-w-[200px]">
                      {tr.span_name}
                    </span>
                    <span
                      className={`font-mono text-[10px] px-1.5 py-0.5 rounded font-semibold ${
                        isError
                          ? "bg-rose-950 text-rose-300 border border-rose-800"
                          : "bg-emerald-950 text-emerald-300 border border-emerald-800"
                      }`}
                    >
                      {tr.status_code}
                    </span>
                  </div>

                  <div className="mt-1 flex items-center justify-between text-[11px] text-slate-400">
                    <span>{tr.model_name}</span>
                    <span className="font-mono">{tr.latency_ms} ms</span>
                  </div>

                  <div className="mt-1.5 flex items-center justify-between text-[10px] text-slate-500">
                    <span>{tr.total_tokens} tokens</span>
                    <span>${tr.estimated_cost?.toFixed(5)}</span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Right Column: Selected Trace Detail Viewer (7 cols) */}
        <div className="lg:col-span-7 rounded-xl border border-slate-800 bg-slate-900/40 p-5 flex flex-col h-[650px] overflow-y-auto space-y-4">
          {selectedTrace ? (
            <>
              {/* Header */}
              <div className="flex items-start justify-between border-b border-slate-800 pb-3">
                <div>
                  <h3 className="text-base font-bold text-white flex items-center gap-2">
                    {selectedTrace.span_name}
                  </h3>
                  <div className="text-xs font-mono text-slate-400 mt-0.5">
                    Trace ID: {selectedTrace.trace_id} ({selectedTrace.id})
                  </div>
                </div>
                <StatusBadge
                  status={selectedTrace.status_code === 200 ? "200 Success" : `${selectedTrace.status_code} Failed`}
                />
              </div>

              {/* Metrics Summary Strip */}
              <div className="grid grid-cols-4 gap-2 text-xs">
                <div className="p-2.5 rounded-lg bg-slate-950/60 border border-slate-800">
                  <span className="text-[10px] uppercase text-slate-500 block">Latency</span>
                  <span className="font-mono text-amber-300 font-semibold">{selectedTrace.latency_ms} ms</span>
                </div>
                <div className="p-2.5 rounded-lg bg-slate-950/60 border border-slate-800">
                  <span className="text-[10px] uppercase text-slate-500 block">Tokens (In/Out)</span>
                  <span className="font-mono text-cyan-300 font-semibold">
                    {selectedTrace.prompt_tokens} / {selectedTrace.completion_tokens}
                  </span>
                </div>
                <div className="p-2.5 rounded-lg bg-slate-950/60 border border-slate-800">
                  <span className="text-[10px] uppercase text-slate-500 block">Est. Cost</span>
                  <span className="font-mono text-emerald-300 font-semibold">${selectedTrace.estimated_cost?.toFixed(5)}</span>
                </div>
                <div className="p-2.5 rounded-lg bg-slate-950/60 border border-slate-800">
                  <span className="text-[10px] uppercase text-slate-500 block">Provider</span>
                  <span className="font-mono text-slate-300">{selectedTrace.provider}</span>
                </div>
              </div>

              {/* Error Alert if any */}
              {selectedTrace.error_message && (
                <div className="p-3 rounded-lg border border-rose-900 bg-rose-950/40 text-xs text-rose-300 flex items-start gap-2">
                  <AlertCircle className="w-4 h-4 text-rose-400 shrink-0 mt-0.5" />
                  <div>
                    <span className="font-bold">Execution Error: </span>
                    {selectedTrace.error_message}
                  </div>
                </div>
              )}

              {/* Redacted Prompt Payload */}
              <div>
                <span className="text-xs font-semibold text-slate-300 block mb-1">
                  Redacted Prompt Payload
                </span>
                <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 text-xs font-mono text-slate-300 leading-relaxed max-h-36 overflow-y-auto whitespace-pre-wrap">
                  {selectedTrace.redacted_prompt || "No prompt payload captured"}
                </div>
              </div>

              {/* Redacted Completion Payload */}
              <div>
                <span className="text-xs font-semibold text-slate-300 block mb-1">
                  Redacted Model Completion
                </span>
                <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 text-xs font-mono text-emerald-300/90 leading-relaxed max-h-36 overflow-y-auto whitespace-pre-wrap">
                  {selectedTrace.redacted_completion || "No completion payload captured (status != 200)"}
                </div>
              </div>

              {/* Metadata */}
              {selectedTrace.metadata && (
                <div>
                  <span className="text-xs font-semibold text-slate-300 block mb-1">
                    Span Metadata
                  </span>
                  <pre className="p-3 rounded-lg bg-slate-950 border border-slate-800 text-[11px] font-mono text-indigo-300 overflow-x-auto">
                    {selectedTrace.metadata}
                  </pre>
                </div>
              )}
            </>
          ) : (
            <div className="h-full flex items-center justify-center text-xs text-slate-500">
              Select a trace from the left panel to inspect details.
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
