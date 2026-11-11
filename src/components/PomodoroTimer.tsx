import { Pause, Play, RotateCcw, SkipForward } from "lucide-react";
import { useTranslation } from "react-i18next";
import { formatCountdown } from "../format";
import type { Pomodoro } from "../pomodoro/usePomodoro";
import IconButton from "./IconButton";
import styles from "./PomodoroTimer.module.css";

/** The Pomodoro timer in the sidebar: time left, phase and its controls. */
function PomodoroTimer({ timer }: { timer: Pomodoro }) {
  const { t, i18n } = useTranslation();

  return (
    <section className={styles.timer} aria-labelledby="pomodoro-title">
      <h2 id="pomodoro-title" className={styles.title}>
        {t("pomodoro.title")}
      </h2>
      <div className={styles.row} data-phase={timer.phase} data-running={timer.running || undefined}>
        <div className={styles.readout}>
          <span className={styles.time} role="timer">
            {formatCountdown(timer.remainingMs, i18n.language)}
          </span>
          <span className={styles.phase}>{t(`pomodoro.phases.${timer.phase}`)}</span>
        </div>
        <IconButton
          label={timer.running ? t("pomodoro.pause") : t("pomodoro.start")}
          active={timer.running}
          onClick={timer.toggle}
        >
          {timer.running ? <Pause aria-hidden="true" /> : <Play aria-hidden="true" />}
        </IconButton>
        <IconButton label={t("pomodoro.reset")} onClick={timer.reset} disabled={!timer.active}>
          <RotateCcw aria-hidden="true" />
        </IconButton>
        <IconButton label={t("pomodoro.skip")} onClick={timer.skip}>
          <SkipForward aria-hidden="true" />
        </IconButton>
      </div>
    </section>
  );
}

export default PomodoroTimer;
