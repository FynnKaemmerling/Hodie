import { useSyncExternalStore } from "react";
import { api } from "../api";
import { DEFAULT_SOUND, isSoundChoice, type PomodoroSoundChoice } from "./sounds";

// The chosen sound, shared by the timer and the settings in the main window.
let current: PomodoroSoundChoice = DEFAULT_SOUND;
let loading: Promise<void> | null = null;
const listeners = new Set<() => void>();

function update(choice: PomodoroSoundChoice) {
  current = choice;
  listeners.forEach((listener) => listener());
}

/** Reads the choice from the settings once; unknown names fall back to the default. */
export function loadPomodoroSound(): Promise<void> {
  loading ??= api
    .getPomodoroSound()
    .then((stored) => {
      if (isSoundChoice(stored)) update(stored);
    })
    .catch(() => {
      loading = null;
    });
  return loading;
}

export function currentPomodoroSound(): PomodoroSoundChoice {
  return current;
}

/** Saves a new choice; shows it at once and takes it back if saving fails. */
export async function choosePomodoroSound(choice: PomodoroSoundChoice): Promise<void> {
  const previous = current;
  update(choice);
  try {
    await api.setPomodoroSound(choice);
  } catch (error) {
    update(previous);
    throw error;
  }
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  void loadPomodoroSound();
  return () => listeners.delete(listener);
}

export function usePomodoroSound(): PomodoroSoundChoice {
  return useSyncExternalStore(subscribe, currentPomodoroSound);
}
