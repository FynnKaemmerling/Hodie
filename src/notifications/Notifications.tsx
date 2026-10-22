import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { errorMessageKey } from "../api/errors";
import { isModKey, isTextInput } from "../platform";
import styles from "./Notifications.module.css";

const VISIBLE_MS = 6000;

type Undo = () => Promise<void> | void;

interface Toast {
  id: number;
  message: string;
  kind: "info" | "error";
  undo?: Undo;
}

interface Notifications {
  /** Shows a message; with `undo`, offers "Undo" (also via Mod+Z). */
  info: (message: string, undo?: Undo) => void;
  /** Shows the translated message for a rejected command. */
  error: (error: unknown) => void;
}

const NotificationsContext = createContext<Notifications | null>(null);

export function useNotifications(): Notifications {
  const context = useContext(NotificationsContext);
  if (!context) throw new Error("useNotifications needs a NotificationsProvider");
  return context;
}

export function NotificationsProvider({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  const [toast, setToast] = useState<Toast | null>(null);
  // The last undo stays available for Mod+Z after its toast has faded.
  const lastUndo = useRef<Undo | null>(null);
  const nextId = useRef(0);

  const show = useCallback((toast: Omit<Toast, "id">) => {
    setToast({ ...toast, id: nextId.current++ });
  }, []);

  const runUndo = useCallback(() => {
    const undo = lastUndo.current;
    if (!undo) return;
    lastUndo.current = null;
    setToast(null);
    void undo();
  }, []);

  const notifications = useMemo<Notifications>(
    () => ({
      info: (message, undo) => {
        lastUndo.current = undo ?? null;
        show({ message, kind: "info", undo });
      },
      error: (error) => {
        console.error(error);
        show({ message: t(errorMessageKey(error)), kind: "error" });
      },
    }),
    [show, t],
  );

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), VISIBLE_MS);
    return () => window.clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key.toLowerCase() !== "z" || !isModKey(event) || event.shiftKey) return;
      if (isTextInput(event.target) || !lastUndo.current) return;
      event.preventDefault();
      runUndo();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [runUndo]);

  return (
    <NotificationsContext.Provider value={notifications}>
      {children}
      <div className={styles.region} role="status" aria-live="polite">
        {toast && (
          <div key={toast.id} className={styles.toast} data-kind={toast.kind}>
            <span className={styles.message}>{toast.message}</span>
            {toast.undo && (
              <button type="button" className={styles.action} onClick={runUndo}>
                {t("actions.undo")}
              </button>
            )}
          </div>
        )}
      </div>
    </NotificationsContext.Provider>
  );
}
