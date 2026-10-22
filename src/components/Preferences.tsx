import { useTranslation } from "react-i18next";
import { changeLanguage, SUPPORTED_LANGUAGES, type Language } from "../i18n";
import {
  setThemePreference,
  THEME_PREFERENCES,
  useThemePreference,
  type ThemePreference,
} from "../theme";
import styles from "./Preferences.module.css";

/** Language and theme settings. */
function Preferences() {
  const { t, i18n } = useTranslation();
  const theme = useThemePreference();

  return (
    <div className={styles.preferences}>
      <label className={styles.field}>
        <span className={styles.label}>{t("settings.language")}</span>
        <select
          className={styles.select}
          value={i18n.resolvedLanguage}
          onChange={(event) => void changeLanguage(event.target.value as Language)}
        >
          {SUPPORTED_LANGUAGES.map((language) => (
            <option key={language} value={language}>
              {t(`languages.${language}`)}
            </option>
          ))}
        </select>
      </label>

      <label className={styles.field}>
        <span className={styles.label}>{t("settings.theme")}</span>
        <select
          className={styles.select}
          value={theme}
          onChange={(event) => setThemePreference(event.target.value as ThemePreference)}
        >
          {THEME_PREFERENCES.map((preference) => (
            <option key={preference} value={preference}>
              {t(`settings.themes.${preference}`)}
            </option>
          ))}
        </select>
      </label>
    </div>
  );
}

export default Preferences;
