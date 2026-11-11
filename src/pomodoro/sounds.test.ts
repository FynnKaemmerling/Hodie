import { describe, expect, it } from "vitest";
import { DEFAULT_SOUND, isSoundChoice, POMODORO_SOUNDS, SILENT, SOUNDS } from "./sounds";

describe("pomodoro sounds", () => {
  it("offers ten sounds, each short and audible", () => {
    expect(POMODORO_SOUNDS).toHaveLength(10);
    for (const name of POMODORO_SOUNDS) {
      const tones = SOUNDS[name];
      expect(tones.length).toBeGreaterThan(0);
      for (const tone of tones) {
        expect(tone.frequency).toBeGreaterThan(20);
        expect(tone.frequency).toBeLessThan(10_000);
        expect(tone.at + tone.duration).toBeLessThanOrEqual(4);
        expect(tone.volume ?? 0.18).toBeLessThanOrEqual(0.3);
      }
    }
  });

  it("accepts the sounds and silence, nothing else", () => {
    expect(isSoundChoice(DEFAULT_SOUND)).toBe(true);
    expect(isSoundChoice(SILENT)).toBe(true);
    expect(isSoundChoice("bell")).toBe(true);
    expect(isSoundChoice("trumpet")).toBe(false);
    expect(isSoundChoice(null)).toBe(false);
  });
});
