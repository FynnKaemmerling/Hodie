import { describe, expect, it } from "vitest";
import {
  advance,
  INITIAL_POMODORO,
  isActive,
  isRunning,
  pause,
  PHASE_MS,
  remainingMs,
  reset,
  skip,
  start,
} from "./pomodoro";

describe("pomodoro", () => {
  it("counts down from the end time while running", () => {
    const running = start(INITIAL_POMODORO, 1_000);
    expect(isRunning(running)).toBe(true);
    expect(remainingMs(running, 1_000 + 60_000)).toBe(PHASE_MS.focus - 60_000);
  });

  it("keeps the time left while paused and resumes from it", () => {
    const paused = pause(start(INITIAL_POMODORO, 0), 10_000);
    expect(isRunning(paused)).toBe(false);
    expect(isActive(paused)).toBe(true);
    expect(remainingMs(paused, 999_999)).toBe(PHASE_MS.focus - 10_000);
    const resumed = start(paused, 50_000);
    expect(remainingMs(resumed, 50_000)).toBe(PHASE_MS.focus - 10_000);
  });

  it("is not active before it starts or after a reset", () => {
    expect(isActive(INITIAL_POMODORO)).toBe(false);
    expect(isActive(reset(pause(start(INITIAL_POMODORO, 0), 5_000)))).toBe(false);
  });

  it("moves to a break once focus runs out, stopped", () => {
    const running = start(INITIAL_POMODORO, 0);
    expect(advance(running, PHASE_MS.focus - 1)).toBe(running);
    const next = advance(running, PHASE_MS.focus);
    expect(next.phase).toBe("shortBreak");
    expect(next.completed).toBe(1);
    expect(isRunning(next)).toBe(false);
    expect(next.remainingMs).toBe(PHASE_MS.shortBreak);
  });

  it("takes a long break after every fourth focus session", () => {
    let state = INITIAL_POMODORO;
    const breaks: string[] = [];
    for (let i = 0; i < 8; i++) {
      state = skip(state);
      breaks.push(state.phase);
      state = skip(state);
      expect(state.phase).toBe("focus");
    }
    expect(breaks).toEqual([
      "shortBreak",
      "shortBreak",
      "shortBreak",
      "longBreak",
      "shortBreak",
      "shortBreak",
      "shortBreak",
      "longBreak",
    ]);
  });
});
