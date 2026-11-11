import { Volume2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNotifications } from "../notifications/Notifications";
import { choosePomodoroSound, usePomodoroSound } from "../pomodoro/soundSetting";
import { isSoundChoice, playSound, POMODORO_SOUNDS, SILENT } from "../pomodoro/sounds";
import IconButton from "./IconButton";
import styles from "./PomodoroSoundSetting.module.css";

/** Picks the sound for the end of a Pomodoro phase, with a button to try it. */
function PomodoroSoundSetting() {
  const { t } = useTranslation();
  const notify = useNotifications();
  const sound = usePomodoroSound();

  function choose(value: string) {
    if (!isSoundChoice(value)) return;
    playSound(value);
    choosePomodoroSound(value).catch(notify.error);
  }

  return (
    <div className={styles.field}>
      <label htmlFor="pomodoro-sound" className={styles.label}>
        {t("settings.pomodoro.sound")}
      </label>
      <div className={styles.controls}>
        <select
          id="pomodoro-sound"
          className={styles.select}
          value={sound}
          onChange={(event) => choose(event.target.value)}
        >
          {POMODORO_SOUNDS.map((name) => (
            <option key={name} value={name}>
              {t(`settings.pomodoro.sounds.${name}`)}
            </option>
          ))}
          <option value={SILENT}>{t("settings.pomodoro.sounds.none")}</option>
        </select>
        <IconButton
          label={t("settings.pomodoro.preview")}
          onClick={() => playSound(sound)}
          disabled={sound === SILENT}
        >
          <Volume2 aria-hidden="true" />
        </IconButton>
      </div>
    </div>
  );
}

export default PomodoroSoundSetting;
