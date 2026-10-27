"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { StatusBadge } from "@/components/StatusBadge";
import {
  CheckCircle2,
  AlertTriangle,
  Play,
  HelpCircle,
  FileText,
  ChevronRight,
  ShieldAlert,
} from "lucide-react";

export default function EvaluationsPage() {
  const [evaluations, setEvaluations] = useState<any[]>([]);
  const [selectedEval, setSelectedEval] = useState<any>(null);
  const [detailData, setDetailData] = useState<any>(null);
  const [definitions, setDefinitions] = useState<any[]>([]);
  const [showDefs, setShowDefs] = useState(false);
  const [isTriggerModalOpen, setIsTriggerModalOpen] = useState(false);
  const [loading, setLoading] = useState(true);

  const [newRun, setNewRun] = useState({
    name: "Enterprise RAG Hallucination & Faithfulness Gate",
    model_name: "Llama-3.1-70B-Instruct",
    dataset_name: "enterprise-kb-v3-eval",
    evaluator_framework: "Ragas & TruLens Suite",
  });

  const loadEvaluations = async () => {
    try {
      setLoading(true);
      const data = await fetchApi<any[]>("/evaluations");
      setEvaluations(data);
      if (data.length > 0 && !selectedEval) {
        setSelectedEval(data[0]);
      }
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const loadDetail = async (id: string) => {
    try {
      const data = await fetchApi<any>(`/evaluations/${id}`);
      setDetailData(data);
    } catch (err) {
      console.error(err);
    }
  };

  const loadDefinitions = async () => {
    try {
      const defs = await fetchApi<any[]>("/evaluations/metrics/definitions");
      setDefinitions(defs);
    } catch (err) {
      console.error(err);
    }
  };

  useEffect(() => {
    loadEvaluations();
    loadDefinitions();
  }, []);

  useEffect(() => {
    if (selectedEval) {
      loadDetail(selectedEval.id);
    }
  }, [selectedEval]);

  const handleTriggerRun = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await fetchApi("/evaluations", {
        method: "POST",
        body: JSON.stringify(newRun),
      });
      setIsTriggerModalOpen(false);
      loadEvaluations();
      setTimeout(loadEvaluations, 1500);
    } catch (err: any) {
      alert(`Failed to trigger evaluation: ${err.message}`);
    }
  };

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <CheckCircle2 className="w-5 h-5 text-indigo-400" />
            LLM Evaluation & Hallucination Analysis
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Ragas & TruLens feedback loops: Faithfulness, Answer Relevance, Context Precision, and Ungrounded Assertion Detection.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowDefs(!showDefs)}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-slate-900 hover:bg-slate-850 border border-slate-800 text-slate-300 rounded-lg text-xs transition"
          >
            <HelpCircle className="w-4 h-4 text-indigo-400" />
            Metric Definitions
          </button>
          <button
            onClick={() => setIsTriggerModalOpen(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold shadow-sm transition"
          >
            <Play className="w-3.5 h-3.5 fill-current" />
            New Evaluation Run
          </button>
        </div>
      </div>

      {/* Metric Definitions Drawer/Card */}
      {showDefs && (
        <div className="rounded-xl border border-indigo-900/50 bg-indigo-950/20 p-5 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-bold text-indigo-300 flex items-center gap-2">
              <HelpCircle className="w-4 h-4" />
              Standard Evaluation Metrics & Hallucination Diagnostics
            </h3>
            <span className="text-[11px] text-slate-400">
              Note: Automated metrics are statistical estimators, not absolute mathematical truth.
            </span>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 pt-2">
            {definitions.map((def) => (
              <div key={def.metric_name} className="p-3 rounded-lg bg-slate-900/80 border border-slate-800 text-xs">
                <div className="font-semibold text-white flex items-center justify-between">
                  <span>{def.metric_name}</span>
                  <span className="text-[10px] font-mono text-indigo-400">{def.target_range}</span>
                </div>
                <p className="text-[11px] text-slate-300 mt-1">{def.formula_or_approach}</p>
                <div className="mt-2 pt-2 border-t border-slate-800 text-[10px] text-amber-300/90">
                  <span className="font-semibold">Hallucination Link: </span>
                  {def.hallucination_relevance}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Evaluation Runs Table */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/40 overflow-hidden">
        <div className="p-3.5 border-b border-slate-800 flex items-center justify-between">
          <span className="text-xs font-semibold text-white">Evaluation Benchmarks</span>
          <span className="text-[11px] text-slate-400">Click a benchmark to inspect test cases</span>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-slate-950/60 text-slate-400 uppercase font-semibold border-b border-slate-800">
              <tr>
                <th className="py-3 px-4">Evaluation Run</th>
                <th className="py-3 px-4">Model & Dataset</th>
                <th className="py-3 px-4">Framework</th>
                <th className="py-3 px-4">Faithfulness</th>
                <th className="py-3 px-4">Relevance</th>
                <th className="py-3 px-4">Groundedness</th>
                <th className="py-3 px-4">Hallucination</th>
                <th className="py-3 px-4">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              {evaluations.map((ev) => {
                const isSelected = selectedEval?.id === ev.id;
                return (
                  <tr
                    key={ev.id}
                    onClick={() => setSelectedEval(ev)}
                    className={`cursor-pointer transition ${
                      isSelected ? "bg-indigo-950/30" : "hover:bg-slate-800/30"
                    }`}
                  >
                    <td className="py-3.5 px-4 font-medium text-white flex items-center gap-2">
                      <ChevronRight className={`w-3.5 h-3.5 ${isSelected ? "text-indigo-400" : "text-slate-600"}`} />
                      <div>
                        <div>{ev.name}</div>
                        <div className="text-[10px] font-mono text-slate-500">{ev.id}</div>
                      </div>
                    </td>
                    <td className="py-3.5 px-4">
                      <div className="text-slate-200 font-medium">{ev.model_name}</div>
                      <div className="text-[11px] text-slate-500">{ev.dataset_name}</div>
                    </td>
                    <td className="py-3.5 px-4 font-mono text-indigo-300">
                      {ev.evaluator_framework}
                    </td>
                    <td className="py-3.5 px-4 font-mono font-bold text-emerald-400">
                      {(ev.faithfulness * 100).toFixed(1)}%
                    </td>
                    <td className="py-3.5 px-4 font-mono text-slate-300">
                      {(ev.answer_relevance * 100).toFixed(1)}%
                    </td>
                    <td className="py-3.5 px-4 font-mono text-cyan-400">
                      {(ev.groundedness * 100).toFixed(1)}%
                    </td>
                    <td className="py-3.5 px-4 font-mono font-bold text-rose-400">
                      {(ev.hallucination_score * 100).toFixed(1)}%
                    </td>
                    <td className="py-3.5 px-4">
                      <StatusBadge status={ev.status} />
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* Selected Evaluation Test Cases Inspector */}
      {detailData && (
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 space-y-4">
          <div className="flex items-center justify-between border-b border-slate-800 pb-3">
            <div>
              <h3 className="text-sm font-bold text-white flex items-center gap-2">
                <FileText className="w-4 h-4 text-indigo-400" />
                Individual Test Case Verification ({detailData.evaluation.name})
              </h3>
              <p className="text-xs text-slate-400 mt-0.5">
                Inspect retrieved chunks, model answer, faithfulness breakdown, and hallucination flags.
              </p>
            </div>
            <div className="flex items-center gap-3 text-xs">
              <span className="text-slate-400">Avg Latency: <strong className="text-white">{detailData.evaluation.avg_latency_ms} ms</strong></span>
              <span className="text-slate-400">Tokens: <strong className="text-white">{detailData.evaluation.total_tokens?.toLocaleString()}</strong></span>
            </div>
          </div>

          <div className="space-y-4">
            {detailData.test_cases?.map((tc: any, idx: number) => {
              const isHallucinated = tc.hallucination_detected === 1;

              return (
                <div
                  key={tc.id}
                  className={`rounded-lg border p-4 text-xs space-y-3 transition ${
                    isHallucinated
                      ? "border-rose-800/80 bg-rose-950/20"
                      : "border-slate-800 bg-slate-950/60"
                  }`}
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex items-center gap-2 font-semibold text-white">
                      <span className="w-5 h-5 rounded-full bg-slate-800 flex items-center justify-center text-[10px] text-slate-400">
                        {idx + 1}
                      </span>
                      <span>{tc.question}</span>
                    </div>

                    <div className="flex items-center gap-2 shrink-0">
                      {isHallucinated ? (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-rose-950 text-rose-300 border border-rose-800 font-semibold">
                          <ShieldAlert className="w-3.5 h-3.5 text-rose-400" />
                          Hallucination Flagged
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-950 text-emerald-300 border border-emerald-800 font-semibold">
                          <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                          Faithful to Evidence
                        </span>
                      )}
                      <span className="font-mono text-slate-400 text-[11px]">
                        Faithfulness: {(tc.faithfulness_score * 100).toFixed(0)}%
                      </span>
                    </div>
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3 pt-2">
                    {/* Retrieved Context */}
                    <div className="rounded bg-slate-900/90 p-3 border border-slate-800">
                      <span className="text-[10px] uppercase font-bold text-slate-400 block mb-1">
                        Retrieved Context Evidence
                      </span>
                      <p className="text-slate-300 text-[11px] leading-relaxed">
                        {tc.retrieved_context || "No context attached"}
                      </p>
                    </div>

                    {/* Generated Answer */}
                    <div className="rounded bg-slate-900/90 p-3 border border-slate-800">
                      <span className="text-[10px] uppercase font-bold text-slate-400 block mb-1">
                        Model Generated Answer
                      </span>
                      <p className={`text-[11px] leading-relaxed ${isHallucinated ? "text-rose-200" : "text-emerald-200"}`}>
                        {tc.generated_answer}
                      </p>
                    </div>
                  </div>

                  {tc.ground_truth && (
                    <div className="text-[11px] text-slate-400 pt-1">
                      <span className="font-semibold text-slate-300">Ground Truth Reference: </span>
                      {tc.ground_truth}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Trigger Run Modal */}
      {isTriggerModalOpen && (
        <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl max-w-md w-full p-6 space-y-4 shadow-2xl">
            <h3 className="text-base font-bold text-white flex items-center gap-2">
              <Play className="w-4 h-4 text-indigo-400 fill-current" />
              Trigger New Evaluation Benchmark
            </h3>
            <form onSubmit={handleTriggerRun} className="space-y-3 text-xs">
              <div>
                <label className="block text-slate-400 mb-1">Benchmark Run Name</label>
                <input
                  required
                  value={newRun.name}
                  onChange={(e) => setNewRun({ ...newRun, name: e.target.value })}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                />
              </div>
              <div>
                <label className="block text-slate-400 mb-1">Target Model</label>
                <select
                  value={newRun.model_name}
                  onChange={(e) => setNewRun({ ...newRun, model_name: e.target.value })}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="Llama-3.1-70B-Instruct">Llama-3.1-70B-Instruct</option>
                  <option value="Mistral-Large-2">Mistral-Large-2</option>
                  <option value="Qwen-2.5-Coder-32B">Qwen-2.5-Coder-32B</option>
                </select>
              </div>
              <div>
                <label className="block text-slate-400 mb-1">Golden Dataset Name</label>
                <input
                  value={newRun.dataset_name}
                  onChange={(e) => setNewRun({ ...newRun, dataset_name: e.target.value })}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                />
              </div>
              <div>
                <label className="block text-slate-400 mb-1">Evaluation Framework</label>
                <select
                  value={newRun.evaluator_framework}
                  onChange={(e) => setNewRun({ ...newRun, evaluator_framework: e.target.value })}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="Ragas & TruLens Suite">Ragas & TruLens Suite (Native Runner)</option>
                  <option value="TruLens Triad">TruLens Triad</option>
                  <option value="Native Algorithmic Suite">Native Algorithmic Suite</option>
                </select>
              </div>

              <div className="flex justify-end gap-2 pt-3 border-t border-slate-800">
                <button
                  type="button"
                  onClick={() => setIsTriggerModalOpen(false)}
                  className="px-3 py-1.5 bg-slate-800 text-slate-300 rounded-lg text-xs"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold"
                >
                  Start Benchmark Job
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
