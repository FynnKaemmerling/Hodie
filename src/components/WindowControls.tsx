import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { isMac } from "../platform";
import styles from "./WindowControls.module.css";

/**
 * Minimise, maximise and close for the frameless main window on Windows and
 * Linux. macOS keeps its own traffic lights (see tauri.macos.conf.json).
 */
function WindowControls() {
  const { t } = useTranslation();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    if (isMac) return;
    const appWindow = getCurrentWindow();
    const update = () => void appWindow.isMaximized().then(setMaximized);
    update();
    const unlisten = appWindow.onResized(update);
    return () => void unlisten.then((stop) => stop());
  }, []);

  if (isMac) return null;
  const appWindow = getCurrentWindow();

  return (
    <div className={styles.controls}>
      <button
        type="button"
        className={styles.button}
        aria-label={t("window.minimize")}
        onClick={() => void appWindow.minimize()}
      >
        <Minus aria-hidden="true" />
      </button>
      <button
        type="button"
        className={styles.button}
        aria-label={maximized ? t("window.restore") : t("window.maximize")}
        onClick={() => void appWindow.toggleMaximize()}
      >
        {maximized ? <Copy aria-hidden="true" /> : <Square aria-hidden="true" />}
      </button>
      <button
        type="button"
        className={styles.button}
        data-close
        aria-label={t("window.close")}
        onClick={() => void appWindow.close()}
      >
        <X aria-hidden="true" />
      </button>
    </div>
  );
}

export default WindowControls;
