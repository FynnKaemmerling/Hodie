export const isMac = /Mac|iPhone|iPad/.test(navigator.userAgent);

export const isWindows = /Windows/.test(navigator.userAgent);

/** Windows 11 or newer, the first with the Mica material (Chromium reports 13+ there). */
export async function isWindows11(): Promise<boolean> {
  if (!isWindows) return false;
  try {
    const data = await (navigator as any).userAgentData?.getHighEntropyValues(["platformVersion"]);
    return parseInt(data?.platformVersion ?? "0", 10) >= 13;
  } catch {
    return false;
  }
}

/** "Mod" is Cmd on macOS and Ctrl elsewhere. */
export function isModKey(event: { metaKey: boolean; ctrlKey: boolean }): boolean {
  return isMac ? event.metaKey : event.ctrlKey;
}

/** Label of a Mod shortcut for display, e.g. "⌘F" or "Strg+F". */
export function modShortcut(t: (key: string) => string, key: string): string {
  return isMac ? `⌘${key}` : `${t("keys.ctrl")}+${key}`;
}

/** Whether a key event happens inside a text field, which owns its own keys. */
export function isTextInput(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName))
  );
}
