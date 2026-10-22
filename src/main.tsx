import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { NotificationsProvider } from "./notifications/Notifications";
import "./i18n";
import "./styles/global.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <NotificationsProvider>
      <App />
    </NotificationsProvider>
  </React.StrictMode>,
);
