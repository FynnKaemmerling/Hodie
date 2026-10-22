import { useTranslation } from "react-i18next";
import LanguageSwitcher from "./components/LanguageSwitcher";
import Sidebar from "./components/Sidebar";
import styles from "./App.module.css";

function App() {
  const { t } = useTranslation();

  return (
    <div className={styles.app}>
      <Sidebar />

      <div className={styles.main}>
        <header className={styles.toolbar}>
          <input
            className={styles.search}
            type="search"
            placeholder={t("toolbar.search")}
            aria-label={t("toolbar.search")}
          />
          <LanguageSwitcher />
        </header>

        <main className={styles.content}>
          <div className={styles.empty}>
            <h1 className={styles.emptyTitle}>{t("home.empty.title")}</h1>
            <p className={styles.emptyText}>{t("home.empty.description")}</p>
          </div>
        </main>

        <footer className={styles.statusbar}>{t("status.ready")}</footer>
      </div>
    </div>
  );
}

export default App;
