import { useCallback, useEffect, useRef, useState } from "react";
import {
  closestCenter,
  closestCorners,
  DndContext,
  DragOverlay,
  getFirstCollision,
  KeyboardSensor,
  PointerSensor,
  pointerWithin,
  rectIntersection,
  useSensor,
  useSensors,
  type Announcements,
  type CollisionDetection,
  type DragEndEvent,
  type DragOverEvent,
  type DragStartEvent,
  type UniqueIdentifier,
} from "@dnd-kit/core";
import {
  horizontalListSortingStrategy,
  SortableContext,
  sortableKeyboardCoordinates,
} from "@dnd-kit/sortable";
import { useTranslation } from "react-i18next";
import type { BoardView, Id } from "../api";
import { findCard, locateCard, moveCard, neighbourCard } from "../board/boardState";
import type { Direction } from "../board/boardState";
import { dndId, parseDndId, type DragData } from "../board/dnd";
import { focusCard, focusColumn } from "../board/focus";
import type { BoardActions } from "../board/useBoard";
import { CardOverlay } from "./BoardCard";
import BoardColumn, { ColumnOverlay } from "./BoardColumn";
import { isModKey } from "../platform";
import InlineInput from "./InlineInput";
import styles from "./BoardCanvas.module.css";

function dragData(value: { data: { current?: unknown } } | null | undefined): DragData | null {
  return (value?.data.current as DragData | undefined) ?? null;
}

/**
 * Columns only collide with columns. Cards prefer the card under the
 * pointer; over a column's empty area they snap to its closest card.
 */
const collisionDetection: CollisionDetection = (args) => {
  if (dragData(args.active)?.kind === "column") {
    return closestCenter({
      ...args,
      droppableContainers: args.droppableContainers.filter(
        (container) => dragData(container)?.kind === "column",
      ),
    });
  }

  // pointerWithin finds nothing for keyboard drags, which have no pointer.
  const underPointer = pointerWithin(args);
  const collisions = underPointer.length > 0 ? underPointer : rectIntersection(args);
  const overId = getFirstCollision(collisions, "id");
  if (overId === null) return closestCorners(args);

  const over = args.droppableContainers.find((container) => container.id === overId);
  const overData = dragData(over);
  if (overData?.kind === "column") {
    const cardsInColumn = args.droppableContainers.filter((container) => {
      const data = dragData(container);
      return data?.kind === "card" && data.columnId === overData.id && container.id !== args.active.id;
    });
    if (cardsInColumn.length > 0) {
      return closestCenter({ ...args, droppableContainers: cardsInColumn });
    }
  }
  return [{ id: overId }];
};

/** The column Mod+N adds to: the focused one, the open card's, or the first. */
function targetColumn(view: BoardView, focused: Element | null, selectedCardId: Id | null): Id {
  const section = focused?.closest<HTMLElement>("[data-column-section]");
  const fromFocus = Number(section?.dataset.columnSection);
  if (view.columns.some((column) => column.id === fromFocus)) return fromFocus;
  const selected = selectedCardId === null ? null : locateCard(view, selectedCardId);
  return selected ? view.columns[selected.columnIndex].id : view.columns[0].id;
}

interface Props {
  board: BoardActions;
  selectedCardId: Id | null;
  onOpenCard: (cardId: Id) => void;
  /** Hides cards that do not match the current search and filter. */
  filterView: (view: BoardView) => BoardView;
  /** While filtering, positions among the visible cards would be wrong. */
  dragDisabled: boolean;
}

function BoardCanvas({ board, selectedCardId, onOpenCard, filterView, dragDisabled }: Props) {
  const { t } = useTranslation();
  const { view, tags } = board;
  // While dragging, cards moved between columns live in this draft until drop.
  const [draft, setDraft] = useState<BoardView | null>(null);
  const draftRef = useRef<BoardView | null>(null);
  const [activeId, setActiveId] = useState<UniqueIdentifier | null>(null);
  const [addingColumn, setAddingColumn] = useState(false);
  const [addingCardIn, setAddingCardIn] = useState<Id | null>(null);
  const latest = useRef({ view, selectedCardId });
  latest.current = { view, selectedCardId };

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (!isModKey(event) || event.shiftKey || event.altKey) return;
      if (event.key.toLowerCase() !== "n") return;
      const { view, selectedCardId } = latest.current;
      if (!view || view.columns.length === 0) return;
      event.preventDefault();
      setAddingCardIn(targetColumn(view, document.activeElement, selectedCardId));
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const updateDraft = useCallback((next: BoardView | null) => {
    draftRef.current = next;
    setDraft(next);
  }, []);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
      // Enter stays free to open a card.
      keyboardCodes: { start: ["Space"], cancel: ["Escape"], end: ["Space"] },
    }),
  );

  if (!view) return null;
  const shown = filterView(draft ?? view);

  function nameOf(id: UniqueIdentifier): string {
    const parsed = parseDndId(id);
    const current = draftRef.current ?? view;
    if (!parsed || !current) return "";
    if (parsed.kind === "card") return findCard(current, parsed.id)?.title ?? "";
    return current.columns.find((column) => column.id === parsed.id)?.name ?? "";
  }

  function describeTarget(activeId: UniqueIdentifier): string {
    const parsed = parseDndId(activeId);
    const current = draftRef.current;
    if (!parsed || !current) return "";
    if (parsed.kind === "column") {
      const index = current.columns.findIndex((column) => column.id === parsed.id);
      return t("dnd.columnPosition", { position: index + 1, total: current.columns.length });
    }
    const location = locateCard(current, parsed.id);
    if (!location) return "";
    return t("dnd.cardPosition", {
      column: current.columns[location.columnIndex].name,
      position: location.cardIndex + 1,
    });
  }

  const announcements: Announcements = {
    onDragStart: ({ active }) => t("dnd.pickedUp", { name: nameOf(active.id) }),
    onDragOver: ({ active, over }) =>
      over
        ? t("dnd.over", { name: nameOf(active.id), target: describeTarget(active.id) })
        : t("dnd.notOver", { name: nameOf(active.id) }),
    onDragEnd: ({ active }) => t("dnd.dropped", { name: nameOf(active.id) }),
    onDragCancel: ({ active }) => t("dnd.cancelled", { name: nameOf(active.id) }),
  };

  function handleDragStart({ active }: DragStartEvent) {
    setActiveId(active.id);
    updateDraft(view);
  }

  // Moves a dragged card into another column as soon as it hovers there,
  // so that column's cards make room for it.
  function handleDragOver({ active, over }: DragOverEvent) {
    const current = draftRef.current;
    const activeData = dragData(active);
    const overData = dragData(over);
    if (!current || !over || activeData?.kind !== "card" || !overData) return;
    if (over.id === active.id) return;

    const from = locateCard(current, activeData.id);
    if (!from) return;
    const fromColumnId = current.columns[from.columnIndex].id;

    let toColumnId: Id;
    let toIndex: number;
    if (overData.kind === "column") {
      toColumnId = overData.id;
      const column = current.columns.find((c) => c.id === toColumnId);
      toIndex = column ? column.cards.filter((card) => card.id !== activeData.id).length : 0;
    } else {
      const to = locateCard(current, overData.id);
      if (!to) return;
      toColumnId = current.columns[to.columnIndex].id;
      const translated = active.rect.current.translated;
      const below = translated !== null && translated.top > over.rect.top + over.rect.height / 2;
      toIndex = to.cardIndex + (below ? 1 : 0);
    }

    if (toColumnId !== fromColumnId) {
      updateDraft(moveCard(current, activeData.id, toColumnId, toIndex));
    }
  }

  function handleDragEnd({ active, over, activatorEvent }: DragEndEvent) {
    const current = draftRef.current;
    const activeData = dragData(active);
    const overData = dragData(over);
    const byKeyboard = activatorEvent instanceof KeyboardEvent;
    setActiveId(null);
    updateDraft(null);
    if (!current || !view || !activeData) return;

    if (activeData.kind === "column") {
      if (overData?.kind === "column" && overData.id !== activeData.id) {
        const index = view.columns.findIndex((column) => column.id === overData.id);
        board.moveColumn(activeData.id, index);
      }
      if (byKeyboard) focusColumn(activeData.id);
      return;
    }

    const location = locateCard(current, activeData.id);
    if (!location) return;
    const columnId = current.columns[location.columnIndex].id;
    let index = location.cardIndex;
    if (overData?.kind === "card" && overData.id !== activeData.id) {
      const overLocation = locateCard(current, overData.id);
      if (overLocation && overLocation.columnIndex === location.columnIndex) {
        index = overLocation.cardIndex;
      }
    }

    const original = locateCard(view, activeData.id);
    const changed =
      !original ||
      view.columns[original.columnIndex].id !== columnId ||
      original.cardIndex !== index;
    if (changed) board.moveCard(activeData.id, columnId, index);
    if (byKeyboard) focusCard(activeData.id);
  }

  function handleDragCancel() {
    setActiveId(null);
    updateDraft(null);
  }

  function navigate(cardId: Id, direction: Direction) {
    if (!view) return;
    const target = neighbourCard(shown, cardId, direction);
    if (target !== null) focusCard(target);
  }

  const active = activeId === null ? null : parseDndId(activeId);
  const activeCard = active?.kind === "card" ? findCard(shown, active.id) : null;
  const activeColumn =
    active?.kind === "column" ? shown.columns.find((column) => column.id === active.id) : null;

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={collisionDetection}
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDragEnd={handleDragEnd}
      onDragCancel={handleDragCancel}
      accessibility={{
        announcements,
        screenReaderInstructions: { draggable: t("dnd.instructions") },
      }}
    >
      <div className={styles.canvas}>
        <SortableContext
          items={shown.columns.map((column) => dndId("column", column.id))}
          strategy={horizontalListSortingStrategy}
        >
          {shown.columns.map((column) => (
            <BoardColumn
              key={column.id}
              column={column}
              tags={tags}
              onRename={board.renameColumn}
              onSetDone={board.setDoneColumn}
              onDelete={board.deleteColumn}
              onCreateCard={board.createCard}
              onRenameCard={board.renameCard}
              onDeleteCard={board.deleteCard}
              onNavigate={navigate}
              onOpenCard={onOpenCard}
              onArchiveDone={board.archiveDoneCards}
              selectedCardId={selectedCardId}
              dragDisabled={dragDisabled}
              addingCard={addingCardIn === column.id}
              onAddingCardChange={(open) => setAddingCardIn(open ? column.id : null)}
            />
          ))}
        </SortableContext>

        <div className={styles.addColumn}>
          {addingColumn ? (
            <InlineInput
              label={t("column.name")}
              placeholder={t("column.namePlaceholder")}
              onSubmit={board.createColumn}
              onClose={() => setAddingColumn(false)}
            />
          ) : (
            <button type="button" className={styles.add} onClick={() => setAddingColumn(true)}>
              <span aria-hidden="true">+</span> {t("column.add")}
            </button>
          )}
        </div>
      </div>

      <DragOverlay>
        {activeCard && <CardOverlay card={activeCard} tags={tags} />}
        {activeColumn && <ColumnOverlay column={activeColumn} />}
      </DragOverlay>
    </DndContext>
  );
}

export default BoardCanvas;
