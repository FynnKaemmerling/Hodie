import { useSyncExternalStore } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export const THEME_PREFERENCES = ["system", "light", "dark"] as const;
export type ThemePreference = (typeof THEME_PREFERENCES)[number];

// Temporary storage until settings live in the Rust backend. Shared by both
// windows, which also hear each other's changes through the storage event.
const STORAGE_KEY = "hodie.theme";
const systemDark = window.matchMedia("(prefers-color-scheme: dark)");
const listeners = new Set<() => void>();

function isPreference(value: string | null): value is ThemePreference {
  return THEME_PREFERENCES.includes(value as ThemePreference);
}

export function readThemePreference(): ThemePreference {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return isPreference(stored) ? stored : "system";
  } catch {
    return "system";
  }
}

/** Sets the colour tokens (see tokens.css) and the native window frame. */
function apply(preference: ThemePreference) {
  const dark = preference === "dark" || (preference === "system" && systemDark.matches);
  document.documentElement.dataset.theme = dark ? "dark" : "light";
  try {
    void getCurrentWindow()
      .setTheme(preference === "system" ? null : preference)
      .catch(() => {});
  } catch {
    // Not running inside Tauri (e.g. plain browser); the page theme still applies.
  }
  listeners.forEach((listener) => listener());
}

export function setThemePreference(preference: ThemePreference) {
  try {
    localStorage.setItem(STORAGE_KEY, preference);
  } catch {
    // Not persisted; the theme still applies for this session.
  }
  apply(preference);
}

/** Applies the stored theme before the first render and keeps it current. */
export function initTheme() {
  apply(readThemePreference());
  systemDark.addEventListener("change", () => apply(readThemePreference()));
  window.addEventListener("storage", (event) => {
    if (event.key === STORAGE_KEY) apply(readThemePreference());
  });
}

export function useThemePreference(): ThemePreference {
  return useSyncExternalStore((listener) => {
    listeners.add(listener);
    return () => listeners.delete(listener);
  }, readThemePreference);
}
