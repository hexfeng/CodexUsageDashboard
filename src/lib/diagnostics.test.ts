import { describe, expect, it } from "vitest";
import { diagnosticSummary } from "./diagnostics";
import type { DiagnosticsState } from "../types";

describe("diagnosticSummary", () => {
  it("copies only sanitized diagnostic metadata", () => {
    const diagnostics: DiagnosticsState = {
      appVersion: "0.2.0",
      platform: "windows",
      arch: "x86_64",
      sessionsPath: "C:/Users/PC/.codex/sessions",
      sessionsExists: true,
      sessionsReadable: true,
      databasePath: "C:/Users/PC/AppData/Local/CodexUsageWidget/usage.sqlite",
      logDirectory: "C:/Users/PC/AppData/Local/Codex Usage/logs",
      lastScanStartedAt: "2026-06-30T20:00:00Z",
      lastScanCompletedAt: "2026-06-30T20:00:01Z",
      lastSuccessfulDataUpdate: "2026-06-30T20:00:01Z",
      watcherStatus: "polling",
      filesScanned: 2,
      tokenEventsAccepted: 3,
      limitSnapshotsAccepted: 1,
      malformedLines: 0,
      ioFailures: 0,
      lastScanResult: "success",
      lastError: "Permission denied",
    };

    const summary = diagnosticSummary(diagnostics);

    expect(summary).toContain("Codex Usage 0.2.0");
    expect(summary).toContain("Sessions readable: yes");
    expect(summary).not.toContain("prompt");
    expect(summary).not.toContain("message");
    expect(summary).not.toContain("apiKey");
    expect(summary).not.toContain("payload");
  });
});
