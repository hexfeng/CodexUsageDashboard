import type { HeatmapDay } from "../types";

export type HeatmapMode = "daily" | "weekly" | "cumulative";

export interface CalendarHeatmap {
  weeks: Required<HeatmapDay>[][];
  weekCount: number;
  monthLabels: Array<{ label: string; column: number }>;
}

export function buildHeatmapDays(
  sourceDays: HeatmapDay[],
  count = 14,
  now = new Date(),
): Required<HeatmapDay>[] {
  const days = sourceDays.length > 0 ? sourceDays.slice(-count) : emptyDays(count, now);
  const max = Math.max(...days.map((day) => day.totalTokens), 0);

  return days.map((day) => ({
    date: day.date,
    totalTokens: day.totalTokens,
    sessions: day.sessions,
    level: getHeatmapLevel(day.totalTokens, max),
  }));
}

export function buildCalendarHeatmap(
  sourceDays: HeatmapDay[],
  now = new Date(),
  months = 6,
  mode: HeatmapMode = "daily",
): CalendarHeatmap {
  const end = startOfDay(now);
  const start = new Date(end);
  start.setMonth(start.getMonth() - (months - 1), 1);
  let gridStart = startOfWeek(start);
  const gridEnd = endOfWeek(end);
  const minWeeks = months * 4 + 4;
  const currentWeeks = Math.floor((gridEnd.getTime() - gridStart.getTime()) / (7 * 86_400_000)) + 1;
  if (currentWeeks < minWeeks) {
    gridStart = new Date(gridStart);
    gridStart.setDate(gridStart.getDate() - (minWeeks - currentWeeks) * 7);
  }
  const byDate = buildMetricDays(sourceDays, gridStart, gridEnd, mode);
  const max = Math.max(...Array.from(byDate.values()).map((day) => day.totalTokens), 0);
  const weeks: Required<HeatmapDay>[][] = [];

  for (let cursor = new Date(gridStart); cursor <= gridEnd; cursor.setDate(cursor.getDate() + 7)) {
    const week: Required<HeatmapDay>[] = [];
    for (let dayIndex = 0; dayIndex < 7; dayIndex += 1) {
      const date = new Date(cursor);
      date.setDate(cursor.getDate() + dayIndex);
      const key = toLocalDateKey(date);
      const value = byDate.get(key);
      const inRange = date >= start && date <= end;
      const totalTokens = inRange ? value?.totalTokens ?? 0 : 0;
      const sessions = inRange ? value?.sessions ?? 0 : 0;
      week.push({
        date: key,
        totalTokens,
        sessions,
        level: getHeatmapLevel(totalTokens, max),
      });
    }
    weeks.push(week);
  }

  return {
    weeks,
    weekCount: weeks.length,
    monthLabels: buildMonthLabels(start, end, gridStart),
  };
}

function buildMetricDays(
  sourceDays: HeatmapDay[],
  gridStart: Date,
  gridEnd: Date,
  mode: HeatmapMode,
): Map<string, HeatmapDay> {
  const daily = new Map(sourceDays.map((day) => [day.date, day]));

  if (mode === "daily") {
    return daily;
  }

  const weeklyTotals = new Map<string, number>();
  const cumulativeTotals = new Map<string, number>();
  let runningTotal = 0;

  for (let cursor = new Date(gridStart); cursor <= gridEnd; cursor.setDate(cursor.getDate() + 1)) {
    const key = toLocalDateKey(cursor);
    const day = daily.get(key);
    const total = day?.totalTokens ?? 0;
    const weekKey = toLocalDateKey(startOfWeek(cursor));
    weeklyTotals.set(weekKey, (weeklyTotals.get(weekKey) ?? 0) + total);
    runningTotal += total;
    cumulativeTotals.set(key, runningTotal);
  }

  const metricDays = new Map<string, HeatmapDay>();
  for (let cursor = new Date(gridStart); cursor <= gridEnd; cursor.setDate(cursor.getDate() + 1)) {
    const key = toLocalDateKey(cursor);
    const day = daily.get(key);
    const weekKey = toLocalDateKey(startOfWeek(cursor));
    metricDays.set(key, {
      date: key,
      totalTokens: mode === "weekly" ? weeklyTotals.get(weekKey) ?? 0 : cumulativeTotals.get(key) ?? 0,
      sessions: day?.sessions ?? 0,
    });
  }

  return metricDays;
}

export function getHeatmapLevel(totalTokens: number, maxTokens: number): number {
  if (totalTokens <= 0 || maxTokens <= 0) {
    return 0;
  }

  const ratio = totalTokens / maxTokens;
  if (ratio <= 0.15) {
    return 1;
  }

  if (ratio <= 0.35) {
    return 2;
  }

  if (ratio <= 0.65) {
    return 3;
  }

  return 4;
}

function emptyDays(count: number, now: Date): HeatmapDay[] {
  const end = toLocalDateKey(now);
  const days: HeatmapDay[] = [];
  const endDate = fromDateKey(end);

  for (let index = count - 1; index >= 0; index -= 1) {
    const date = new Date(endDate);
    date.setDate(endDate.getDate() - index);
    days.push({
      date: toLocalDateKey(date),
      totalTokens: 0,
      sessions: 0,
    });
  }

  return days;
}

function toLocalDateKey(date: Date): string {
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function fromDateKey(dateKey: string): Date {
  const [year, month, day] = dateKey.split("-").map(Number);
  return new Date(year, month - 1, day);
}

function startOfDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

function startOfWeek(date: Date): Date {
  const value = startOfDay(date);
  value.setDate(value.getDate() - value.getDay());
  return value;
}

function endOfWeek(date: Date): Date {
  const value = startOfWeek(date);
  value.setDate(value.getDate() + 6);
  return value;
}

function buildMonthLabels(start: Date, end: Date, gridStart: Date): Array<{ label: string; column: number }> {
  const labels: Array<{ label: string; column: number }> = [];
  const cursor = new Date(start.getFullYear(), start.getMonth(), 1);

  while (cursor <= end) {
    labels.push({
      label: cursor.toLocaleString("en-US", { month: "short" }),
      column: Math.floor((cursor.getTime() - gridStart.getTime()) / (7 * 86_400_000)),
    });
    cursor.setMonth(cursor.getMonth() + 1);
  }

  return labels;
}
