export type FreshnessState = "fresh" | "stale" | "missing";
export type LimitTone = "ok" | "warning" | "critical" | "unavailable";

export interface Freshness {
  state: FreshnessState;
  label: string;
  ageSeconds: number | null;
}

export interface LimitBucket {
  label: "5h" | "Weekly";
  usedPercent?: number;
  remainingPercent?: number;
  resetAt?: number | null;
  resetLabel?: string;
  available: boolean;
  unusual: boolean;
}

export interface LimitSummary {
  fiveHour: LimitBucket;
  weekly: LimitBucket;
  planType: string | null;
}

export interface TodayUsage {
  date: string;
  totalTokens: number;
  inputTokens: number;
  cachedInputTokens: number;
  outputTokens: number;
  reasoningTokens: number;
  sessions: number;
}

export interface HeatmapDay {
  date: string;
  totalTokens: number;
  sessions: number;
  level?: number;
}

export interface DashboardState {
  sourcePath: string;
  updatedAt: string | null;
  freshness: Freshness;
  warnings: string[];
  limits: LimitSummary;
  today: TodayUsage;
  heatmapDays: HeatmapDay[];
}

export interface Settings {
  sessionsPath: string;
  alwaysOnTop: boolean;
  staleAfterMinutes: number;
  heatmapDays: number;
}

export interface DiagnosticsState {
  appVersion: string;
  platform: string;
  arch: string;
  sessionsPath: string;
  sessionsExists: boolean;
  sessionsReadable: boolean;
  databasePath: string;
  logDirectory: string;
  lastScanStartedAt: string | null;
  lastScanCompletedAt: string | null;
  lastSuccessfulDataUpdate: string | null;
  watcherStatus: string;
  filesScanned: number;
  tokenEventsAccepted: number;
  limitSnapshotsAccepted: number;
  malformedLines: number;
  ioFailures: number;
  lastScanResult: string;
  lastError: string | null;
}
