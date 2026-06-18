import { useEffect, useState } from "react";
import { Pin, PinOff, RefreshCw } from "lucide-react";
import { Heatmap } from "./Heatmap";
import { LimitBar } from "./LimitBar";
import { formatTokens, getFreshness } from "../lib/format";
import type { HeatmapMode } from "../lib/heatmap";
import type { DashboardState } from "../types";

interface DashboardWidgetProps {
  state: DashboardState;
  pinned: boolean;
  refreshing?: boolean;
  onRefresh: () => void;
  onTogglePin: () => void;
}

export function DashboardWidget({
  state,
  pinned,
  refreshing = false,
  onRefresh,
  onTogglePin,
}: DashboardWidgetProps) {
  const [heatmapMode, setHeatmapMode] = useState<HeatmapMode>("daily");
  const [now, setNow] = useState(() => new Date());
  const freshness = state.updatedAt ? getFreshness(state.updatedAt, now) : state.freshness;

  useEffect(() => {
    setNow(new Date());
    const id = window.setInterval(() => {
      setNow(new Date());
    }, 1000);

    return () => window.clearInterval(id);
  }, [state.updatedAt]);

  return (
    <main className="widget-shell">
      <header className="widget-header">
        <div className="drag-zone" data-tauri-drag-region>
          <h1 data-tauri-drag-region>Codex Usage</h1>
          <p className={`freshness freshness-${freshness.state}`} data-tauri-drag-region>
            <span className="freshness-dot" />
            {freshness.label}
          </p>
        </div>
        <div className="header-actions">
          <button
            aria-label="Refresh usage data"
            className="icon-button"
            disabled={refreshing}
            type="button"
            onClick={onRefresh}
            title="Refresh"
          >
            <RefreshCw size={16} />
          </button>
          <button
            aria-label={pinned ? "Disable always on top" : "Enable always on top"}
            className={`icon-button ${pinned ? "icon-button-active" : ""}`}
            type="button"
            onClick={onTogglePin}
            title={pinned ? "Disable always on top" : "Enable always on top"}
          >
            {pinned ? <PinOff size={16} /> : <Pin size={16} />}
          </button>
        </div>
      </header>

      <section className="heatmap-panel">
        <div className="activity-header">
          <h2>Token activity</h2>
          <div className="activity-tabs" aria-label="Activity metric">
            {activityTabs.map((tab) => (
              <button
                aria-label={`${tab.label} token activity`}
                aria-pressed={heatmapMode === tab.mode}
                className={heatmapMode === tab.mode ? "activity-tab activity-tab-active" : "activity-tab"}
                key={tab.mode}
                type="button"
                onClick={() => setHeatmapMode(tab.mode)}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>
        <Heatmap days={state.heatmapDays} mode={heatmapMode} todayDate={state.today.date} />
      </section>

      <section className="summary-strip">
        <span>Today</span>
        <strong>{formatTokens(state.today.totalTokens)} tokens</strong>
        <span>{freshness.label}</span>
      </section>

      <section className="limits-panel">
        <LimitBar limit={state.limits.fiveHour} />
        <LimitBar limit={state.limits.weekly} />
      </section>

    </main>
  );
}

const activityTabs: Array<{ label: string; mode: HeatmapMode }> = [
  { label: "Daily", mode: "daily" },
  { label: "Weekly", mode: "weekly" },
  { label: "Cumulative", mode: "cumulative" },
];
