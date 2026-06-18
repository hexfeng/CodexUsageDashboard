import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { DashboardWidget } from "./DashboardWidget";
import type { DashboardState } from "../types";

const baseState: DashboardState = {
  sourcePath: "C:\\Users\\PC\\.codex\\sessions",
  updatedAt: "2026-06-17T17:09:00.000Z",
  freshness: { state: "fresh", label: "Updated 1m ago", ageSeconds: 60 },
  warnings: [],
  limits: {
    fiveHour: {
      label: "5h",
      usedPercent: 42,
      remainingPercent: 58,
      resetAt: 1_797_770_400,
      resetLabel: "1h 20m",
      available: true,
      unusual: false,
    },
    weekly: {
      label: "Weekly",
      usedPercent: 68,
      remainingPercent: 32,
      resetAt: 1_798_132_400,
      resetLabel: "4d 5h",
      available: true,
      unusual: false,
    },
    planType: "plus",
  },
  today: {
    date: "2026-06-17",
    totalTokens: 1_824_000,
    inputTokens: 1_240_000,
    cachedInputTokens: 410_000,
    outputTokens: 170_000,
    reasoningTokens: 4_000,
    sessions: 6,
  },
  heatmapDays: [
    { date: "2026-06-04", totalTokens: 0, sessions: 0, level: 0 },
    { date: "2026-06-05", totalTokens: 20_000, sessions: 1, level: 1 },
    { date: "2026-06-06", totalTokens: 100_000, sessions: 1, level: 2 },
    { date: "2026-06-07", totalTokens: 600_000, sessions: 2, level: 4 },
  ],
};

describe("DashboardWidget", () => {
  it("renders the core widget metrics", () => {
    render(<DashboardWidget state={baseState} pinned={true} onRefresh={vi.fn()} onTogglePin={vi.fn()} />);

    expect(screen.getByText("Codex Usage")).toBeInTheDocument();
    expect(screen.getByText("Token activity")).toBeInTheDocument();
    expect(screen.getByText("Remaining 58%")).toBeInTheDocument();
    expect(screen.getByText("Remaining 32%")).toBeInTheDocument();
    expect(screen.getByText("5 Hours")).toBeInTheDocument();
    expect(screen.getAllByText("Weekly").length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText("1.82M tokens")).toBeInTheDocument();
    expect(screen.getByLabelText("12 month token activity heatmap")).toBeInTheDocument();
    expect(screen.getByLabelText("Refresh usage data")).toBeInTheDocument();
    expect(screen.getByLabelText("Disable always on top")).toBeInTheDocument();
  });

  it("switches token activity metric tabs", async () => {
    const user = userEvent.setup();
    render(<DashboardWidget state={baseState} pinned={true} onRefresh={vi.fn()} onTogglePin={vi.fn()} />);

    expect(screen.getByRole("button", { name: "Daily token activity" })).toHaveAttribute("aria-pressed", "true");

    await user.click(screen.getByRole("button", { name: "Weekly token activity" }));

    expect(screen.getByRole("button", { name: "Weekly token activity" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByLabelText("2026-06-06: 120K weekly tokens")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Cumulative token activity" }));

    expect(screen.getByRole("button", { name: "Cumulative token activity" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByLabelText("2026-06-07: 720K cumulative tokens")).toBeInTheDocument();
  });

  it("renders unavailable limits without crashing", () => {
    const state: DashboardState = {
      ...baseState,
      limits: {
        fiveHour: { label: "5h", available: false, unusual: false },
        weekly: { label: "Weekly", available: false, unusual: false },
        planType: null,
      },
      warnings: ["No rate_limits found in latest Codex session logs."],
    };

    render(<DashboardWidget state={state} pinned={false} onRefresh={vi.fn()} onTogglePin={vi.fn()} />);

    expect(screen.getAllByText("Usage limit unavailable")).toHaveLength(2);
    expect(screen.getByLabelText("Enable always on top")).toBeInTheDocument();
  });

  it("colors limit bars from remaining percentage thresholds", () => {
    const state: DashboardState = {
      ...baseState,
      limits: {
        fiveHour: {
          ...baseState.limits.fiveHour,
          usedPercent: 51,
          remainingPercent: 49,
        },
        weekly: {
          ...baseState.limits.weekly,
          usedPercent: 81,
          remainingPercent: 19,
        },
        planType: "plus",
      },
    };

    const { container } = render(
      <DashboardWidget state={state} pinned={true} onRefresh={vi.fn()} onTogglePin={vi.fn()} />,
    );

    expect(container.querySelector(".segment-active.segment-warning")).toBeInTheDocument();
    expect(container.querySelector(".segment-active.segment-critical")).toBeInTheDocument();
  });

  it("ticks the freshness label between data refreshes", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-17T17:09:00.000Z"));
    const state: DashboardState = {
      ...baseState,
      updatedAt: "2026-06-17T17:09:00.000Z",
      freshness: { state: "fresh", label: "Updated 0s ago", ageSeconds: 0 },
    };

    try {
      render(<DashboardWidget state={state} pinned={true} onRefresh={vi.fn()} onTogglePin={vi.fn()} />);

      expect(screen.getAllByText("Updated 0s ago")).toHaveLength(2);

      act(() => {
        vi.advanceTimersByTime(2000);
      });

      expect(screen.getAllByText("Updated 2s ago")).toHaveLength(2);
    } finally {
      vi.useRealTimers();
    }
  });
});
