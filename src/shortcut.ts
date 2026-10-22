// Global shortcuts are stored in the backend's format, e.g.
// "Control+Shift+Space": modifiers, then a KeyboardEvent.code.

const MODIFIERS = ["Control", "Alt", "Shift", "Super"] as const;
type Modifier = (typeof MODIFIERS)[number];

/** Keys that can end a shortcut, by KeyboardEvent.code. */
function isAllowedKey(code: string): boolean {
  return (
    /^Key[A-Z]$/.test(code) ||
    /^Digit[0-9]$/.test(code) ||
    /^F([1-9]|1[0-9]|2[0-4])$/.test(code) ||
    /^Arrow(Up|Down|Left|Right)$/.test(code) ||
    ["Space", "Enter", "Backquote", "Minus", "Equal", "Comma", "Period", "Slash"].includes(code)
  );
}

interface KeyEventLike {
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}

/**
 * The shortcut a key press describes, or `null` while it is incomplete or
 * would clash with typing: it needs Ctrl, Alt or Super/Cmd plus one key.
 */
export function shortcutFromEvent(event: KeyEventLike): string | null {
  if (!isAllowedKey(event.code)) return null;
  const pressed: Record<Modifier, boolean> = {
    Control: event.ctrlKey,
    Alt: event.altKey,
    Shift: event.shiftKey,
    Super: event.metaKey,
  };
  if (!pressed.Control && !pressed.Alt && !pressed.Super) return null;
  const modifiers = MODIFIERS.filter((modifier) => pressed[modifier]);
  return [...modifiers, event.code].join("+");
}

const KEY_LABELS: Record<string, string> = {
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  Backquote: "`",
  Minus: "-",
  Equal: "=",
  Comma: ",",
  Period: ".",
  Slash: "/",
};

/** A shortcut for display, e.g. "Strg+Umschalt+Leertaste" or "⇧⌘Space". */
export function formatShortcut(
  shortcut: string,
  t: (key: string) => string,
  mac: boolean,
): string {
  const labels = shortcut.split("+").map((part) => {
    switch (part) {
      case "Control":
        return mac ? "⌃" : t("keys.ctrl");
      case "Alt":
        return mac ? "⌥" : "Alt";
      case "Shift":
        return mac ? "⇧" : t("keys.shift");
      case "Super":
        return mac ? "⌘" : "Win";
      case "Space":
        return t("keys.space");
      case "Enter":
        return t("keys.enter");
      default:
        return KEY_LABELS[part] ?? part.replace(/^Key|^Digit/, "");
    }
  });
  return labels.join(mac ? "" : "+");
}
