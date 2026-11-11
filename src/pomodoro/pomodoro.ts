/**
 * Pomodoro timer as pure functions over a small state. Running timers keep
 * their end time rather than counting ticks, so a throttled or sleeping
 * window never makes them drift.
 */

export type PomodoroPhase = "focus" | "shortBreak" | "longBreak";

const MINUTE = 60_000;

export const PHASE_MS: Record<PomodoroPhase, number> = {
  focus: 25 * MINUTE,
  shortBreak: 5 * MINUTE,
  longBreak: 15 * MINUTE,
};

/** Every so many focus sessions, the break is a long one. */
export const SESSIONS_PER_LONG_BREAK = 4;

export interface PomodoroState {
  phase: PomodoroPhase;
  /** Focus sessions finished so far. */
  completed: number;
  /** When the running phase ends (epoch ms); null while stopped or paused. */
  endsAt: number | null;
  /** Time left while stopped or paused. */
  remainingMs: number;
}

export const INITIAL_POMODORO: PomodoroState = {
  phase: "focus",
  completed: 0,
  endsAt: null,
  remainingMs: PHASE_MS.focus,
};

export function isRunning(state: PomodoroState): boolean {
  return state.endsAt !== null;
}

/** Running, or paused part-way through a phase. */
export function isActive(state: PomodoroState): boolean {
  return isRunning(state) || state.remainingMs < PHASE_MS[state.phase];
}

export function remainingMs(state: PomodoroState, now: number): number {
  return state.endsAt === null ? state.remainingMs : Math.max(0, state.endsAt - now);
}

export function start(state: PomodoroState, now: number): PomodoroState {
  if (isRunning(state)) return state;
  return { ...state, endsAt: now + state.remainingMs };
}

export function pause(state: PomodoroState, now: number): PomodoroState {
  if (!isRunning(state)) return state;
  return { ...state, endsAt: null, remainingMs: remainingMs(state, now) };
}

/** Back to the start of the current phase, stopped. */
export function reset(state: PomodoroState): PomodoroState {
  return { ...state, endsAt: null, remainingMs: PHASE_MS[state.phase] };
}

/** Ends the current phase now and moves on to the next one, stopped. */
export function skip(state: PomodoroState): PomodoroState {
  if (state.phase !== "focus") {
    return { ...state, phase: "focus", endsAt: null, remainingMs: PHASE_MS.focus };
  }
  const completed = state.completed + 1;
  const phase = completed % SESSIONS_PER_LONG_BREAK === 0 ? "longBreak" : "shortBreak";
  return { phase, completed, endsAt: null, remainingMs: PHASE_MS[phase] };
}

/** Moves on once the running phase has run out; otherwise unchanged. */
export function advance(state: PomodoroState, now: number): PomodoroState {
  if (state.endsAt === null || now < state.endsAt) return state;
  return skip(state);
}
