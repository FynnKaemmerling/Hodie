import { Coffee, Pause, Timer } from "lucide-react";
import { useTranslation } from "react-i18next";
import { formatCountdown } from "../format";
import type { Pomodoro } from "../pomodoro/usePomodoro";
import styles from "./PomodoroBadge.module.css";

/**
 * The running Pomodoro timer in the top right corner of the window, so it
 * stays in sight with the sidebar hidden. A click pauses or resumes it.
 */
function PomodoroBadge({ timer }: { timer: Pomodoro }) {
  const { t, i18n } = useTranslation();
  if (!timer.active) return null;

  const time = formatCountdown(timer.remainingMs, i18n.language);
  const phase = t(`pomodoro.phases.${timer.phase}`);
  const label = t(timer.running ? "pomodoro.badgePause" : "pomodoro.badgeResume", { phase, time });
  const Icon = !timer.running ? Pause : timer.phase === "focus" ? Timer : Coffee;

  return (
    <button
      type="button"
      className={styles.badge}
      data-phase={timer.phase}
      data-paused={!timer.running || undefined}
      aria-label={label}
      title={label}
      onClick={timer.toggle}
    >
      <Icon aria-hidden="true" />
      <span className={styles.time}>{time}</span>
    </button>
  );
}

export default PomodoroBadge;
