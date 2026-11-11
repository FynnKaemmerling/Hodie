import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNotifications } from "../notifications/Notifications";
import * as pomodoro from "./pomodoro";
import type { PomodoroPhase } from "./pomodoro";
import { currentPomodoroSound, loadPomodoroSound } from "./soundSetting";
import { playSound, unlockSound } from "./sounds";
import { notifySystem, prepareSystemNotifications } from "./systemNotification";

// Often enough that the seconds never visibly skip.
const TICK_MS = 250;

export interface Pomodoro {
  phase: PomodoroPhase;
  remainingMs: number;
  running: boolean;
  /** Running, or paused part-way through a phase. */
  active: boolean;
  toggle: () => void;
  reset: () => void;
  skip: () => void;
}

/** The Pomodoro timer of the main window; lives as long as the window. */
export function usePomodoro(): Pomodoro {
  const { t } = useTranslation();
  const notify = useNotifications();
  const [state, setState] = useState(pomodoro.INITIAL_POMODORO);
  const [now, setNow] = useState(() => Date.now());
  const running = pomodoro.isRunning(state);

  useEffect(() => void loadPomodoroSound(), []);

  useEffect(() => {
    if (!running) return;
    const timer = window.setInterval(() => setNow(Date.now()), TICK_MS);
    return () => window.clearInterval(timer);
  }, [running]);

  // Hidden windows throttle repeating timers hard, a single one far less;
  // so the phase end is also caught by its own timeout.
  const endsAt = state.endsAt;
  useEffect(() => {
    if (endsAt === null) return;
    const timer = window.setTimeout(() => setNow(Date.now()), Math.max(0, endsAt - Date.now()));
    return () => window.clearTimeout(timer);
  }, [endsAt]);

  // A finished phase hands over to the next one and says so.
  useEffect(() => {
    const next = pomodoro.advance(state, now);
    if (next === state) return;
    setState(next);
    const message = t(next.phase === "focus" ? "pomodoro.breakDone" : "pomodoro.focusDone");
    notify.info(message);
    playSound(currentPomodoroSound());
    // The toast goes unseen while the window is in the background.
    if (!document.hasFocus()) void notifySystem(t("pomodoro.title"), message);
  }, [state, now, notify, t]);

  const toggle = useCallback(() => {
    const at = Date.now();
    setNow(at);
    void prepareSystemNotifications();
    unlockSound();
    setState((s) => (pomodoro.isRunning(s) ? pomodoro.pause(s, at) : pomodoro.start(s, at)));
  }, []);
  const reset = useCallback(() => setState(pomodoro.reset), []);
  const skip = useCallback(() => setState(pomodoro.skip), []);

  return {
    phase: state.phase,
    remainingMs: pomodoro.remainingMs(state, now),
    running,
    active: pomodoro.isActive(state),
    toggle,
    reset,
    skip,
  };
}
