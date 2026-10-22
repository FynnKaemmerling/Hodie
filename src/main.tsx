import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { NotificationsProvider } from "./notifications/Notifications";
import QuickAdd from "./quickAdd/QuickAdd";
import "./i18n";
import { initTheme } from "./theme";
import "./styles/global.css";

initTheme();

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
