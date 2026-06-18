import { describe, expect, it } from "vitest";
import { buildCalendarHeatmap, buildHeatmapDays } from "./heatmap";
import type { HeatmapDay } from "../types";

describe("buildHeatmapDays", () => {
  it("keeps the latest 14 days and assigns relative intensity levels", () => {
    const days: HeatmapDay[] = [
      { date: "2026-06-01", totalTokens: 0, sessions: 0 },
      { date: "2026-06-02", totalTokens: 5_000, sessions: 1 },
      { date: "2026-06-03", totalTokens: 25_000, sessions: 1 },
      { date: "2026-06-04", totalTokens: 250_000, sessions: 2 },
      { date: "2026-06-05", totalTokens: 1_000_000, sessions: 4 },
      { date: "2026-06-06", totalTokens: 10_000, sessions: 1 },
      { date: "2026-06-07", totalTokens: 0, sessions: 0 },
      { date: "2026-06-08", totalTokens: 50_000, sessions: 1 },
      { date: "2026-06-09", totalTokens: 75_000, sessions: 2 },
      { date: "2026-06-10", totalTokens: 100_000, sessions: 2 },
      { date: "2026-06-11", totalTokens: 200_000, sessions: 3 },
      { date: "2026-06-12", totalTokens: 300_000, sessions: 3 },
      { date: "2026-06-13", totalTokens: 400_000, sessions: 3 },
      { date: "2026-06-14", totalTokens: 500_000, sessions: 4 },
      { date: "2026-06-15", totalTokens: 750_000, sessions: 4 },
    ];

    const result = buildHeatmapDays(days, 14);

    expect(result).toHaveLength(14);
    expect(result[0].date).toBe("2026-06-02");
    expect(result[result.length - 1]?.date).toBe("2026-06-15");
    expect(result.find((day) => day.totalTokens === 0)?.level).toBe(0);
    expect(result.find((day) => day.totalTokens === 1_000_000)?.level).toBe(4);
  });

  it("fills an empty set with zero-level days", () => {
    const result = buildHeatmapDays([], 3, new Date("2026-06-17T12:00:00.000Z"));

    expect(result).toEqual([
      { date: "2026-06-15", totalTokens: 0, sessions: 0, level: 0 },
      { date: "2026-06-16", totalTokens: 0, sessions: 0, level: 0 },
      { date: "2026-06-17", totalTokens: 0, sessions: 0, level: 0 },
    ]);
  });
});

describe("buildCalendarHeatmap", () => {
  it("builds a twelve month week grid with month labels", () => {
    const result = buildCalendarHeatmap(
      [
        { date: "2026-06-15", totalTokens: 1_000_000, sessions: 2 },
        { date: "2026-06-17", totalTokens: 50_000, sessions: 1 },
      ],
      new Date("2026-06-17T12:00:00.000Z"),
      12,
    );

    expect(result.weeks.length).toBeGreaterThanOrEqual(52);
    expect(result.weeks[result.weeks.length - 1][3].date).toBe("2026-06-17");
    expect(result.weeks.flat().find((day) => day.date === "2026-06-15")?.level).toBe(4);
    expect(result.weekCount).toBe(result.weeks.length);
    expect(result.monthLabels.map((label) => label.label)).toEqual([
      "Jul",
      "Aug",
      "Sep",
      "Oct",
      "Nov",
      "Dec",
      "Jan",
      "Feb",
      "Mar",
      "Apr",
      "May",
      "Jun",
    ]);
  });

  it("switches weekly and cumulative token metrics", () => {
    const days: HeatmapDay[] = [
      { date: "2026-06-14", totalTokens: 100, sessions: 1 },
      { date: "2026-06-15", totalTokens: 200, sessions: 1 },
      { date: "2026-06-16", totalTokens: 300, sessions: 1 },
    ];

    const weekly = buildCalendarHeatmap(days, new Date("2026-06-16T12:00:00.000Z"), 12, "weekly");
    const cumulative = buildCalendarHeatmap(days, new Date("2026-06-16T12:00:00.000Z"), 12, "cumulative");

    expect(weekly.weeks.flat().find((day) => day.date === "2026-06-14")?.totalTokens).toBe(600);
    expect(weekly.weeks.flat().find((day) => day.date === "2026-06-15")?.totalTokens).toBe(600);
    expect(weekly.weeks.flat().find((day) => day.date === "2026-06-16")?.totalTokens).toBe(600);
    expect(cumulative.weeks.flat().find((day) => day.date === "2026-06-16")?.totalTokens).toBe(600);
  });
});
