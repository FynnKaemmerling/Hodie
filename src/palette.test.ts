import { describe, expect, it } from "vitest";
import {
  BUILTIN_PRESETS,
  contrast,
  contrastIssues,
  derivePalette,
  modeColors,
  parseHex,
} from "./palette";

describe("parseHex", () => {
  it("reads #rrggbb in any case and rejects the rest", () => {
    expect(parseHex("#C0501c")).toEqual([192, 80, 28]);
    expect(parseHex("#fff")).toBeNull();
    expect(parseHex("c0501c")).toBeNull();
    expect(parseHex("#c0501g")).toBeNull();
  });
});

describe("contrast", () => {
  it("matches the WCAG extremes", () => {
    expect(contrast("#000000", "#ffffff")).toBeCloseTo(21, 5);
    expect(contrast("#777777", "#777777")).toBeCloseTo(1, 5);
  });
});

describe("derivePalette", () => {
  it("uses the colours as accent and app background", () => {
    const palette = derivePalette({ accent: "#1b6a88", base: "#f1f5f7" }, false);
    expect(palette["color-accent"]).toBe("#1b6a88");
    expect(palette["color-bg-app"]).toBe("#f1f5f7");
    expect(palette["color-text-on-accent"]).toBe("#ffffff");
  });

  it("puts dark text on a light accent", () => {
    const palette = derivePalette({ accent: "#8fcf9a", base: "#191d19" }, true);
    expect(palette["color-text-on-accent"]).toBe("#1d1f22");
  });

  it("returns nothing for invalid colours", () => {
    expect(derivePalette({ accent: "red", base: "#ffffff" }, false)).toEqual({});
  });
});

describe("contrastIssues", () => {
  it("finds no issues in the built-in presets", () => {
    for (const preset of BUILTIN_PRESETS) {
      for (const dark of [false, true]) {
        expect(contrastIssues(modeColors(preset.colors, dark), dark), preset.name).toEqual([]);
      }
    }
  });

  it("flags a pale accent and a dark base in light mode", () => {
    expect(contrastIssues({ accent: "#ffd27a", base: "#f6f6f4" }, false)).toContain("accentText");
    expect(contrastIssues({ accent: "#c0501c", base: "#30343a" }, false)).toContain("text");
  });
});
