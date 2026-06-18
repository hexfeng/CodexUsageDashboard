import type { CSSProperties } from "react";
import { useState } from "react";
import { formatTokens } from "../lib/format";
import { buildCalendarHeatmap, type HeatmapMode } from "../lib/heatmap";
import type { HeatmapDay } from "../types";

interface HeatmapProps {
  days: HeatmapDay[];
  mode: HeatmapMode;
  todayDate: string;
}

export function Heatmap({ days, mode, todayDate }: HeatmapProps) {
  const [tooltip, setTooltip] = useState<HeatmapTooltip | null>(null);
  const calendar = buildCalendarHeatmap(days, fromDateKey(todayDate), 12, mode);
  const gridStyle = {
    "--week-count": calendar.weekCount,
  } as CSSProperties;

  return (
    <div className="calendar-heatmap" aria-label="12 month token activity heatmap" onPointerLeave={() => setTooltip(null)}>
      <div className="calendar-grid" style={gridStyle}>
        {calendar.weeks.map((week, weekIndex) => (
          <div className="calendar-week" key={`${week[0].date}-${weekIndex}`}>
            {week.map((day) => {
              const tooltip = `${formatTokens(day.totalTokens)} ${modeLabel(mode)} on ${formatTooltipDate(day.date)}`;
              return (
                <div
                  className={`heatmap-cell heatmap-level-${day.level}`}
                  key={day.date}
                  data-tooltip={tooltip}
                  aria-label={`${day.date}: ${formatTokens(day.totalTokens)} ${mode} tokens`}
                  tabIndex={0}
                  onBlur={() => setTooltip(null)}
                  onClick={(event) => setTooltip(positionTooltip(tooltip, event.clientX, event.clientY))}
                  onFocus={(event) => {
                    const rect = event.currentTarget.getBoundingClientRect();
                    setTooltip(positionTooltip(tooltip, rect.left + rect.width / 2, rect.top));
                  }}
                  onPointerEnter={(event) => setTooltip(positionTooltip(tooltip, event.clientX, event.clientY))}
                  onPointerMove={(event) => setTooltip(positionTooltip(tooltip, event.clientX, event.clientY))}
                />
              );
            })}
          </div>
        ))}
      </div>
      <div className="calendar-months" style={gridStyle} aria-hidden="true">
        {calendar.monthLabels.map((label) => (
          <span key={label.label} style={{ gridColumnStart: label.column + 1 }}>
            {label.label}
          </span>
        ))}
      </div>
      {tooltip ? (
        <div className="heatmap-tooltip" style={{ left: tooltip.left, top: tooltip.top }}>
          {tooltip.text}
        </div>
      ) : null}
    </div>
  );
}

interface HeatmapTooltip {
  text: string;
  left: number;
  top: number;
}

function positionTooltip(text: string, clientX: number, clientY: number): HeatmapTooltip {
  const tooltipWidth = Math.min(240, Math.max(128, text.length * 7.2 + 22));
  const margin = 12;
  const left = clamp(clientX, margin + tooltipWidth / 2, window.innerWidth - margin - tooltipWidth / 2);
  const preferredTop = clientY - 14;
  const top = preferredTop < 40 ? clientY + 38 : preferredTop;

  return {
    text,
    left,
    top,
  };
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function modeLabel(mode: HeatmapMode): string {
  if (mode === "daily") {
    return "tokens";
  }

  return `${mode} tokens`;
}

function formatTooltipDate(dateKey: string): string {
  const [year, month, day] = dateKey.split("-").map(Number);
  return new Date(year, month - 1, day).toLocaleString("en-US", {
    month: "short",
    day: "numeric",
  });
}

function fromDateKey(dateKey: string): Date {
  const [year, month, day] = dateKey.split("-").map(Number);
  return new Date(year, month - 1, day);
}
