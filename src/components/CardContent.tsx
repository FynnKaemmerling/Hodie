import { useTranslation } from "react-i18next";
import type { CardSummary, Tag } from "../api";
import TagChip from "./TagChip";
import styles from "./CardContent.module.css";

interface Props {
  card: CardSummary;
  tags: Tag[];
}

/**
 * What a card shows on the board. Empty fields take no space. The priority
 * is shown by the tile around it (see `data-priority`), so here it is only
 * spelled out for screen readers.
 */
function CardContent({ card, tags }: Props) {
  const { t } = useTranslation();
  const cardTags = card.tagIds
    .map((id) => tags.find((tag) => tag.id === id))
    .filter((tag): tag is Tag => tag !== undefined);
  const hasMeta = cardTags.length > 0 || card.subtasksTotal > 0;
  const allSubtasksDone = card.subtasksTotal > 0 && card.subtasksDone === card.subtasksTotal;

  return (
    <>
      <span className={styles.title}>{card.title}</span>
      {card.priority !== "none" && (
        <span className="visually-hidden">{t(`card.priority.${card.priority}`)}</span>
      )}
      {hasMeta && (
        <span className={styles.meta}>
          {cardTags.map((tag) => (
            <TagChip key={tag.id} color={tag.color} size="small">
              {tag.name}
            </TagChip>
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
        </span>
      )}
    </>
  );
}

export default CardContent;
