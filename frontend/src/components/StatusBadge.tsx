import React from "react";

interface StatusBadgeProps {
  status: string;
  size?: "sm" | "md";
}

export function StatusBadge({ status, size = "sm" }: StatusBadgeProps) {
  const norm = status.toLowerCase();

  let colorClasses = "bg-slate-800 text-slate-300 border-slate-700";

  if (norm.includes("health") || norm.includes("success") || norm.includes("prod") || norm.includes("completed")) {
    colorClasses = "bg-emerald-950/70 text-emerald-300 border-emerald-800/60";
  } else if (norm.includes("stag") || norm.includes("run") || norm.includes("progress")) {
    colorClasses = "bg-amber-950/70 text-amber-300 border-amber-800/60";
  } else if (norm.includes("dev") || norm.includes("queue") || norm.includes("config")) {
    colorClasses = "bg-cyan-950/70 text-cyan-300 border-cyan-800/60";
  } else if (norm.includes("fail") || norm.includes("degraded") || norm.includes("err") || norm.includes("disc") || norm.includes("offline")) {
    colorClasses = "bg-rose-950/70 text-rose-300 border-rose-800/60";
  } else if (norm.includes("arch")) {
    colorClasses = "bg-slate-900 text-slate-400 border-slate-800";
  }

  const padding = size === "sm" ? "px-2 py-0.5 text-xs" : "px-2.5 py-1 text-sm";

  return (
    <span
      className={`inline-flex items-center gap-1.5 font-medium rounded-md border ${padding} ${colorClasses}`}
    >
      <span className="w-1.5 h-1.5 rounded-full bg-current opacity-80 animate-pulse" />
      {status}
    </span>
  );
}
