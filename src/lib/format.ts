import type { Freshness, LimitTone } from "../types";

export function formatTokens(tokens: number): string {
  if (!Number.isFinite(tokens) || tokens <= 0) {
    return "0";
  }

  if (tokens >= 1_000_000) {
    return `${trimFixed(tokens / 1_000_000, 2)}M`;
  }

  if (tokens >= 1_000) {
    return `${Math.round(tokens / 1_000)}K`;
  }

  return Math.round(tokens).toString();
}

export function formatCountdown(epochSeconds: number | null | undefined, now = new Date()): string {
  if (!epochSeconds) {
    return "unknown";
  }

  const seconds = Math.max(0, Math.floor(epochSeconds - now.getTime() / 1000));
  if (seconds <= 0) {
    return "now";
  }

  const days = Math.floor(seconds / 86_400);
  const hours = Math.floor((seconds % 86_400) / 3_600);
  const minutes = Math.floor((seconds % 3_600) / 60);

  if (days > 0) {
    return `${days}d ${hours}h`;
  }

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }

  return `${Math.max(1, minutes)}m`;
}

export function getLimitTone(usedPercent: number | null | undefined): LimitTone {
  if (usedPercent === null || usedPercent === undefined || !Number.isFinite(usedPercent)) {
    return "unavailable";
  }

  const remaining = 100 - usedPercent;
  if (remaining < 20) {
    return "critical";
  }

  if (remaining < 50) {
    return "warning";
  }

  return "ok";
}

export function getFreshness(
  updatedAt: string | null | undefined,
  now = new Date(),
  staleAfterMinutes = 5,
): Freshness {
  if (!updatedAt) {
    return {
      state: "missing",
      label: "No local data yet",
      ageSeconds: null,
    };
  }

  const updated = new Date(updatedAt);
  const ageSeconds = Math.max(0, Math.floor((now.getTime() - updated.getTime()) / 1000));
  const state = ageSeconds > staleAfterMinutes * 60 ? "stale" : "fresh";

  return {
    state,
    label: `Updated ${formatAge(ageSeconds)} ago`,
    ageSeconds,
  };
}

export function formatPercent(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return "unknown";
  }

  return `${Math.round(value)}%`;
}

function formatAge(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s`;
  }

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m`;
  }

  const hours = Math.floor(minutes / 60);
  return `${hours}h ${minutes % 60}m`;
}

function trimFixed(value: number, digits: number): string {
  return value.toFixed(digits).replace(/\.?0+$/, "");
}
