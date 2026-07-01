import { Copy, FolderOpen, RefreshCw, X } from "lucide-react";
import { diagnosticSummary } from "../lib/diagnostics";
import type { DiagnosticsState } from "../types";

interface DiagnosticsPanelProps {
  diagnostics: DiagnosticsState | null;
  autostartEnabled: boolean;
  busy?: boolean;
  error?: string | null;
  onClose: () => void;
  onRefresh: () => void;
  onOpenLogs: () => void;
  onToggleAutostart: (enabled: boolean) => void;
}

export function DiagnosticsPanel({
  diagnostics,
  autostartEnabled,
  busy = false,
  error,
  onClose,
  onRefresh,
  onOpenLogs,
  onToggleAutostart,
}: DiagnosticsPanelProps) {
  const copySummary = async () => {
    if (diagnostics) {
      await navigator.clipboard.writeText(diagnosticSummary(diagnostics));
    }
  };

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="diagnostics-panel" aria-label="Diagnostics">
        <header className="modal-header">
          <h2>Diagnostics</h2>
          <button className="icon-button" type="button" title="Close" aria-label="Close diagnostics" onClick={onClose}>
            <X size={16} />
          </button>
        </header>

        {error ? <p className="diagnostics-error">{error}</p> : null}

        <label className="toggle-row">
          <input
            type="checkbox"
            checked={autostartEnabled}
            onChange={(event) => onToggleAutostart(event.currentTarget.checked)}
          />
          <span>Launch at startup</span>
        </label>

        {diagnostics ? (
          <dl className="diagnostics-grid">
            <dt>Version</dt>
            <dd>{diagnostics.appVersion}</dd>
            <dt>Platform</dt>
            <dd>
              {diagnostics.platform} {diagnostics.arch}
            </dd>
            <dt>Sessions</dt>
            <dd>{diagnostics.sessionsPath}</dd>
            <dt>Readable</dt>
            <dd>{diagnostics.sessionsExists && diagnostics.sessionsReadable ? "yes" : "no"}</dd>
            <dt>Database</dt>
            <dd>{diagnostics.databasePath}</dd>
            <dt>Logs</dt>
            <dd>{diagnostics.logDirectory}</dd>
            <dt>Scan</dt>
            <dd>{diagnostics.lastScanResult}</dd>
            <dt>Counts</dt>
            <dd>
              {diagnostics.filesScanned} files, {diagnostics.tokenEventsAccepted} events,{" "}
              {diagnostics.limitSnapshotsAccepted} limits
            </dd>
            <dt>Warnings</dt>
            <dd>
              {diagnostics.malformedLines} malformed, {diagnostics.ioFailures} I/O
            </dd>
            <dt>Error</dt>
            <dd>{diagnostics.lastError ?? "none"}</dd>
          </dl>
        ) : (
          <div className="diagnostics-empty">{busy ? "Loading..." : "No diagnostics yet"}</div>
        )}

        <footer className="modal-actions">
          <button className="secondary-button" type="button" onClick={onRefresh} disabled={busy}>
            <RefreshCw size={14} />
            Refresh
          </button>
          <button className="secondary-button" type="button" onClick={onOpenLogs}>
            <FolderOpen size={14} />
            Logs
          </button>
          <button className="primary-button" type="button" onClick={copySummary} disabled={!diagnostics}>
            <Copy size={14} />
            Copy
          </button>
        </footer>
      </section>
    </div>
  );
}
