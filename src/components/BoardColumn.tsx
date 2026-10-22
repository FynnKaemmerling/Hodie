import { CircleCheck, Plus } from "lucide-react";
import { useState } from "react";
import { SortableContext, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useTranslation } from "react-i18next";
import type { CardSummary, ColumnView, Id, Tag } from "../api";
import type { Direction } from "../board/boardState";
import { dndId, type DragData } from "../board/dnd";
import ActionMenu, { useContextMenu, type ActionMenuItem } from "./ActionMenu";
import BoardCard from "./BoardCard";
import InlineInput from "./InlineInput";
import styles from "./BoardColumn.module.css";

interface Props {
  column: ColumnView;
  tags: Tag[];
  onRename: (columnId: Id, name: string) => void;
  onSetDone: (columnId: Id) => void;
  onDelete: (column: ColumnView) => void;
  onCreateCard: (columnId: Id, title: string) => void;
  onRenameCard: (cardId: Id, title: string) => void;
  onDeleteCard: (card: CardSummary) => void;
  onArchiveCard: (card: CardSummary) => void;
  onNavigate: (cardId: Id, direction: Direction) => void;
  onOpenCard: (cardId: Id) => void;
  onArchiveDone: () => void;
  selectedCardId: Id | null;
  dragDisabled: boolean;
  /** The "add card" field is controlled so Mod+N can open it. */
  addingCard: boolean;
  onAddingCardChange: (open: boolean) => void;
}

function BoardColumn({
  column,
  tags,
  onRename,
  onSetDone,
  onDelete,
  onCreateCard,
  onRenameCard,
  onDeleteCard,
  onArchiveCard,
  onNavigate,
  onOpenCard,
  onArchiveDone,
  selectedCardId,
  dragDisabled,
  addingCard,
  onAddingCardChange,
}: Props) {
  const { t } = useTranslation();
  const [renaming, setRenaming] = useState(false);
  const data: DragData = { kind: "column", id: column.id, columnId: column.id };
  const {
    attributes,
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: dndId("column", column.id), data, disabled: renaming || dragDisabled });

  const menuItems: ActionMenuItem[] = [
    { id: "rename", label: t("actions.rename"), onAction: () => setRenaming(true) },
    {
      id: "done",
      label: t("column.setDone"),
      disabled: column.isDone,
      onAction: () => onSetDone(column.id),
    },
    ...(column.isDone
      ? [
          {
            id: "archiveDone",
            label: t("column.archiveDone"),
            disabled: column.cards.length === 0,
            onAction: onArchiveDone,
          },
        ]
      : []),
    {
      id: "delete",
      label: column.isDone ? t("column.deleteDoneHint") : t("actions.delete"),
      danger: true,
      disabled: column.isDone,
      onAction: () => onDelete(column),
    },
  ];
  const contextMenu = useContextMenu(menuItems);

  return (
    <section
      onContextMenu={contextMenu.onContextMenu}
      ref={setNodeRef}
      className={styles.column}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      data-dragging={isDragging || undefined}
      data-column-section={column.id}
      aria-label={column.name}
    >
      <header className={styles.header}>
        {renaming ? (
          <InlineInput
            label={t("column.name")}
            initialValue={column.name}
            onSubmit={(name) => onRename(column.id, name)}
            onClose={() => setRenaming(false)}
          />
        ) : (
          <>
            <div
              ref={setActivatorNodeRef}
              className={styles.handle}
              data-column-id={column.id}
              {...attributes}
              {...listeners}
              onDoubleClick={() => setRenaming(true)}
              onKeyDown={(event) => {
                listeners?.onKeyDown?.(event);
                if (isDragging || event.defaultPrevented) return;
                if (event.key === "F2") {
                  event.preventDefault();
                  setRenaming(true);
                }
              }}
            >
              <h2 className={styles.name}>{column.name}</h2>
              <span className={styles.count}>
                <span aria-hidden="true">{column.cards.length}</span>
                <span className="visually-hidden">
                  {t("column.cardCount", { count: column.cards.length })}
                </span>
              </span>
              {column.isDone && (
                <span className={styles.done}>
                  <CircleCheck aria-hidden="true" />
                  <span className="visually-hidden">{t("column.isDone")}</span>
                </span>
              )}
            </div>
            <ActionMenu
              label={t("actions.more", { name: column.name })}
              items={menuItems}
            />
          </>
        )}
      </header>

      <SortableContext
        items={column.cards.map((card) => dndId("card", card.id))}
        strategy={verticalListSortingStrategy}
      >
        <ul className={styles.cards}>
          {column.cards.map((card) => (
            <BoardCard
              key={card.id}
              card={card}
              tags={tags}
              onRename={onRenameCard}
              onDelete={onDeleteCard}
              onArchive={onArchiveCard}
              onNavigate={onNavigate}
              onOpen={onOpenCard}
              selected={card.id === selectedCardId}
              dragDisabled={dragDisabled}
            />
          ))}
        </ul>
      </SortableContext>

      <div className={styles.footer}>
        {addingCard ? (
          <InlineInput
            label={t("card.title")}
            placeholder={t("card.titlePlaceholder")}
            keepOpen
            onSubmit={(title) => onCreateCard(column.id, title)}
            onClose={() => onAddingCardChange(false)}
          />
        ) : (
          <button type="button" className={styles.add} onClick={() => onAddingCardChange(true)}>
            <Plus aria-hidden="true" /> {t("card.add")}
          </button>
        )}
      </div>
      {contextMenu.menu}
    </section>
  );
}

/** The copy of a column that follows the pointer while dragging. */
export function ColumnOverlay({ column }: { column: ColumnView }) {
  return (
    <div className={`${styles.column} ${styles.overlay}`}>
      <header className={styles.header}>
        <div className={styles.handle}>
          <h2 className={styles.name}>{column.name}</h2>
          <span className={styles.count}>{column.cards.length}</span>
        </div>
      </header>
    </div>
  );
}

export default BoardColumn;
