import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { NotificationsProvider } from "./notifications/Notifications";
import QuickAdd from "./quickAdd/QuickAdd";
import "./i18n";
import { isMac, isTextInput } from "./platform";
import { initTheme } from "./theme";
import "./styles/global.css";

initTheme();
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

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {windowLabel() === "quick-add" ? (
      <QuickAdd />
    ) : (
      <NotificationsProvider>
        <App />
      </NotificationsProvider>
    )}
  </React.StrictMode>,
);
