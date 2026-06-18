import { describe, expect, it } from "vitest";
import {
  formatCountdown,
  formatTokens,
  getFreshness,
  getLimitTone,
} from "./format";

describe("formatTokens", () => {
  it("formats large token counts compactly", () => {
    expect(formatTokens(1_824_000)).toBe("1.82M");
    expect(formatTokens(984_000)).toBe("984K");
    expect(formatTokens(0)).toBe("0");
  });
});

describe("formatCountdown", () => {
  it("formats future epoch seconds as short reset copy", () => {
    const now = new Date("2026-06-17T17:00:00.000Z");

    expect(formatCountdown(1_781_720_400, now)).toBe("1h 20m");
    expect(formatCountdown(1_782_079_200, now)).toBe("4d 5h");
  });

  it("handles missing or elapsed reset times", () => {
    const now = new Date("2026-06-17T17:00:00.000Z");

    expect(formatCountdown(null, now)).toBe("unknown");
    expect(formatCountdown(1_781_715_540, now)).toBe("now");
  });
});

describe("getLimitTone", () => {
  it("uses remaining percentage to classify status", () => {
    expect(getLimitTone(20)).toBe("ok");
    expect(getLimitTone(51)).toBe("warning");
    expect(getLimitTone(81)).toBe("critical");
  });
});

describe("getFreshness", () => {
  it("marks recent data fresh and old data stale", () => {
    const now = new Date("2026-06-17T17:10:00.000Z");

    expect(getFreshness("2026-06-17T17:08:30.000Z", now, 5).state).toBe("fresh");
    expect(getFreshness("2026-06-17T16:58:30.000Z", now, 5).state).toBe("stale");
  });
});
