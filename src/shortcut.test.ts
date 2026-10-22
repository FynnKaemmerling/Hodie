import { describe, expect, it } from "vitest";
import { formatShortcut, shortcutFromEvent } from "./shortcut";

const press = (code: string, mods: Partial<Record<"ctrl" | "alt" | "shift" | "meta", boolean>> = {}) => ({
  code,
  ctrlKey: mods.ctrl ?? false,
  altKey: mods.alt ?? false,
  shiftKey: mods.shift ?? false,
  metaKey: mods.meta ?? false,
});

describe("shortcutFromEvent", () => {
  it("builds the backend format with modifiers in a fixed order", () => {
    expect(shortcutFromEvent(press("Space", { shift: true, ctrl: true }))).toBe(
      "Control+Shift+Space",
    );
    expect(shortcutFromEvent(press("KeyK", { meta: true, alt: true }))).toBe("Alt+Super+KeyK");
    expect(shortcutFromEvent(press("F5", { ctrl: true }))).toBe("Control+F5");
  });

  it("needs Ctrl, Alt or Super and a real key", () => {
    expect(shortcutFromEvent(press("KeyK"))).toBeNull();
    expect(shortcutFromEvent(press("KeyK", { shift: true }))).toBeNull();
    expect(shortcutFromEvent(press("ControlLeft", { ctrl: true }))).toBeNull();
    expect(shortcutFromEvent(press("Escape", { ctrl: true }))).toBeNull();
  });
});

describe("formatShortcut", () => {
  const t = (key: string) =>
    ({ "keys.ctrl": "Strg", "keys.shift": "Umschalt", "keys.space": "Leertaste", "keys.enter": "Enter" })[
      key
    ] ?? key;

  it("uses words on Windows and Linux", () => {
    expect(formatShortcut("Control+Shift+Space", t, false)).toBe("Strg+Umschalt+Leertaste");
    expect(formatShortcut("Alt+Super+KeyK", t, false)).toBe("Alt+Win+K");
    expect(formatShortcut("Control+Digit1", t, false)).toBe("Strg+1");
  });

  it("uses symbols on macOS", () => {
    expect(formatShortcut("Super+Shift+Space", t, true)).toBe("⌘⇧Leertaste");
    expect(formatShortcut("Control+ArrowUp", t, true)).toBe("⌃↑");
  });
});
