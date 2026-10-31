"use client";

import React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  Box,
  FlaskConical,
  CheckCircle2,
  Activity,
  ListTree,
  Coins,
  Server,
  GitBranch,
  Radio,
  Cpu,
  Layers,
  ShieldCheck,
} from "lucide-react";

const NAV_ITEMS = [
  { href: "/", label: "Overview", icon: LayoutDashboard },
  { href: "/models", label: "Model Registry", icon: Box },
  { href: "/experiments", label: "Experiments", icon: FlaskConical },
  { href: "/evaluations", label: "LLM Evaluations", icon: CheckCircle2 },
  { href: "/monitoring", label: "Production Monitoring", icon: Activity },
  { href: "/traces", label: "Traces & Requests", icon: ListTree },
  { href: "/costs", label: "Token Usage & Costs", icon: Coins },
  { href: "/deployments", label: "Deployments", icon: Server },
  { href: "/cicd", label: "CI/CD Pipelines", icon: GitBranch },
  { href: "/integrations", label: "Integrations", icon: Radio },
  { href: "/system", label: "System Health", icon: Cpu },
  { href: "/qa", label: "QA & Audit Engine", icon: ShieldCheck },
];

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="w-64 border-r border-slate-800 bg-slate-950/80 backdrop-blur-xl flex flex-col shrink-0 h-screen sticky top-0">
      {/* Brand Header */}
      <div className="h-16 border-b border-slate-800 flex items-center px-5 gap-3">
        <div className="w-8 h-8 rounded-lg bg-indigo-600 flex items-center justify-center text-white shadow-lg shadow-indigo-500/20">
          <Layers className="w-5 h-5" />
        </div>
        <div>
          <h1 className="text-sm font-bold tracking-tight text-white flex items-center gap-1.5">
            LLMOps Center
            <span className="text-[10px] font-semibold px-1.5 py-0.2 rounded bg-indigo-950 text-indigo-400 border border-indigo-800/60">
              PRO
            </span>
          </h1>
          <p className="text-[11px] text-slate-400">Enterprise AI Platform</p>
        </div>
      </div>

      {/* Navigation Links */}
      <div className="flex-1 overflow-y-auto px-3 py-4 space-y-1">
        <div className="px-3 pb-2 text-[10px] font-semibold uppercase tracking-wider text-slate-400">
          Core Workflows
        </div>
        {NAV_ITEMS.map((item) => {
          const isActive =
            item.href === "/"
              ? pathname === "/"
              : pathname.startsWith(item.href);
          const Icon = item.icon;

          return (
            <Link
              key={item.href}
              href={item.href}
              className={`flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                isActive
                  ? "bg-indigo-600/15 text-indigo-400 border border-indigo-500/30"
                  : "text-slate-400 hover:text-slate-200 hover:bg-slate-900/60"
              }`}
            >
              <Icon
                className={`w-4 h-4 shrink-0 ${
                  isActive ? "text-indigo-400" : "text-slate-400"
                }`}
              />
              <span className="truncate">{item.label}</span>
            </Link>
          );
        })}
      </div>

      {/* Port & Runtime Isolation Status */}
      <div className="p-3 border-t border-slate-800 bg-slate-900/30">
        <div className="rounded-lg p-2.5 bg-slate-950 border border-slate-850 text-xs space-y-1.5">
          <div className="flex items-center justify-between text-slate-400 text-[11px]">
            <span>Rust Backend</span>
            <span className="font-mono text-emerald-400 font-semibold">:8210</span>
          </div>
          <div className="flex items-center justify-between text-slate-400 text-[11px]">
            <span>Next.js Frontend</span>
            <span className="font-mono text-cyan-400 font-semibold">:3210</span>
          </div>
          <div className="pt-1 border-t border-slate-800 flex items-center justify-between text-[10px] text-slate-400">
            <span className="flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
              Ports Isolated
            </span>
            <span>SQLite Active</span>
          </div>
        </div>
      </div>
    </aside>
  );
}
