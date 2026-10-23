import { TriangleAlert } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, type CardVersion, type Id } from "../api";
import { useNotifications } from "../notifications/Notifications";
import styles from "./CardConflict.module.css";

interface Props {
  cardId: Id;
  /** One version was kept; whatever shows the card should reload. */
  onResolved: () => void;
}

/**
 * The other versions of a card from conflict copies of its file (a sync
 * service made them when the card changed in two places at once). The
 * current version is the one the panel shows; one of them is kept.
 */
function CardConflict({ cardId, onResolved }: Props) {
  const { t, i18n } = useTranslation();
  const notify = useNotifications();
  const [versions, setVersions] = useState<CardVersion[] | null>(null);
  const dateFormat = new Intl.DateTimeFormat(i18n.language, {
    dateStyle: "medium",
    timeStyle: "short",
  });

  useEffect(() => {
    api.getCardConflicts(cardId).then(setVersions, notify.error);
  }, [cardId, notify]);

  async function keep(path: string | null) {
    try {
      await api.resolveCardConflict(cardId, path);
      onResolved();
    } catch (error) {
      notify.error(error);
    }
  }

  if (!versions || versions.length === 0) return null;

  return (
    <section className={styles.conflict} aria-labelledby="card-conflict-title">
      <h3 id="card-conflict-title" className={styles.heading}>
        <TriangleAlert aria-hidden="true" />
        {t("conflict.title")}
      </h3>
      <p className={styles.text}>{t("conflict.description")}</p>
      {versions.map((version) => (
        <div key={version.path} className={styles.version}>
          <p className={styles.versionTitle}>{version.title}</p>
          <p className={styles.meta}>
            {t("conflict.changed", { date: dateFormat.format(version.updatedAt) })}
          </p>
          {version.notes && <p className={styles.notes}>{version.notes}</p>}
          <button type="button" className={styles.action} onClick={() => void keep(version.path)}>
            {t("conflict.takeOther")}
          </button>
        </div>
      ))}
      <button type="button" className={styles.action} onClick={() => void keep(null)}>
        {t("conflict.keepCurrent")}
      </button>
    </section>
  );
}

export default CardConflict;
