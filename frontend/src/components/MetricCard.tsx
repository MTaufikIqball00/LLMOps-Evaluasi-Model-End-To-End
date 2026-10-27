import React from "react";
import { LucideIcon } from "lucide-react";

interface MetricCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  icon?: LucideIcon;
  trend?: {
    value: string;
    isPositive: boolean;
  };
  source?: string;
  alert?: boolean;
}

export function MetricCard({
  title,
  value,
  subtitle,
  icon: Icon,
  trend,
  source = "Live API",
  alert = false,
}: MetricCardProps) {
  return (
    <div
      className={`rounded-xl border p-4 bg-slate-900/60 backdrop-blur-sm transition-all hover:border-slate-700 ${
        alert ? "border-amber-500/50 bg-amber-950/10" : "border-slate-800"
      }`}
    >
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium uppercase tracking-wider text-slate-400">
          {title}
        </span>
        {Icon && (
          <div className="p-1.5 rounded-lg bg-slate-800/80 text-slate-300">
            <Icon className="w-4 h-4" />
          </div>
        )}
      </div>

      <div className="mt-3 flex items-baseline gap-2">
        <span className="text-2xl font-bold tracking-tight text-white">{value}</span>
        {trend && (
          <span
            className={`text-xs font-semibold ${
              trend.isPositive ? "text-emerald-400" : "text-rose-400"
            }`}
          >
            {trend.value}
          </span>
        )}
      </div>

      <div className="mt-2 flex items-center justify-between text-xs text-slate-400">
        <span>{subtitle}</span>
        <span className="text-[10px] px-1.5 py-0.5 rounded bg-slate-800/50 text-slate-400 border border-slate-750">
          {source}
        </span>
      </div>
    </div>
  );
}
