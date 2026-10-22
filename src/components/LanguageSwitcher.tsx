import { useTranslation } from "react-i18next";
import { changeLanguage, SUPPORTED_LANGUAGES, type Language } from "../i18n";
import styles from "./LanguageSwitcher.module.css";

function LanguageSwitcher() {
  const { t, i18n } = useTranslation();

  return (
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
  );
}

export default LanguageSwitcher;
