import React from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { VAULT_CHANGED } from "./api/events";
import { initPalette, syncPalette } from "./colorPreset";
import { NotificationsProvider } from "./notifications/Notifications";
import QuickAdd from "./quickAdd/QuickAdd";
import VaultGate from "./components/VaultGate";
import "./i18n";
import { isMac, isTextInput, isWindows11 } from "./platform";
import { initTheme } from "./theme";
import "./styles/global.css";

initTheme();
initPalette();
document.documentElement.dataset.platform = isMac ? "mac" : "other";

// The webview's own menu (Back, Reload, Inspect …) does not belong in an app.
// Text fields keep it for cut, copy and paste; elements with actions show
// their own menu (see useContextMenu).
document.addEventListener("contextmenu", (event) => {
  if (!isTextInput(event.target)) event.preventDefault();
});

// Both windows load this bundle; the label from tauri.conf.json decides which UI shows.
function windowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    return "main";
  }
}

document.documentElement.dataset.window = windowLabel();
// The macOS main window has a translucent sidebar (see tauri.macos.conf.json).
if (isMac && windowLabel() === "main") document.documentElement.dataset.vibrancy = "";
// Windows 11 gets the Mica material instead (tauri.windows.conf.json). Older versions keep an opaque window.
if (windowLabel() === "main") {
  void isWindows11().then((mica) => {
    if (mica) document.documentElement.dataset.vibrancy = "";
  });
}
if (windowLabel() === "main") void syncPalette();

// Every board, card and tag id belongs to the vault; after a switch both
// windows simply start over.
void listen(VAULT_CHANGED, () => window.location.reload());

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {windowLabel() === "quick-add" ? (
      <QuickAdd />
    ) : (
      <NotificationsProvider>
        <VaultGate>
          <App />
        </VaultGate>
      </NotificationsProvider>
    )}
  </React.StrictMode>,
);
