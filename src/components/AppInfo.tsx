import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import { formatBytes } from "../format";
import styles from "./AppInfo.module.css";

/** Version and disk usage, read each time the tab opens. */
function AppInfo() {
  const { t, i18n } = useTranslation();
  const [version, setVersion] = useState<string | null>(null);
  const [usage, setUsage] = useState<number | null>(null);

  useEffect(() => {
    getVersion().then(setVersion, () => setVersion(null));
    api.getStorageUsage().then(setUsage, () => setUsage(null));
  }, []);

  const unknown = t("settings.info.unknown");

  return (
    <dl className={styles.info}>
      <div className={styles.row}>
        <dt className={styles.term}>{t("settings.info.version")}</dt>
        <dd className={styles.value}>{version ?? unknown}</dd>
      </div>
      <div className={styles.row}>
        <dt className={styles.term}>{t("settings.info.storage")}</dt>
        <dd className={styles.value}>
          {usage === null ? unknown : formatBytes(usage, i18n.resolvedLanguage ?? "de")}
        </dd>
      </div>
    </dl>
  );
}

export default AppInfo;
