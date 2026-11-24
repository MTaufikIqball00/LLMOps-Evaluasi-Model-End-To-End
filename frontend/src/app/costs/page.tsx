"use client";

import React, { useCallback, useEffect, useState } from "react";
import { fetchApi } from "@/lib/api";
import { MetricCard } from "@/components/MetricCard";
import {
  Coins,
  ShieldCheck,
  DollarSign,
  TrendingUp,
  Sliders,
  GitCompare,
  ArrowDownRight,
  ArrowUpRight,
  RefreshCw,
} from "lucide-react";

export default function CostsPage() {
  const [summary, setSummary] = useState<any>(null);
  const [comparison, setComparison] = useState<any>(null);
  const [pricing, setPricing] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [isEditingBudget, setIsEditingBudget] = useState(false);
  const [budgetForm, setBudgetForm] = useState({
    monthly_budget_usd: 2500,
    alert_threshold_pct: 80,
  });

  const loadCosts = useCallback(async () => {
    try {
      setLoading(true);
      const [sum, price, comp] = await Promise.all([
        fetchApi<any>("/costs/summary"),
        fetchApi<any[]>("/costs/pricing"),
        fetchApi<any>("/costs/comparison").catch(() => null),
      ]);
      setSummary(sum);
      setPricing(price);
      setComparison(comp);
      if (sum?.budget) {
        setBudgetForm({
          monthly_budget_usd: sum.budget.monthly_budget_usd,
          alert_threshold_pct: sum.budget.alert_threshold_pct,
        });
      }
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadCosts();
  }, [loadCosts]);

  const handleUpdateBudget = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await fetchApi("/costs/budget", {
        method: "PATCH",
        body: JSON.stringify(budgetForm),
      });
      setIsEditingBudget(false);
      loadCosts();
    } catch (err: any) {
      alert(`Failed to update budget: ${err.message}`);
    }
  };

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <Coins className="w-5 h-5 text-indigo-400" />
            Token Usage & Cost Tracking
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Standard pricing catalogs, provider attribution, spend projections, and automated budget alert thresholds.
          </p>
        </div>
        <button
          onClick={() => setIsEditingBudget(!isEditingBudget)}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-slate-900 hover:bg-slate-850 border border-slate-800 text-slate-300 rounded-lg text-xs transition"
        >
          <Sliders className="w-3.5 h-3.5 text-indigo-400" />
          Configure Budget Limits
        </button>
      </div>

      {/* Budget Editor Panel */}
      {isEditingBudget && (
        <form
          onSubmit={handleUpdateBudget}
          className="rounded-xl border border-indigo-900/60 bg-indigo-950/20 p-5 space-y-4"
        >
          <h3 className="text-sm font-semibold text-white">Update Monthly Budget & Alert Trigger</h3>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
            <div>
              <label className="block text-slate-400 mb-1">Monthly Budget Limit ($ USD)</label>
              <input
                type="number"
                min="100"
                step="50"
                value={budgetForm.monthly_budget_usd}
                onChange={(e) =>
                  setBudgetForm({ ...budgetForm, monthly_budget_usd: parseFloat(e.target.value) || 0 })
                }
                className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white font-mono focus:outline-none focus:border-indigo-500"
              />
            </div>
            <div>
              <label className="block text-slate-400 mb-1">Alert Threshold (% of budget)</label>
              <input
                type="number"
                min="10"
                max="100"
                value={budgetForm.alert_threshold_pct}
                onChange={(e) =>
                  setBudgetForm({ ...budgetForm, alert_threshold_pct: parseFloat(e.target.value) || 0 })
                }
                className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white font-mono focus:outline-none focus:border-indigo-500"
              />
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={() => setIsEditingBudget(false)}
              className="px-3 py-1.5 bg-slate-800 text-slate-300 rounded-lg text-xs"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold"
            >
              Save Budget Limits
            </button>
          </div>
        </form>
      )}

      {/* KPI Cards Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="Current Month Spend"
          value={`$${summary?.budget?.current_spend_usd?.toFixed(2) || "0.00"}`}
          subtitle={`Spend threshold: ${summary?.budget?.alert_threshold_pct || 80}%`}
          icon={DollarSign}
          alert={summary?.is_budget_alert_active}
          source="Pricing Engine"
        />
        <MetricCard
          title="Monthly Budget"
          value={`$${summary?.budget?.monthly_budget_usd?.toFixed(2) || "0.00"}`}
          subtitle="Fixed enterprise cap"
          icon={Coins}
          source="Configured"
        />
        <MetricCard
          title="Budget Consumed"
          value={`${summary?.budget ? ((summary.budget.current_spend_usd / summary.budget.monthly_budget_usd) * 100).toFixed(1) : 0}%`}
          subtitle={`Alert at ${(summary?.budget?.monthly_budget_usd * (summary?.budget?.alert_threshold_pct / 100)).toFixed(0)} USD`}
          icon={TrendingUp}
          alert={summary?.is_budget_alert_active}
          source="Active Threshold"
        />
        <MetricCard
          title="Projected End-of-Month"
          value={`$${summary?.monthly_spend_projection_usd?.toFixed(2) || "0.00"}`}
          subtitle="Linear usage extrapolation"
          icon={TrendingUp}
          source="Forecast Model"
        />
      </div>

      {/* Before vs After Token Efficiency Comparison Panel */}
      {comparison && (
        <div className="rounded-xl border border-slate-800 bg-slate-900/50 p-5 space-y-5">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
            <div>
              <h3 className="text-sm font-semibold text-white flex items-center gap-2">
                <GitCompare className="w-4 h-4 text-cyan-400" />
                Token Efficiency Comparison: Tanpa Program vs Dengan LLMOps Control Guard
              </h3>
              <p className="text-[11px] text-slate-400 mt-0.5">
                Sumber data: <span className="text-slate-300 font-medium">{comparison.data_source}</span>
                {comparison.cutoff_timestamp && (
                  <>
                    {" "}• Cutoff aktif:{" "}
                    <span className="font-mono text-slate-300">
                      {new Date(comparison.cutoff_timestamp).toLocaleString()}
                    </span>
                  </>
                )}
              </p>
            </div>
            <button
              type="button"
              onClick={loadCosts}
              disabled={loading}
              className="self-start sm:self-auto flex items-center gap-1.5 px-3 py-1.5 bg-slate-950 hover:bg-slate-800 border border-slate-800 text-slate-300 rounded-lg text-xs transition"
            >
              <RefreshCw className={`w-3.5 h-3.5 text-cyan-400 ${loading ? "animate-spin" : ""}`} />
              Refresh Comparison
            </button>
          </div>

          {/* Side-by-Side Cohort Cards */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Baseline (Without Program) */}
            <div className="rounded-lg border border-slate-800 bg-slate-950/60 p-4 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold text-slate-200">
                  {comparison.without_program?.label || "Tanpa Program (Baseline)"}
                </span>
                <span className="text-[10px] px-2 py-0.5 rounded-full bg-slate-800 text-slate-300 font-mono">
                  {comparison.without_program?.sessions_count || 0} sesi •{" "}
                  {comparison.without_program?.turns_count || 0} giliran
                </span>
              </div>
              <div className="grid grid-cols-2 gap-3 text-xs">
                <div>
                  <div className="text-[10px] text-slate-500 uppercase">Total Token</div>
                  <div className="text-base font-mono font-bold text-white">
                    {(comparison.without_program?.total_tokens || 0).toLocaleString()}
                  </div>
                </div>
                <div>
                  <div className="text-[10px] text-slate-500 uppercase">Rata-rata / Giliran</div>
                  <div className="text-base font-mono font-bold text-slate-200">
                    {Math.round(comparison.without_program?.avg_tokens_per_turn || 0).toLocaleString()}
                  </div>
                </div>
                <div>
                  <div className="text-[10px] text-slate-500 uppercase">Rata-rata Output / Giliran</div>
                  <div className="text-sm font-mono font-semibold text-amber-300">
                    {Math.round(comparison.without_program?.avg_output_tokens_per_turn || 0).toLocaleString()} tok
                  </div>
                </div>
                <div>
                  <div className="text-[10px] text-slate-500 uppercase">Estimasi Biaya</div>
                  <div className="text-sm font-mono font-semibold text-slate-300">
                    ${(comparison.without_program?.estimated_cost_usd || 0).toFixed(4)}
                  </div>
                </div>
              </div>
              <div className="text-[10px] text-slate-500 font-mono pt-1 border-t border-slate-800/70">
                Input: {(comparison.without_program?.prompt_tokens || 0).toLocaleString()} • Output:{" "}
                {(comparison.without_program?.completion_tokens || 0).toLocaleString()} • Cache:{" "}
                {(comparison.without_program?.cache_read_tokens || 0).toLocaleString()}
              </div>
            </div>

            {/* Guarded (With Program) */}
            <div className="rounded-lg border border-indigo-900/60 bg-indigo-950/20 p-4 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold text-indigo-200">
                  {comparison.with_program?.label || "Dengan LLMOps Control Guard"}
                </span>
                <span className="text-[10px] px-2 py-0.5 rounded-full bg-indigo-900/60 text-indigo-200 font-mono">
                  {comparison.with_program?.sessions_count || 0} sesi •{" "}
                  {comparison.with_program?.turns_count || 0} giliran
                </span>
              </div>
              <div className="grid grid-cols-2 gap-3 text-xs">
                <div>
                  <div className="text-[10px] text-slate-400 uppercase">Total Token</div>
                  <div className="text-base font-mono font-bold text-white">
                    {(comparison.with_program?.total_tokens || 0).toLocaleString()}
                  </div>
                </div>
                <div>
                  <div className="text-[10px] text-slate-400 uppercase">Rata-rata / Giliran</div>
                  <div className="text-base font-mono font-bold text-cyan-300">
                    {Math.round(comparison.with_program?.avg_tokens_per_turn || 0).toLocaleString()}
                  </div>
                </div>
                <div>
                  <div className="text-[10px] text-slate-400 uppercase">Rata-rata Output / Giliran</div>
                  <div className="text-sm font-mono font-semibold text-emerald-400">
                    {Math.round(comparison.with_program?.avg_output_tokens_per_turn || 0).toLocaleString()} tok
                  </div>
                </div>
                <div>
                  <div className="text-[10px] text-slate-400 uppercase">Estimasi Biaya</div>
                  <div className="text-sm font-mono font-semibold text-emerald-300">
                    ${(comparison.with_program?.estimated_cost_usd || 0).toFixed(4)}
                  </div>
                </div>
              </div>
              <div className="text-[10px] text-slate-400 font-mono pt-1 border-t border-indigo-900/40">
                Input: {(comparison.with_program?.prompt_tokens || 0).toLocaleString()} • Output:{" "}
                {(comparison.with_program?.completion_tokens || 0).toLocaleString()} • Cache:{" "}
                {(comparison.with_program?.cache_read_tokens || 0).toLocaleString()}
              </div>
            </div>
          </div>

          {/* Delta Summary Strip */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
            <div className="rounded-lg border border-slate-800 bg-slate-950/50 p-3 flex items-center justify-between">
              <div>
                <div className="text-[10px] text-slate-400 uppercase">Efisiensi Output Token / Giliran</div>
                <div className="text-slate-300 mt-0.5 font-mono">
                  {comparison.avg_output_tokens_saved_per_turn >= 0 ? "Hemat " : "Selisih +"}
                  {Math.abs(Math.round(comparison.avg_output_tokens_saved_per_turn || 0)).toLocaleString()} tok/giliran
                </div>
              </div>
              {comparison.output_token_delta_pct !== null && comparison.output_token_delta_pct !== undefined ? (
                <span
                  className={`inline-flex items-center gap-1 px-2.5 py-1 rounded-full font-mono font-semibold ${
                    comparison.output_token_delta_pct >= 0
                      ? "bg-emerald-950/70 text-emerald-400 border border-emerald-800/60"
                      : "bg-amber-950/70 text-amber-300 border border-amber-800/60"
                  }`}
                >
                  {comparison.output_token_delta_pct >= 0 ? (
                    <ArrowDownRight className="w-3.5 h-3.5" />
                  ) : (
                    <ArrowUpRight className="w-3.5 h-3.5" />
                  )}
                  {comparison.output_token_delta_pct >= 0
                    ? `-${comparison.output_token_delta_pct}%`
                    : `+${Math.abs(comparison.output_token_delta_pct)}%`}
                </span>
              ) : (
                <span className="text-[11px] text-slate-500 font-mono">Menunggu data</span>
              )}
            </div>

            <div className="rounded-lg border border-slate-800 bg-slate-950/50 p-3 flex items-center justify-between">
              <div>
                <div className="text-[10px] text-slate-400 uppercase">Efisiensi Total Token / Giliran</div>
                <div className="text-slate-300 mt-0.5 font-mono">
                  {comparison.avg_tokens_saved_per_turn >= 0 ? "Hemat " : "Selisih +"}
                  {Math.abs(Math.round(comparison.avg_tokens_saved_per_turn || 0)).toLocaleString()} tok/giliran
                </div>
              </div>
              {comparison.total_token_delta_pct !== null && comparison.total_token_delta_pct !== undefined ? (
                <span
                  className={`inline-flex items-center gap-1 px-2.5 py-1 rounded-full font-mono font-semibold ${
                    comparison.total_token_delta_pct >= 0
                      ? "bg-emerald-950/70 text-emerald-400 border border-emerald-800/60"
                      : "bg-amber-950/70 text-amber-300 border border-amber-800/60"
                  }`}
                >
                  {comparison.total_token_delta_pct >= 0 ? (
                    <ArrowDownRight className="w-3.5 h-3.5" />
                  ) : (
                    <ArrowUpRight className="w-3.5 h-3.5" />
                  )}
                  {comparison.total_token_delta_pct >= 0
                    ? `-${comparison.total_token_delta_pct}%`
                    : `+${Math.abs(comparison.total_token_delta_pct)}%`}
                </span>
              ) : (
                <span className="text-[11px] text-slate-500 font-mono">Menunggu data</span>
              )}
            </div>
          </div>

          {comparison.comparison_note && (
            <div className="text-[11px] text-slate-400 bg-slate-950/40 border border-slate-800/70 rounded-lg px-3 py-2">
              {comparison.comparison_note}
            </div>
          )}
        </div>
      )}

      {/* Provider Breakdown & Model Breakdown Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Provider Attribution */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 space-y-4">
          <h3 className="text-sm font-semibold text-white flex items-center gap-2">
            <Coins className="w-4 h-4 text-indigo-400" />
            Spend Breakdown by Inference Provider
          </h3>
          <div className="space-y-3">
            {summary?.provider_breakdown?.map((p: any) => (
              <div key={p.provider} className="text-xs">
                <div className="flex justify-between items-center mb-1 text-slate-300">
                  <span className="font-medium text-white">{p.provider}</span>
                  <span className="font-mono text-emerald-400 font-semibold">
                    ${p.estimated_cost_usd?.toFixed(4)} ({p.percentage}%)
                  </span>
                </div>
                <div className="h-1.5 w-full bg-slate-800 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-indigo-500 rounded-full"
                    style={{ width: `${p.percentage}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Pricing Catalog */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/40 p-5 space-y-4">
          <h3 className="text-sm font-semibold text-white flex items-center justify-between">
            <span className="flex items-center gap-2">
              <ShieldCheck className="w-4 h-4 text-emerald-400" />
              Standard Token Pricing Catalog
            </span>
            <span className="text-[10px] text-slate-400 font-normal">Active Rates</span>
          </h3>

          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs">
              <thead className="bg-slate-950/60 text-slate-400 uppercase font-semibold border-b border-slate-800">
                <tr>
                  <th className="py-2 px-3">Provider</th>
                  <th className="py-2 px-3">Model</th>
                  <th className="py-2 px-3">Prompt / 1k</th>
                  <th className="py-2 px-3">Completion / 1k</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60 text-slate-300">
                {pricing.map((pr) => (
                  <tr key={pr.id} className="hover:bg-slate-800/20">
                    <td className="py-2.5 px-3 text-slate-400">{pr.provider}</td>
                    <td className="py-2.5 px-3 font-mono font-medium text-white">{pr.model_pattern}</td>
                    <td className="py-2.5 px-3 font-mono text-cyan-300">${pr.prompt_cost_per_1k}</td>
                    <td className="py-2.5 px-3 font-mono text-indigo-300">${pr.completion_cost_per_1k}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}
