import { invoke } from "@tauri-apps/api/core";
import type { DashboardState, Settings } from "../types";

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
