import React from "react";

interface SparklineProps {
  data: number[];
  color?: string;
  height?: number;
}

export function Sparkline({ data, color = "#6366f1", height = 40 }: SparklineProps) {
  if (!data || data.length < 2) return null;

  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;

  const width = 100;
  const points = data
    .map((val, idx) => {
      const x = (idx / Math.max(1, data.length - 1)) * width;
      const y = height - ((val - min) / range) * (height - 8) - 4;
      return `${x},${y}`;
    })
    .join(" ");

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="w-full h-full overflow-visible">
      <polyline
        fill="none"
        stroke={color}
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
        points={points}
      />
    </svg>
  );
}

interface TimeseriesChartProps {
  labels: string[];
  series: {
    name: string;
    data: number[];
    color: string;
  }[];
  height?: number;
  unit?: string;
}

export function TimeseriesChart({
  labels,
  series,
  height = 180,
  unit = "ms",
}: TimeseriesChartProps) {
  if (!labels || labels.length === 0 || !series || series.length === 0) {
    return <div className="h-40 flex items-center justify-center text-slate-500 text-sm">No data available</div>;
  }

  const allVals = series.flatMap((s) => s.data);
  const min = Math.min(0, ...allVals);
  const max = Math.max(...allVals) * 1.15 || 100;
  const range = max - min || 1;

  const chartWidth = 500;
  const chartHeight = height;
  const padBottom = 25;
  const usableHeight = chartHeight - padBottom;

  return (
    <div className="w-full">
      <div className="flex items-center gap-4 mb-2 justify-end text-xs">
        {series.map((s) => (
          <div key={s.name} className="flex items-center gap-1.5 text-slate-300">
            <span className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: s.color }} />
            <span>{s.name}</span>
          </div>
        ))}
      </div>

      <div className="relative w-full" style={{ height: `${chartHeight}px` }}>
        <svg
          viewBox={`0 0 ${chartWidth} ${chartHeight}`}
          className="w-full h-full"
          preserveAspectRatio="none"
        >
          {/* Grid lines */}
          {[0, 0.25, 0.5, 0.75, 1].map((pct, idx) => {
            const y = usableHeight * (1 - pct);
            const val = (min + range * pct).toFixed(0);
            return (
              <g key={idx}>
                <line
                  x1="0"
                  y1={y}
                  x2={chartWidth}
                  y2={y}
                  stroke="#1e293b"
                  strokeDasharray="3 3"
                />
                <text
                  x="4"
                  y={y - 4}
                  fill="#64748b"
                  fontSize="9"
                  fontFamily="sans-serif"
                >
                  {val} {unit}
                </text>
              </g>
            );
          })}

          {/* Lines */}
          {series.map((s) => {
            const points = s.data
              .map((val, idx) => {
                const x = (idx / Math.max(1, labels.length - 1)) * chartWidth;
                const y = usableHeight - ((val - min) / range) * usableHeight;
                return `${x},${y}`;
              })
              .join(" ");

            return (
              <polyline
                key={s.name}
                fill="none"
                stroke={s.color}
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
                points={points}
              />
            );
          })}
        </svg>
      </div>

      {/* X Labels */}
      <div className="flex justify-between text-[10px] text-slate-500 mt-1 px-1">
        {labels.map((l, i) => (
          <span key={i} className={i % 2 === 0 ? "opacity-100" : "opacity-40"}>
            {l}
          </span>
        ))}
      </div>
    </div>
  );
}
