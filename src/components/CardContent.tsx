import { useTranslation } from "react-i18next";
import type { CardSummary, Tag } from "../api";
import styles from "./CardContent.module.css";

const PRIORITY_MARKS = { none: "", low: "!", medium: "!!", high: "!!!" } as const;

interface Props {
  card: CardSummary;
  tags: Tag[];
}

/** What a card shows on the board. Empty fields take no space. */
function CardContent({ card, tags }: Props) {
  const { t } = useTranslation();
  const cardTags = card.tagIds
    .map((id) => tags.find((tag) => tag.id === id))
    .filter((tag): tag is Tag => tag !== undefined);
  const hasMeta =
    card.priority !== "none" || cardTags.length > 0 || card.subtasksTotal > 0 || card.hasNotes;
  const allSubtasksDone = card.subtasksTotal > 0 && card.subtasksDone === card.subtasksTotal;

  return (
    <>
      <span className={styles.title}>{card.title}</span>
      {hasMeta && (
        <span className={styles.meta}>
          {card.priority !== "none" && (
            <span className={styles.priority} data-priority={card.priority}>
              <span aria-hidden="true">{PRIORITY_MARKS[card.priority]}</span>
              <span className="visually-hidden">{t(`card.priority.${card.priority}`)}</span>
            </span>
          )}
          {cardTags.map((tag) => (
            <span key={tag.id} className={styles.tag} data-color={tag.color}>
              {tag.name}
            </span>
          ))}
          {card.subtasksTotal > 0 && (
            <span className={styles.badge} data-complete={allSubtasksDone || undefined}>
              <span aria-hidden="true">
                ✓ {card.subtasksDone}/{card.subtasksTotal}
              </span>
              <span className="visually-hidden">
                {t("card.subtasks", { done: card.subtasksDone, total: card.subtasksTotal })}
              </span>
            </span>
          )}
          {card.hasNotes && (
            <span className={styles.badge}>
              <span aria-hidden="true">≡</span>
              <span className="visually-hidden">{t("card.hasNotes")}</span>
            </span>
          )}
        </span>
      )}
    </>
  );
}

export default CardContent;
