import { useEffect, useRef, useState } from "react";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useTranslation } from "react-i18next";
import type { CardSummary, Id, Tag } from "../api";
import type { Direction } from "../board/boardState";
import { dndId, type DragData } from "../board/dnd";
import { isMac } from "../platform";
import { useContextMenu } from "./ActionMenu";
import CardContent from "./CardContent";
import InlineInput from "./InlineInput";
import styles from "./BoardCard.module.css";

const ARROWS: Record<string, Direction> = {
  ArrowUp: "up",
  ArrowDown: "down",
  ArrowLeft: "left",
  ArrowRight: "right",
};

interface Props {
  card: CardSummary;
  tags: Tag[];
  onRename: (cardId: Id, title: string) => void;
  onDelete: (card: CardSummary) => void;
  onArchive: (card: CardSummary) => void;
  onNavigate: (cardId: Id, direction: Direction) => void;
  onOpen: (cardId: Id) => void;
  selected: boolean;
  dragDisabled: boolean;
}

// A drop can end with a click on the card; that click must not open it.
const CLICK_AFTER_DRAG_MS = 250;

function BoardCard({
  card,
  tags,
  onRename,
  onDelete,
  onArchive,
  onNavigate,
  onOpen,
  selected,
  dragDisabled,
}: Props) {
  const { t } = useTranslation();
  const [editing, setEditing] = useState(false);
  const lastDrag = useRef(0);
  const data: DragData = { kind: "card", id: card.id, columnId: card.columnId };
  const {
    attributes,
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: dndId("card", card.id), data, disabled: editing || dragDisabled });

  const contextMenu = useContextMenu([
    { id: "open", label: t("card.open"), onAction: () => onOpen(card.id) },
    { id: "rename", label: t("actions.rename"), onAction: () => setEditing(true) },
    { id: "archive", label: t("panel.archive"), onAction: () => onArchive(card) },
    { id: "delete", label: t("actions.delete"), danger: true, onAction: () => onDelete(card) },
  ]);

  useEffect(() => {
    if (isDragging) lastDrag.current = Number.POSITIVE_INFINITY;
    else if (lastDrag.current === Number.POSITIVE_INFINITY) lastDrag.current = Date.now();
  }, [isDragging]);

  return (
    <li
      ref={setNodeRef}
      className={styles.slot}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      data-dragging={isDragging || undefined}
    >
      {editing ? (
        <div className={styles.card}>
          <InlineInput
            label={t("card.title")}
            initialValue={card.title}
            onSubmit={(title) => onRename(card.id, title)}
            onClose={() => setEditing(false)}
          />
        </div>
      ) : (
        <div
          ref={setActivatorNodeRef}
          className={styles.card}
          data-card-id={card.id}
          data-selected={selected || undefined}
          data-priority={card.priority}
          {...attributes}
          {...listeners}
          onContextMenu={contextMenu.onContextMenu}
          onClick={() => {
            if (Date.now() - lastDrag.current > CLICK_AFTER_DRAG_MS) onOpen(card.id);
          }}
          onKeyDown={(event) => {
            listeners?.onKeyDown?.(event);
            // While the card is picked up, the keys belong to drag and drop.
            if (isDragging || event.defaultPrevented) return;
            const direction = ARROWS[event.key];
            if (direction) {
              event.preventDefault();
              onNavigate(card.id, direction);
            } else if (event.key === "Enter") {
              event.preventDefault();
              onOpen(card.id);
            } else if (event.key === "F2") {
              event.preventDefault();
              setEditing(true);
            } else if (event.key === "Delete" || (isMac && event.key === "Backspace")) {
              event.preventDefault();
              onDelete(card);
            }
          }}
        >
          <CardContent card={card} tags={tags} />
        </div>
      )}
      {contextMenu.menu}
    </li>
  );
}

/** The copy of a card that follows the pointer while dragging. */
export function CardOverlay({ card, tags }: { card: CardSummary; tags: Tag[] }) {
  return (
    <div className={`${styles.card} ${styles.overlay}`} data-priority={card.priority}>
      <CardContent card={card} tags={tags} />
    </div>
  );
}

export default BoardCard;
