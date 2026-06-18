import { buildHeatmapDays } from "./heatmap";
import { getFreshness } from "./format";
import type { DashboardState } from "../types";

export function createFallbackState(sourcePath = "%USERPROFILE%\\.codex\\sessions"): DashboardState {
  const today = new Date();
  const date = [
    today.getFullYear(),
    `${today.getMonth() + 1}`.padStart(2, "0"),
    `${today.getDate()}`.padStart(2, "0"),
  ].join("-");

  return {
    sourcePath,
    updatedAt: null,
    freshness: getFreshness(null),
    warnings: ["Waiting for local Codex session data."],
    limits: {
      fiveHour: { label: "5h", available: false, unusual: false },
      weekly: { label: "Weekly", available: false, unusual: false },
      planType: null,
    },
    today: {
      date,
      totalTokens: 0,
      inputTokens: 0,
      cachedInputTokens: 0,
      outputTokens: 0,
      reasoningTokens: 0,
      sessions: 0,
    },
    heatmapDays: buildHeatmapDays([], 183),
  };
}
