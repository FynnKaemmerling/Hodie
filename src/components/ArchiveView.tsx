import { ArchiveRestore, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { CardSummary, Id, Tag } from "../api";
import type { CardRef } from "../board/useBoard";
import CardContent from "./CardContent";
import styles from "./ArchiveView.module.css";

interface Props {
  cards: CardSummary[] | null;
  tags: Tag[];
  selectedCardId: Id | null;
  onOpen: (cardId: Id) => void;
  onRestore: (card: CardRef) => void;
  onDelete: (card: CardRef) => void;
}

/** Archived cards of the current board, most recently archived first. */
function ArchiveView({ cards, tags, selectedCardId, onOpen, onRestore, onDelete }: Props) {
  const { t } = useTranslation();
  if (cards === null) return null;

  if (cards.length === 0) {
    return (
      <div className={styles.empty}>
        <p className={styles.emptyTitle}>{t("archive.empty.title")}</p>
        <p className={styles.emptyText}>{t("archive.empty.description")}</p>
      </div>
    );
  }

  return (
    <ul className={styles.list}>
      {cards.map((card) => (
        <li
          key={card.id}
          className={styles.row}
          data-selected={card.id === selectedCardId || undefined}
          data-priority={card.priority}
        >
          <button
            type="button"
            className={styles.card}
            data-card-id={card.id}
            onClick={() => onOpen(card.id)}
          >
            <CardContent card={card} tags={tags} />
          </button>
          <div className={styles.actions}>
            <button type="button" className={styles.action} onClick={() => onRestore(card)}>
              <ArchiveRestore aria-hidden="true" />
              {t("archive.restore")}
            </button>
            <button
              type="button"
              className={styles.action}
              data-danger
              aria-label={t("archive.delete", { title: card.title })}
              onClick={() => onDelete(card)}
            >
              <Trash2 aria-hidden="true" />
              {t("actions.delete")}
            </button>
          </div>
        </li>
      ))}
    </ul>
  );
}

export default ArchiveView;
