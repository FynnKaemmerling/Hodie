export const isMac = /Mac|iPhone|iPad/.test(navigator.userAgent);

/** "Mod" is Cmd on macOS and Ctrl elsewhere. */
export function isModKey(event: { metaKey: boolean; ctrlKey: boolean }): boolean {
  return isMac ? event.metaKey : event.ctrlKey;
}

/** Whether a key event happens inside a text field, which owns its own keys. */
export function isTextInput(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName))
  );
}
