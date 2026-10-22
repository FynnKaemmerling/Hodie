import { Globe, SunMoon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { changeLanguage, SUPPORTED_LANGUAGES, type Language } from "../i18n";
import {
  setThemePreference,
  THEME_PREFERENCES,
  useThemePreference,
  type ThemePreference,
} from "../theme";
import styles from "./Preferences.module.css";

/** Language and theme, at the bottom of the sidebar. */
function Preferences() {
  const { t, i18n } = useTranslation();
  const theme = useThemePreference();

  return (
    <div className={styles.preferences}>
      <label className={styles.field}>
        <Globe aria-hidden="true" />
        <select
          className={styles.select}
          value={i18n.resolvedLanguage}
          onChange={(event) => void changeLanguage(event.target.value as Language)}
          aria-label={t("settings.language")}
        >
          {SUPPORTED_LANGUAGES.map((language) => (
            <option key={language} value={language}>
              {t(`languages.${language}`)}
            </option>
          ))}
        </select>
      </label>

      <label className={styles.field}>
        <SunMoon aria-hidden="true" />
        <select
          className={styles.select}
          value={theme}
          onChange={(event) => setThemePreference(event.target.value as ThemePreference)}
          aria-label={t("settings.theme")}
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
