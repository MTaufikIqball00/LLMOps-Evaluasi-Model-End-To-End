"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { StatusBadge } from "@/components/StatusBadge";
import {
  FileCode2,
  GitCompare,
  Plus,
  Trash2,
  CheckCircle2,
  Sparkles,
  Tag,
} from "lucide-react";

export default function PromptsPage() {
  const [prompts, setPrompts] = useState<any[]>([]);
  const [stageFilter, setStageFilter] = useState<string>("all");
  const [compareA, setCompareA] = useState<string>("");
  const [compareB, setCompareB] = useState<string>("");
  const [comparison, setComparison] = useState<any>(null);
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [loading, setLoading] = useState(true);

  const [newPrompt, setNewPrompt] = useState({
    name: "RAG Citation Guardrail Prompt",
    version: "v2.1.0",
    model_target: "Llama-3.1-70B-Instruct",
    stage: "Draft",
    description: "Strict evidence-bound RAG synthesis prompt",
    template:
      "You are an enterprise policy assistant.\nUse ONLY verified facts in <context>:\n{{context}}\n\nQuestion: {{question}}\nCitation:",
  });

  const loadPrompts = async () => {
    try {
      setLoading(true);
      const url =
        stageFilter === "all"
          ? "/prompts"
          : `/prompts?stage=${encodeURIComponent(stageFilter)}`;
      const data = await fetchApi<any[]>(url);
      setPrompts(data);
      if (data.length >= 2 && (!compareA || !compareB)) {
        setCompareA(data[1]?.id || data[0].id);
        setCompareB(data[0].id);
      }
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const loadComparison = async (aId: string, bId: string) => {
    if (!aId || !bId) return;
    try {
      const res = await fetchApi<any>(
        `/prompts/compare?a=${encodeURIComponent(aId)}&b=${encodeURIComponent(bId)}`
      );
      setComparison(res);
    } catch (err) {
      console.error(err);
    }
  };

  useEffect(() => {
    loadPrompts();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [stageFilter]);

  useEffect(() => {
    if (compareA && compareB) {
      loadComparison(compareA, compareB);
    }
  }, [compareA, compareB]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await fetchApi("/prompts", {
        method: "POST",
        body: JSON.stringify({
          ...newPrompt,
          tags: ["prompt-registry", "custom"],
        }),
      });
      setIsCreateOpen(false);
      await loadPrompts();
    } catch (err: any) {
      alert(`Failed to create prompt: ${err.message}`);
    }
  };

  const handleStageChange = async (id: string, nextStage: string) => {
    try {
      await fetchApi(`/prompts/${id}`, {
        method: "PATCH",
        body: JSON.stringify({ stage: nextStage }),
      });
      await loadPrompts();
    } catch (err: any) {
      alert(`Failed to update stage: ${err.message}`);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await fetchApi(`/prompts/${id}`, { method: "DELETE" });
      await loadPrompts();
    } catch (err: any) {
      alert(`Failed to delete prompt: ${err.message}`);
    }
  };

  const parseJsonArray = (raw: string | null | undefined): string[] => {
    if (!raw) return [];
    try {
      const parsed = JSON.parse(raw);
      return Array.isArray(parsed) ? parsed : [];
    } catch {
      return [];
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <FileCode2 className="w-5 h-5 text-indigo-400" />
            Prompt Registry &amp; A/B Comparison
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Versioned prompt templates, variable extraction, stage promotion, and side-by-side faithfulness diffing.
          </p>
        </div>

        <button
          onClick={() => setIsCreateOpen(true)}
          className="inline-flex items-center gap-1.5 px-3.5 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold transition"
        >
          <Plus className="w-4 h-4" />
          New Prompt Version
        </button>
      </div>

      {/* Stage Filter Ribbon */}
      <div className="flex flex-wrap items-center gap-2">
        {["all", "Production", "Staging", "Draft", "Archived"].map((st) => (
          <button
            key={st}
            onClick={() => setStageFilter(st)}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition ${
              stageFilter === st
                ? "bg-indigo-600/20 text-indigo-300 border border-indigo-500/40"
                : "bg-slate-900/60 text-slate-400 border border-slate-800 hover:text-slate-200"
            }`}
          >
            {st === "all" ? "All Stages" : st}
          </button>
        ))}
      </div>

      {/* Prompts Grid */}
      {loading && prompts.length === 0 ? (
        <div className="h-32 rounded-xl border border-slate-800 bg-slate-900/40 animate-pulse" />
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {prompts.map((p) => {
            const vars = parseJsonArray(p.variables);
            return (
              <div
                key={p.id}
                className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 space-y-3 flex flex-col justify-between"
              >
                <div className="space-y-3">
                  <div className="flex items-start justify-between gap-2">
                    <div>
                      <div className="flex items-center gap-2">
                        <h3 className="text-sm font-bold text-white">{p.name}</h3>
                        <span className="text-[11px] font-mono px-1.5 py-0.5 rounded bg-slate-800 text-cyan-300">
                          {p.version}
                        </span>
                      </div>
                      <div className="text-[11px] text-slate-400 font-mono mt-0.5">
                        slug: {p.slug} | target: {p.model_target}
                      </div>
                    </div>
                    <StatusBadge status={p.stage} />
                  </div>

                  {p.description && (
                    <p className="text-xs text-slate-400">{p.description}</p>
                  )}

                  <pre className="p-3 rounded-lg bg-slate-950 border border-slate-800 text-[11px] font-mono text-slate-200 whitespace-pre-wrap overflow-x-auto max-h-36">
                    {p.template}
                  </pre>

                  <div className="flex flex-wrap items-center gap-1.5">
                    {vars.map((v) => (
                      <span
                        key={v}
                        className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-indigo-950/60 text-indigo-300 border border-indigo-800/50 text-[10px] font-mono"
                      >
                        <Tag className="w-2.5 h-2.5" />
                        {`{{${v}}}`}
                      </span>
                    ))}
                  </div>
                </div>

                <div className="pt-3 border-t border-slate-800 flex flex-wrap items-center justify-between gap-2 text-xs">
                  <div className="flex items-center gap-3 text-slate-400 font-mono text-[11px]">
                    <span>
                      Faithfulness:{" "}
                      <strong className="text-emerald-400">
                        {((p.avg_faithfulness || 0) * 100).toFixed(1)}%
                      </strong>
                    </span>
                    <span>
                      Latency: <strong className="text-amber-300">{p.avg_latency_ms}ms</strong>
                    </span>
                  </div>

                  <div className="flex items-center gap-1.5">
                    <select
                      value={p.stage}
                      onChange={(e) => handleStageChange(p.id, e.target.value)}
                      className="bg-slate-950 border border-slate-800 rounded px-2 py-1 text-[11px] text-slate-200"
                    >
                      <option value="Draft">Draft</option>
                      <option value="Staging">Staging</option>
                      <option value="Production">Production</option>
                      <option value="Archived">Archived</option>
                    </select>
                    <button
                      onClick={() => handleDelete(p.id)}
                      className="p-1.5 rounded bg-slate-900 hover:bg-rose-950/60 text-slate-400 hover:text-rose-400 border border-slate-800 transition"
                      title="Delete prompt version"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* A/B Prompt Comparison & Diff Viewer */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 space-y-4">
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-3 border-b border-slate-800 pb-3">
          <div>
            <h3 className="text-sm font-bold text-white flex items-center gap-2">
              <GitCompare className="w-4 h-4 text-indigo-400" />
              A/B Prompt Template Diff &amp; Quality Comparison
            </h3>
            <p className="text-xs text-slate-400 mt-0.5">
              Compare line changes, variable contracts, and faithfulness/latency impact across versions.
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2 text-xs">
            <select
              value={compareA}
              onChange={(e) => setCompareA(e.target.value)}
              className="bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-slate-200"
            >
              {prompts.map((p) => (
                <option key={p.id} value={p.id}>
                  Baseline A: {p.slug} ({p.version})
                </option>
              ))}
            </select>
            <span className="text-slate-500 font-semibold">vs</span>
            <select
              value={compareB}
              onChange={(e) => setCompareB(e.target.value)}
              className="bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-slate-200"
            >
              {prompts.map((p) => (
                <option key={p.id} value={p.id}>
                  Candidate B: {p.slug} ({p.version})
                </option>
              ))}
            </select>
          </div>
        </div>

        {comparison ? (
          <div className="space-y-4">
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-xs">
              <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800">
                <span className="text-slate-400 block text-[10px] uppercase">Faithfulness Delta (B - A)</span>
                <span
                  className={`font-mono font-bold text-base ${
                    comparison.faithfulness_delta >= 0 ? "text-emerald-400" : "text-rose-400"
                  }`}
                >
                  {comparison.faithfulness_delta >= 0 ? "+" : ""}
                  {(comparison.faithfulness_delta * 100).toFixed(1)}%
                </span>
              </div>
              <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800">
                <span className="text-slate-400 block text-[10px] uppercase">Latency Delta (B - A)</span>
                <span
                  className={`font-mono font-bold text-base ${
                    comparison.latency_delta_ms <= 0 ? "text-emerald-400" : "text-amber-300"
                  }`}
                >
                  {comparison.latency_delta_ms > 0 ? "+" : ""}
                  {comparison.latency_delta_ms} ms
                </span>
              </div>
              <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800">
                <span className="text-slate-400 block text-[10px] uppercase">Variable Contract</span>
                <span className="font-mono text-slate-200 text-xs mt-1 block">
                  Shared: {comparison.shared_variables?.join(", ") || "None"}
                </span>
              </div>
            </div>

            <div className="rounded-lg bg-slate-950 border border-slate-800 overflow-hidden font-mono text-[11px]">
              {comparison.diff_lines?.map((dl: any, idx: number) => (
                <div
                  key={`${dl.line_number}-${idx}`}
                  className={`px-3 py-1 border-b border-slate-900/60 flex gap-3 ${
                    dl.change_type === "added"
                      ? "bg-emerald-950/40 text-emerald-300"
                      : dl.change_type === "removed"
                      ? "bg-rose-950/40 text-rose-300"
                      : "text-slate-400"
                  }`}
                >
                  <span className="w-4 shrink-0 font-bold">
                    {dl.change_type === "added" ? "+" : dl.change_type === "removed" ? "-" : " "}
                  </span>
                  <span className="whitespace-pre-wrap">{dl.content}</span>
                </div>
              ))}
            </div>
          </div>
        ) : (
          <div className="text-xs text-slate-400 flex items-center gap-2">
            <Sparkles className="w-4 h-4 text-indigo-400" />
            Select two prompt templates above to inspect line-by-line diffs.
          </div>
        )}
      </div>

      {/* Create Prompt Modal */}
      {isCreateOpen && (
        <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl max-w-lg w-full p-6 space-y-4 shadow-2xl">
            <h3 className="text-base font-bold text-white flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-indigo-400" />
              Register New Prompt Version
            </h3>
            <form onSubmit={handleCreate} className="space-y-3 text-xs">
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-slate-400 mb-1">Prompt Name</label>
                  <input
                    required
                    value={newPrompt.name}
                    onChange={(e) => setNewPrompt({ ...newPrompt, name: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white"
                  />
                </div>
                <div>
                  <label className="block text-slate-400 mb-1">Version</label>
                  <input
                    required
                    value={newPrompt.version}
                    onChange={(e) => setNewPrompt({ ...newPrompt, version: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white font-mono"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-slate-400 mb-1">Target Model</label>
                  <input
                    required
                    value={newPrompt.model_target}
                    onChange={(e) => setNewPrompt({ ...newPrompt, model_target: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white"
                  />
                </div>
                <div>
                  <label className="block text-slate-400 mb-1">Stage</label>
                  <select
                    value={newPrompt.stage}
                    onChange={(e) => setNewPrompt({ ...newPrompt, stage: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white"
                  >
                    <option value="Draft">Draft</option>
                    <option value="Staging">Staging</option>
                    <option value="Production">Production</option>
                  </select>
                </div>
              </div>

              <div>
                <label className="block text-slate-400 mb-1">Prompt Template (use {`{{var}}`} syntax)</label>
                <textarea
                  required
                  rows={5}
                  value={newPrompt.template}
                  onChange={(e) => setNewPrompt({ ...newPrompt, template: e.target.value })}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white font-mono"
                />
              </div>

              <div className="flex justify-end gap-2 pt-3 border-t border-slate-800">
                <button
                  type="button"
                  onClick={() => setIsCreateOpen(false)}
                  className="px-3 py-1.5 bg-slate-800 text-slate-300 rounded-lg text-xs"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-3.5 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold"
                >
                  Save Prompt Version
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
