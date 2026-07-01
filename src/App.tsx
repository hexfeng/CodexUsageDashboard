import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { check, type Update } from "@tauri-apps/plugin-updater";
import {
  getAutostartEnabled,
  getDashboardState,
  getDiagnostics,
  openLogsFolder,
  refreshNow,
  setAlwaysOnTop,
  setAutostartEnabled,
} from "./lib/api";
import { createFallbackState } from "./lib/fallbackState";
import { DiagnosticsPanel } from "./components/DiagnosticsPanel";
import { DashboardWidget } from "./components/DashboardWidget";
import { UpdatePrompt } from "./components/UpdatePrompt";
import type { DashboardState, DiagnosticsState } from "./types";
import "./styles.css";

export default function App() {
  const [state, setState] = useState<DashboardState>(() => createFallbackState());
  const [pinned, setPinned] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false);
  const [diagnostics, setDiagnostics] = useState<DiagnosticsState | null>(null);
  const [diagnosticsBusy, setDiagnosticsBusy] = useState(false);
  const [diagnosticsError, setDiagnosticsError] = useState<string | null>(null);
  const [autostart, setAutostart] = useState(false);
  const [update, setUpdate] = useState<Update | null>(null);
  const [updateInstalling, setUpdateInstalling] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);

  const loadState = useCallback(async () => {
    setRefreshing(true);
    try {
      setState(await getDashboardState());
    } catch (error) {
      setState((current) => ({
        ...current,
        warnings: [`Unable to read local Codex usage data: ${getErrorMessage(error)}`],
      }));
    } finally {
      setRefreshing(false);
    }
  }, []);

  useEffect(() => {
    void loadState();
    const id = window.setInterval(() => {
      void loadState();
    }, 60_000);

    return () => window.clearInterval(id);
  }, [loadState]);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }

    let disposed = false;
    const unlisten = listen<DashboardState>("dashboard-state-updated", (event) => {
      if (!disposed) {
        setState(event.payload);
      }
    }).catch((error) => {
      setState((current) => ({
        ...current,
        warnings: [`Unable to subscribe to local usage updates: ${getErrorMessage(error)}`],
      }));
      return () => {};
    });

    return () => {
      disposed = true;
      void unlisten.then((dispose) => dispose());
    };
  }, []);

  const loadDiagnostics = useCallback(async () => {
    setDiagnosticsBusy(true);
    setDiagnosticsError(null);
    try {
      const [nextDiagnostics, nextAutostart] = await Promise.all([getDiagnostics(), getAutostartEnabled()]);
      setDiagnostics(nextDiagnostics);
      setAutostart(nextAutostart);
    } catch (error) {
      setDiagnosticsError(getErrorMessage(error));
    } finally {
      setDiagnosticsBusy(false);
    }
  }, []);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }

    let disposed = false;
    const unlisten = listen("show-diagnostics", () => {
      if (!disposed) {
        setDiagnosticsOpen(true);
        void loadDiagnostics();
      }
    }).catch((error) => {
      setState((current) => ({
        ...current,
        warnings: [`Unable to subscribe to diagnostics event: ${getErrorMessage(error)}`],
      }));
      return () => {};
    });

    return () => {
      disposed = true;
      void unlisten.then((dispose) => dispose());
    };
  }, [loadDiagnostics]);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }

    const id = window.setTimeout(() => {
      check()
        .then((nextUpdate) => {
          if (nextUpdate) {
            setUpdate(nextUpdate);
          }
        })
        .catch((error) => {
          console.info("Update check skipped:", getErrorMessage(error));
        });
    }, 5_000);

    return () => window.clearTimeout(id);
  }, []);

  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    try {
      setState(await refreshNow());
    } catch (error) {
      setState((current) => ({
        ...current,
        warnings: [`Refresh failed: ${getErrorMessage(error)}`],
      }));
    } finally {
      setRefreshing(false);
    }
  }, []);

  const handleTogglePin = useCallback(async () => {
    const next = !pinned;
    setPinned(next);
    try {
      await setAlwaysOnTop(next);
    } catch (error) {
      setPinned(!next);
      setState((current) => ({
        ...current,
        warnings: [`Pin toggle failed: ${getErrorMessage(error)}`],
      }));
    }
  }, [pinned]);

  const handleToggleAutostart = useCallback(async (enabled: boolean) => {
    const previous = autostart;
    setAutostart(enabled);
    try {
      setAutostart(await setAutostartEnabled(enabled));
    } catch (error) {
      setAutostart(previous);
      setDiagnosticsError(getErrorMessage(error));
    }
  }, [autostart]);

  const handleInstallUpdate = useCallback(async () => {
    if (!update) {
      return;
    }

    setUpdateInstalling(true);
    setUpdateError(null);
    try {
      await update.downloadAndInstall();
    } catch (error) {
      setUpdateError(getErrorMessage(error));
    } finally {
      setUpdateInstalling(false);
    }
  }, [update]);

  return (
    <>
      <DashboardWidget
        state={state}
        pinned={pinned}
        refreshing={refreshing}
        onRefresh={handleRefresh}
        onTogglePin={handleTogglePin}
      />
      {diagnosticsOpen ? (
        <DiagnosticsPanel
          diagnostics={diagnostics}
          autostartEnabled={autostart}
          busy={diagnosticsBusy}
          error={diagnosticsError}
          onClose={() => setDiagnosticsOpen(false)}
          onRefresh={loadDiagnostics}
          onOpenLogs={() => {
            void openLogsFolder().catch((error) => setDiagnosticsError(getErrorMessage(error)));
          }}
          onToggleAutostart={handleToggleAutostart}
        />
      ) : null}
      {update ? (
        <UpdatePrompt
          version={update.version}
          notes={typeof update.body === "string" ? update.body : undefined}
          installing={updateInstalling}
          error={updateError}
          onInstall={handleInstallUpdate}
          onDismiss={() => setUpdate(null)}
        />
      ) : null}
    </>
  );
}

function getErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function isTauriRuntime(): boolean {
  return typeof getTauriInternals()?.transformCallback === "function";
}

function getTauriInternals(): { transformCallback?: unknown } | undefined {
  return (window as unknown as { __TAURI_INTERNALS__?: { transformCallback?: unknown } }).__TAURI_INTERNALS__;
}
