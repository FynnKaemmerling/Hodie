import type { PresetColors } from "./api/types";

/** Accent and base colour for one mode. */
export interface ModeColors {
  accent: string;
  base: string;
}

export type BuiltinPresetName = "hodie" | "ocean" | "forest";

export interface BuiltinPreset {
  name: BuiltinPresetName;
  colors: PresetColors;
}

/**
 * Presets that ship with the app; names match BUILTIN_PRESETS in
 * src-tauri/src/model.rs. "hodie" is the hand-tuned palette in tokens.css,
 * the others are derived like custom presets.
 */
export const BUILTIN_PRESETS: BuiltinPreset[] = [
  {
    name: "hodie",
    colors: {
      lightAccent: "#c0501c",
      lightBase: "#f6f6f4",
      darkAccent: "#f0a06a",
      darkBase: "#1b1d20",
    },
  },
  {
    name: "ocean",
    colors: {
      lightAccent: "#1b6a88",
      lightBase: "#f1f5f7",
      darkAccent: "#6cc4e0",
      darkBase: "#161b20",
    },
  },
  {
    name: "forest",
    colors: {
      lightAccent: "#38734a",
      lightBase: "#f3f5f0",
      darkAccent: "#8fcf9a",
      darkBase: "#191d19",
    },
  },
];

export const DEFAULT_PRESET: BuiltinPresetName = "hodie";

export function modeColors(colors: PresetColors, dark: boolean): ModeColors {
  return dark
    ? { accent: colors.darkAccent, base: colors.darkBase }
    : { accent: colors.lightAccent, base: colors.lightBase };
}

type Rgb = [number, number, number];

const WHITE: Rgb = [255, 255, 255];
const BLACK: Rgb = [0, 0, 0];
// Must match tokens.css: text and the main surface (board area, cards,
// panel) stay neutral in every preset.
const TEXT_LIGHT = "#33373b";
const TEXT_DARK = "#e4e5e7";
const SURFACE_LIGHT = "#ffffff";
const SURFACE_DARK = "#212428";
// Dark text on light accents, as in the dark set of tokens.css.
const ON_ACCENT_DARK = "#1d1f22";
const AA = 4.5;

export function parseHex(hex: string): Rgb | null {
  const match = /^#([0-9a-f]{6})$/i.exec(hex);
  if (!match) return null;
  const value = parseInt(match[1], 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}

function toHex(rgb: Rgb): string {
  return `#${rgb.map((c) => Math.round(c).toString(16).padStart(2, "0")).join("")}`;
}

/** `amount` of `to` mixed into `from`. */
function mix(from: Rgb, to: Rgb, amount: number): Rgb {
  return [0, 1, 2].map((i) => from[i] + (to[i] - from[i]) * amount) as Rgb;
}

function alpha(rgb: Rgb, value: number): string {
  return `rgb(${rgb.map(Math.round).join(" ")} / ${value})`;
}

function luminance(rgb: Rgb): number {
  const [r, g, b] = rgb.map((c) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG contrast ratio between two colours. */
export function contrast(a: string, b: string): number {
  const x = parseHex(a);
  const y = parseHex(b);
  if (!x || !y) return 1;
  const [hi, lo] = [luminance(x), luminance(y)].sort((p, q) => q - p);
  return (hi + 0.05) / (lo + 0.05);
}

function textOnAccent(accent: string): string {
  return contrast(accent, toHex(WHITE)) >= contrast(accent, ON_ACCENT_DARK)
    ? toHex(WHITE)
    : ON_ACCENT_DARK;
}

/**
 * Token values (names without the leading "--") for one mode, derived from
 * an accent and a base colour. The main surface, text, status, priority and
 * tag colours are not touched, so they keep the contrast checked in tokens.css.
 */
export function derivePalette({ accent, base }: ModeColors, dark: boolean): Record<string, string> {
  const a = parseHex(accent);
  const b = parseHex(base);
  if (!a || !b) return {};
  // Light surfaces darken the base, dark ones lighten it.
  const shade = (amount: number) => toHex(mix(b, dark ? WHITE : BLACK, amount));
  const surface = parseHex(dark ? SURFACE_DARK : SURFACE_LIGHT)!;
  const focus = dark ? a : mix(a, WHITE, 0.2);

  return {
    "color-bg-app": toHex(b),
    "color-bg-sidebar-tint": alpha(b, 0.6),
    "color-bg-sunken": shade(dark ? 0.06 : 0.03),
    "color-bg-hover": shade(dark ? 0.08 : 0.045),
    "color-bg-selected": toHex(mix(dark ? b : surface, a, dark ? 0.15 : 0.14)),
    "color-bg-column": shade(dark ? 0.045 : 0.03),
    "color-bg-count": shade(dark ? 0.13 : 0.085),
    "color-border": shade(dark ? 0.1 : 0.075),
    "color-border-strong": shade(dark ? 0.2 : 0.16),
    "color-border-card": shade(dark ? 0.12 : 0.065),
    "color-canvas-dot": shade(dark ? 0.07 : 0.11),
    "color-accent": toHex(a),
    "color-accent-hover": toHex(dark ? mix(a, WHITE, 0.2) : mix(a, BLACK, 0.14)),
    "color-focus-ring": toHex(focus),
    "color-focus-ring-soft": alpha(focus, dark ? 0.3 : 0.25),
    "color-accent-soft": alpha(a, dark ? 0.16 : 0.12),
    "color-accent-on-inverse": toHex(dark ? mix(a, BLACK, 0.3) : mix(a, WHITE, 0.4)),
    "color-text-on-accent": textOnAccent(toHex(a)),
  };
}

export type ContrastIssue = "text" | "accentText" | "onAccent";

/** Pairs below WCAG AA (4.5:1) that a preset would cause in one mode. */
export function contrastIssues(colors: ModeColors, dark: boolean): ContrastIssue[] {
  const palette = derivePalette(colors, dark);
  const surface = dark ? SURFACE_DARK : SURFACE_LIGHT;
  const accent = palette["color-accent"];
  if (!accent) return [];
  const text = dark ? TEXT_DARK : TEXT_LIGHT;
  const issues: ContrastIssue[] = [];
  if (
    contrast(text, surface) < AA ||
    contrast(text, palette["color-bg-app"]) < AA ||
    contrast(text, palette["color-bg-column"]) < AA
  ) {
    issues.push("text");
  }
  if (contrast(accent, surface) < AA) issues.push("accentText");
  if (contrast(palette["color-text-on-accent"], accent) < AA) issues.push("onAccent");
  return issues;
}
