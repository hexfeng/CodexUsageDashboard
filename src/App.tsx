import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getDashboardState, refreshNow, setAlwaysOnTop } from "./lib/api";
import { createFallbackState } from "./lib/fallbackState";
import { DashboardWidget } from "./components/DashboardWidget";
import type { DashboardState } from "./types";
import "./styles.css";

export default function App() {
  const [state, setState] = useState<DashboardState>(() => createFallbackState());
  const [pinned, setPinned] = useState(true);
  const [refreshing, setRefreshing] = useState(false);

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

  return (
    <DashboardWidget
      state={state}
      pinned={pinned}
      refreshing={refreshing}
      onRefresh={handleRefresh}
      onTogglePin={handleTogglePin}
    />
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
