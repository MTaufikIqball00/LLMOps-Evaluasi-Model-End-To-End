"use client";

import React, { useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { StatusBadge } from "@/components/StatusBadge";
import { Box, Plus, Tag, ArrowRight, CheckCircle, Database } from "lucide-react";

export default function ModelRegistryPage() {
  const [models, setModels] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [stageFilter, setStageFilter] = useState<string>("all");
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [newModel, setNewModel] = useState({
    name: "",
    framework: "vLLM / PyTorch",
    task: "Text Generation / RAG",
    stage: "Development",
    version: "v1.0.0",
    parameters_count: "70B",
    artifact_uri: "s3://ml-artifacts/model-v1",
    description: "",
  });

  const loadModels = async () => {
    try {
      setLoading(true);
      const url = stageFilter === "all" ? "/models" : `/models?stage=${stageFilter}`;
      const data = await fetchApi<any[]>(url);
      setModels(data);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadModels();
  }, [stageFilter]);

  const handleUpdateStage = async (id: string, currentStage: string) => {
    const nextStage =
      currentStage === "Development"
        ? "Staging"
        : currentStage === "Staging"
        ? "Production"
        : "Archived";

    try {
      await fetchApi(`/models/${id}/stage`, {
        method: "PATCH",
        body: JSON.stringify({ stage: nextStage }),
      });
      loadModels();
    } catch (err: any) {
      alert(`Failed to update stage: ${err.message}`);
    }
  };

  const handleCreateModel = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await fetchApi("/models", {
        method: "POST",
        body: JSON.stringify(newModel),
      });
      setIsModalOpen(false);
      loadModels();
    } catch (err: any) {
      alert(`Failed to create model: ${err.message}`);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <Box className="w-5 h-5 text-indigo-400" />
            Model Registry & Lifecycle
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Track weights, deployment stages, parameter architectures, and artifact lineages.
          </p>
        </div>
        <div className="flex items-center gap-3">
          <select
            value={stageFilter}
            onChange={(e) => setStageFilter(e.target.value)}
            className="bg-slate-900 border border-slate-800 text-xs text-slate-300 rounded-lg px-3 py-1.5 focus:outline-none focus:border-indigo-500"
          >
            <option value="all">All Stages</option>
            <option value="Production">Production</option>
            <option value="Staging">Staging</option>
            <option value="Development">Development</option>
            <option value="Archived">Archived</option>
          </select>
          <button
            onClick={() => setIsModalOpen(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold shadow-sm transition"
          >
            <Plus className="w-4 h-4" />
            Register Model
          </button>
        </div>
      </div>

      {/* Models List */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/40 overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-slate-950/60 text-slate-400 uppercase font-semibold border-b border-slate-800">
              <tr>
                <th className="py-3 px-4">Model & Task</th>
                <th className="py-3 px-4">Framework / Size</th>
                <th className="py-3 px-4">Version</th>
                <th className="py-3 px-4">Stage</th>
                <th className="py-3 px-4">Artifact URI</th>
                <th className="py-3 px-4 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              {loading ? (
                <tr>
                  <td colSpan={6} className="py-8 text-center text-slate-500">
                    Loading registered models from SQLite/Rust backend...
                  </td>
                </tr>
              ) : models.length === 0 ? (
                <tr>
                  <td colSpan={6} className="py-8 text-center text-slate-500">
                    No models registered in this stage.
                  </td>
                </tr>
              ) : (
                models.map((m) => (
                  <tr key={m.id} className="hover:bg-slate-800/30 transition">
                    <td className="py-3.5 px-4">
                      <div className="font-semibold text-white">{m.name}</div>
                      <div className="text-[11px] text-slate-400">{m.task}</div>
                    </td>
                    <td className="py-3.5 px-4">
                      <span className="font-mono text-indigo-300 bg-indigo-950/50 px-2 py-0.5 rounded border border-indigo-900/50">
                        {m.parameters_count || "N/A"}
                      </span>
                      <div className="text-[11px] text-slate-400 mt-0.5">{m.framework}</div>
                    </td>
                    <td className="py-3.5 px-4 font-mono font-medium text-slate-200">
                      {m.version}
                    </td>
                    <td className="py-3.5 px-4">
                      <StatusBadge status={m.stage} />
                    </td>
                    <td className="py-3.5 px-4">
                      <span className="font-mono text-[11px] text-slate-400 truncate max-w-[220px] block" title={m.artifact_uri}>
                        {m.artifact_uri || "local://weights"}
                      </span>
                    </td>
                    <td className="py-3.5 px-4 text-right">
                      {m.stage !== "Archived" && (
                        <button
                          onClick={() => handleUpdateStage(m.id, m.stage)}
                          className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium border border-slate-750 transition"
                        >
                          Promote
                          <ArrowRight className="w-3 h-3 text-indigo-400" />
                        </button>
                      )}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Register Model Modal */}
      {isModalOpen && (
        <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl max-w-md w-full p-6 space-y-4 shadow-2xl">
            <h3 className="text-base font-bold text-white flex items-center gap-2">
              <Box className="w-4 h-4 text-indigo-400" />
              Register New Model Version
            </h3>
            <form onSubmit={handleCreateModel} className="space-y-3 text-xs">
              <div>
                <label className="block text-slate-400 mb-1">Model Name</label>
                <input
                  required
                  value={newModel.name}
                  onChange={(e) => setNewModel({ ...newModel, name: e.target.value })}
                  placeholder="e.g. Llama-3.3-70B-Instruct"
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-slate-400 mb-1">Framework</label>
                  <input
                    value={newModel.framework}
                    onChange={(e) => setNewModel({ ...newModel, framework: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                  />
                </div>
                <div>
                  <label className="block text-slate-400 mb-1">Parameters (e.g. 70B)</label>
                  <input
                    value={newModel.parameters_count}
                    onChange={(e) => setNewModel({ ...newModel, parameters_count: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                  />
                </div>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-slate-400 mb-1">Version</label>
                  <input
                    value={newModel.version}
                    onChange={(e) => setNewModel({ ...newModel, version: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                  />
                </div>
                <div>
                  <label className="block text-slate-400 mb-1">Stage</label>
                  <select
                    value={newModel.stage}
                    onChange={(e) => setNewModel({ ...newModel, stage: e.target.value })}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                  >
                    <option value="Development">Development</option>
                    <option value="Staging">Staging</option>
                    <option value="Production">Production</option>
                  </select>
                </div>
              </div>
              <div>
                <label className="block text-slate-400 mb-1">Artifact S3/Blob URI</label>
                <input
                  value={newModel.artifact_uri}
                  onChange={(e) => setNewModel({ ...newModel, artifact_uri: e.target.value })}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white font-mono text-[11px] focus:outline-none focus:border-indigo-500"
                />
              </div>
              <div>
                <label className="block text-slate-400 mb-1">Description</label>
                <textarea
                  value={newModel.description}
                  onChange={(e) => setNewModel({ ...newModel, description: e.target.value })}
                  rows={2}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                />
              </div>
              <div className="flex justify-end gap-2 pt-3 border-t border-slate-800">
                <button
                  type="button"
                  onClick={() => setIsModalOpen(false)}
                  className="px-3 py-1.5 bg-slate-800 text-slate-300 rounded-lg text-xs"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold"
                >
                  Save Model
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
