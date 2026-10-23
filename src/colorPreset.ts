import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, type ColorPreset, type ColorPresetPatch, type PresetColors } from "./api";
import { useNotifications } from "./notifications/Notifications";
import { BUILTIN_PRESETS, DEFAULT_PRESET, derivePalette, modeColors } from "./palette";

// The backend holds presets and the choice. The resolved colours are cached
// here so both windows paint the right colours before the backend answers,
// and so the quick entry window follows changes via the storage event.
const CACHE_KEY = "hodie.palette";
const CUSTOM_PREFIX = "custom:";
// Saving colours waits until the colour picker has stopped moving.
const SAVE_DELAY_MS = 300;

let applied: string[] = [];

export function customChoice(id: number): string {
  return `${CUSTOM_PREFIX}${id}`;
}

/** The colours of a choice, or null for the hand-tuned default in tokens.css. */
export function resolveChoice(choice: string, presets: ColorPreset[]): PresetColors | null {
  if (choice === DEFAULT_PRESET) return null;
  const builtin = BUILTIN_PRESETS.find((preset) => preset.name === choice);
  if (builtin) return builtin.colors;
  return presets.find((preset) => customChoice(preset.id) === choice)?.colors ?? null;
}

function readCache(): PresetColors | null {
  try {
    const stored = localStorage.getItem(CACHE_KEY);
    return stored ? (JSON.parse(stored) as PresetColors) : null;
  } catch {
    return null;
  }
}

/** Sets the derived tokens for the current light or dark mode. */
export function applyPalette() {
  const root = document.documentElement;
  const colors = readCache();
  const dark = root.dataset.theme === "dark";
  const palette = colors ? derivePalette(modeColors(colors, dark), dark) : {};
  for (const name of applied) root.style.removeProperty(`--${name}`);
  applied = Object.keys(palette);
  for (const [name, value] of Object.entries(palette)) root.style.setProperty(`--${name}`, value);
}

function activate(colors: PresetColors | null) {
  try {
    if (colors) localStorage.setItem(CACHE_KEY, JSON.stringify(colors));
    else localStorage.removeItem(CACHE_KEY);
  } catch {
    // Not cached; the colours still apply in this window.
  }
  applyPalette();
}

/** Applies the cached colours and follows changes from the other window. */
export function initPalette() {
  applyPalette();
  window.addEventListener("storage", (event) => {
    if (event.key === CACHE_KEY) applyPalette();
  });
}

/** Refreshes the cache from the backend, e.g. after it was cleared. */
export async function syncPalette() {
  try {
    const [presets, choice] = await Promise.all([
      api.listColorPresets(),
      api.getColorPresetChoice(),
    ]);
    activate(resolveChoice(choice, presets));
  } catch {
    // Keep the cached colours; the settings show errors when opened.
  }
}

/** Presets and the active choice for the settings, with optimistic changes. */
export function useColorPresets() {
  const { t } = useTranslation();
  const notify = useNotifications();
  const [presets, setPresets] = useState<ColorPreset[] | null>(null);
  const [choice, setChoice] = useState<string>(DEFAULT_PRESET);
  const latest = useRef({ presets, choice });
  latest.current = { presets, choice };
  const pending = useRef(new Map<number, number>());

  const reload = useCallback(async () => {
    try {
      const [loaded, active] = await Promise.all([
        api.listColorPresets(),
        api.getColorPresetChoice(),
      ]);
      setPresets(loaded);
      setChoice(active);
      activate(resolveChoice(active, loaded));
    } catch (error) {
      notify.error(error);
    }
  }, [notify]);

  useEffect(() => {
    void reload();
  }, [reload]);

  // Unsaved colour changes are written right away when the settings close.
  useEffect(() => {
    const timers = pending.current;
    return () => {
      for (const timer of timers.values()) window.clearTimeout(timer);
      for (const id of timers.keys()) {
        const preset = latest.current.presets?.find((p) => p.id === id);
        if (preset) void api.updateColorPreset(id, { colors: preset.colors });
      }
      timers.clear();
    };
  }, []);

  const select = useCallback(
    async (next: string, list = latest.current.presets ?? []) => {
      setChoice(next);
      activate(resolveChoice(next, list));
      try {
        await api.setColorPresetChoice(next);
      } catch (error) {
        notify.error(error);
        void reload();
      }
    },
    [notify, reload],
  );

  /** Starts a new preset from the active colours and switches to it. */
  const create = useCallback(async () => {
    const { presets: list, choice: active } = latest.current;
    const base = resolveChoice(active, list ?? []) ?? BUILTIN_PRESETS[0].colors;
    try {
      const preset = await api.createColorPreset(t("settings.colors.newName"), base);
      const next = [...(list ?? []), preset];
      setPresets(next);
      await select(customChoice(preset.id), next);
    } catch (error) {
      notify.error(error);
    }
  }, [notify, select, t]);

  const update = useCallback(
    (id: number, patch: ColorPresetPatch) => {
      const next = (latest.current.presets ?? []).map((preset) =>
        preset.id === id ? { ...preset, ...patch } : preset,
      );
      setPresets(next);
      if (latest.current.choice === customChoice(id)) activate(resolveChoice(customChoice(id), next));

      const save = async () => {
        try {
          await api.updateColorPreset(id, patch);
        } catch (error) {
          notify.error(error);
          void reload();
        }
      };
      if (!patch.colors) {
        void save();
        return;
      }
      const timers = pending.current;
      window.clearTimeout(timers.get(id));
      timers.set(
        id,
        window.setTimeout(() => {
          timers.delete(id);
          void save();
        }, SAVE_DELAY_MS),
      );
    },
    [notify, reload],
  );

  const remove = useCallback(
    async (preset: ColorPreset) => {
      const wasActive = latest.current.choice === customChoice(preset.id);
      window.clearTimeout(pending.current.get(preset.id));
      pending.current.delete(preset.id);
      setPresets((current) => current?.filter((p) => p.id !== preset.id) ?? null);
      if (wasActive) {
        setChoice(DEFAULT_PRESET);
        activate(null);
      }
      try {
        await api.deleteColorPreset(preset.id);
        notify.info(t("settings.colors.deleted", { name: preset.name }), async () => {
          // Undo recreates it; the backend gives it a new id.
          try {
            const restored = await api.createColorPreset(preset.name, preset.colors);
            const next = [...(latest.current.presets ?? []), restored];
            setPresets(next);
            if (wasActive) await select(customChoice(restored.id), next);
          } catch (error) {
            notify.error(error);
          }
        });
      } catch (error) {
        notify.error(error);
        void reload();
      }
    },
    [notify, reload, select, t],
  );

  return { presets, choice, select, create, update, remove };
}
