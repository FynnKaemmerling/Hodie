/**
 * The sounds the Pomodoro timer can play when a phase ends, synthesised with
 * the Web Audio API so no sound files have to ship with the app.
 */

export const POMODORO_SOUNDS = [
  "chime",
  "bell",
  "marimba",
  "ding",
  "beep",
  "descend",
  "arpeggio",
  "gong",
  "digital",
  "harp",
] as const;

export type PomodoroSound = (typeof POMODORO_SOUNDS)[number];
/** The choice "no sound" next to the sounds themselves. */
export const SILENT = "none";
export type PomodoroSoundChoice = PomodoroSound | typeof SILENT;
export const DEFAULT_SOUND: PomodoroSoundChoice = "chime";

export function isSoundChoice(value: unknown): value is PomodoroSoundChoice {
  return value === SILENT || (POMODORO_SOUNDS as readonly unknown[]).includes(value);
}

export interface Tone {
  /** Hz */
  frequency: number;
  /** Start, in seconds after the sound begins. */
  at: number;
  /** How long it takes to fade out, in seconds. */
  duration: number;
  type?: OscillatorType;
  /** Peak gain, 0 to 1. */
  volume?: number;
}

const DEFAULT_VOLUME = 0.18;

// Note frequencies (Hz) used below.
const C4 = 261.63;
const G3 = 196.0;
const C5 = 523.25;
const D5 = 587.33;
const E5 = 659.25;
const G5 = 783.99;
const A5 = 880.0;
const B5 = 987.77;
const C6 = 1046.5;

function sequence(
  frequencies: number[],
  step: number,
  duration: number,
  type: OscillatorType = "sine",
  volume = DEFAULT_VOLUME,
): Tone[] {
  return frequencies.map((frequency, i) => ({ frequency, at: i * step, duration, type, volume }));
}

export const SOUNDS: Record<PomodoroSound, Tone[]> = {
  // Two notes a fifth apart, the second one rising.
  chime: sequence([E5, B5], 0.18, 1.2),
  // A struck bell: fundamental plus quieter inharmonic overtones.
  bell: [
    { frequency: C6, at: 0, duration: 2.2, volume: 0.16 },
    { frequency: C6 * 2.76, at: 0, duration: 1.0, volume: 0.05 },
    { frequency: C6 * 5.4, at: 0, duration: 0.5, volume: 0.025 },
  ],
  marimba: sequence([C5, E5, G5], 0.14, 0.45, "triangle", 0.28),
  ding: sequence([A5], 0, 0.9),
  beep: sequence([A5, A5], 0.22, 0.15, "triangle", 0.22),
  descend: sequence([G5, E5, C5], 0.2, 0.9),
  arpeggio: sequence([C5, E5, G5, C6], 0.12, 0.8),
  // Low and long, with a fifth above for body.
  gong: [
    { frequency: G3, at: 0, duration: 3.2, volume: 0.24 },
    { frequency: C4 * 1.5, at: 0, duration: 2.4, volume: 0.1 },
    { frequency: G3 * 2.4, at: 0.01, duration: 1.4, volume: 0.05 },
  ],
  digital: sequence([C6, C6, C6], 0.16, 0.08, "square", 0.06),
  harp: sequence([C5, D5, E5, G5, A5, C6], 0.06, 1.0, "triangle", 0.2),
};

let context: AudioContext | null = null;

/**
 * Creates or wakes the audio context. Webviews only allow sound after a user
 * gesture, so this runs when the timer is started; the sound that plays
 * minutes later then reuses the unlocked context.
 */
export function unlockSound(): void {
  try {
    context ??= new AudioContext();
    if (context.state === "suspended") void context.resume();
  } catch {
    context = null;
  }
}

/** Plays a sound; silently does nothing for "none" or without audio. */
export function playSound(choice: PomodoroSoundChoice): void {
  if (choice === SILENT) return;
  try {
    unlockSound();
    if (!context) return;
    const start = context.currentTime + 0.02;
    for (const tone of SOUNDS[choice]) {
      const oscillator = context.createOscillator();
      const gain = context.createGain();
      oscillator.type = tone.type ?? "sine";
      oscillator.frequency.value = tone.frequency;
      // Quick attack, exponential fade like a struck note.
      const at = start + tone.at;
      gain.gain.setValueAtTime(0.0001, at);
      gain.gain.exponentialRampToValueAtTime(tone.volume ?? DEFAULT_VOLUME, at + 0.01);
      gain.gain.exponentialRampToValueAtTime(0.0001, at + tone.duration);
      oscillator.connect(gain).connect(context.destination);
      oscillator.start(at);
      oscillator.stop(at + tone.duration);
    }
  } catch {
    // No sound then; the toast and system notification still tell the user.
  }
}
