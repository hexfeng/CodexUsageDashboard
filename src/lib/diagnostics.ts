import type { DiagnosticsState } from "../types";

export function diagnosticSummary(diagnostics: DiagnosticsState): string {
  return [
    `Codex Usage ${diagnostics.appVersion}`,
    `Platform: ${diagnostics.platform} ${diagnostics.arch}`,
    `Sessions path: ${diagnostics.sessionsPath}`,
    `Sessions readable: ${diagnostics.sessionsReadable ? "yes" : "no"}`,
    `Database path: ${diagnostics.databasePath}`,
    `Log directory: ${diagnostics.logDirectory}`,
    `Last scan: ${diagnostics.lastScanCompletedAt ?? "never"}`,
    `Last successful update: ${diagnostics.lastSuccessfulDataUpdate ?? "never"}`,
    `Last scan result: ${diagnostics.lastScanResult}`,
    `Last error: ${diagnostics.lastError ?? "none"}`,
  ].join("\n");
}
