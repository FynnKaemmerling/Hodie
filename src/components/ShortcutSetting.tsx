import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, type ShortcutInfo } from "../api";
import { errorMessageKey } from "../api/errors";
import { isMac } from "../platform";
import { formatShortcut, shortcutFromEvent } from "../shortcut";
import styles from "./ShortcutSetting.module.css";

const MODIFIER_KEYS = ["Control", "Alt", "Shift", "Meta", "OS"];

/** Shows the quick entry shortcut and records a new one. */
function ShortcutSetting() {
  const { t } = useTranslation();
  const [info, setInfo] = useState<ShortcutInfo | null>(null);
  const [recording, setRecording] = useState(false);
  const [message, setMessage] = useState<{ text: string; error: boolean } | null>(null);
  const recorder = useRef<HTMLButtonElement>(null);

  // Keys only reach the recorder while it has focus.
  function startRecording() {
    setRecording(true);
    setMessage(null);
    recorder.current?.focus();
  }

  useEffect(() => {
    api
      .getQuickAddShortcut()
      .then(setInfo)
      .catch((error) => setMessage({ text: t(errorMessageKey(error)), error: true }));
  }, [t]);

  async function save(shortcut: string) {
    setRecording(false);
    try {
      const saved = await api.setQuickAddShortcut(shortcut);
      setInfo(saved);
      setMessage({
        text: t("toast.shortcutSaved", { shortcut: formatShortcut(saved.shortcut, t, isMac) }),
        error: false,
      });
    } catch (error) {
      setMessage({ text: t(errorMessageKey(error)), error: true });
    }
  }

  if (!info) return message && <p className={styles.error}>{message.text}</p>;

  return (
    <div className={styles.setting}>
      <div className={styles.row}>
        <span className={styles.label}>{t("settings.shortcut.label")}</span>
        <button
          ref={recorder}
          type="button"
          className={styles.recorder}
          data-recording={recording || undefined}
          aria-describedby="shortcut-message"
          onClick={startRecording}
          onBlur={() => setRecording(false)}
          onKeyDown={(event) => {
            if (!recording) return;
            event.preventDefault();
            event.stopPropagation();
            if (event.key === "Escape") {
              setRecording(false);
              return;
            }
            if (MODIFIER_KEYS.includes(event.key)) return;
            const shortcut = shortcutFromEvent(event);
            if (shortcut) void save(shortcut);
            else setMessage({ text: t("settings.shortcut.needsModifier"), error: true });
          }}
        >
          {recording ? t("settings.shortcut.recording") : <kbd>{formatShortcut(info.shortcut, t, isMac)}</kbd>}
        </button>
      </div>

      <div className={styles.actions}>
        {!recording && (
          <button type="button" className={styles.link} onClick={startRecording}>
            {t("settings.shortcut.change")}
          </button>
        )}
        {info.shortcut !== info.defaultShortcut && (
          <button type="button" className={styles.link} onClick={() => void save(info.defaultShortcut)}>
            {t("settings.shortcut.reset")}
          </button>
        )}
      </div>

      <p id="shortcut-message" className={styles.message} role="status">
        {message ? (
          <span className={message.error ? styles.error : undefined}>{message.text}</span>
        ) : (
          !info.registered && <span className={styles.error}>{t("settings.shortcut.unavailable")}</span>
        )}
      </p>
    </div>
  );
}

export default ShortcutSetting;
