import { useTranslation } from "react-i18next";
import styles from "./Sidebar.module.css";

// Placeholder navigation until the real views exist.
const ITEMS = ["home"] as const;

function Sidebar() {
  const { t } = useTranslation();

  return (
    <nav className={styles.sidebar} aria-label={t("nav.title")}>
      <div className={styles.header}>
        <span className={styles.appName}>{t("app.name")}</span>
      </div>

      <ul className={styles.items}>
        {ITEMS.map((item) => (
          <li key={item}>
            <button
              type="button"
              className={styles.item}
              aria-current={item === "home" ? "page" : undefined}
            >
              {t(`nav.${item}`)}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  );
}

export default Sidebar;
