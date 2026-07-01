import { invoke } from "@tauri-apps/api/core";
import type { DashboardState, DiagnosticsState, Settings } from "../types";

export async function getDashboardState(): Promise<DashboardState> {
  return invoke<DashboardState>("get_dashboard_state");
}

export async function refreshNow(): Promise<DashboardState> {
  return invoke<DashboardState>("refresh_now");
}

export async function setAlwaysOnTop(enabled: boolean): Promise<void> {
  await invoke("set_always_on_top", { enabled });
}

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function updateSettings(settings: Settings): Promise<Settings> {
  return invoke<Settings>("update_settings", { settings });
}

export async function getDiagnostics(): Promise<DiagnosticsState> {
  return invoke<DiagnosticsState>("get_diagnostics");
}

export async function openLogsFolder(): Promise<void> {
  return invoke<void>("open_logs_folder");
}

export async function getAutostartEnabled(): Promise<boolean> {
  return invoke<boolean>("get_autostart_enabled");
}

export async function setAutostartEnabled(enabled: boolean): Promise<boolean> {
  return invoke<boolean>("set_autostart_enabled", { enabled });
}
