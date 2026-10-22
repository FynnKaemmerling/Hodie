// Pure, immutable updates of a BoardView. The UI applies them optimistically
// before the backend confirms the change.
import type { BoardView, CardSummary, Column, ColumnView, Id } from "../api";

export interface CardLocation {
  columnIndex: number;
  cardIndex: number;
}

export function locateCard(view: BoardView, cardId: Id): CardLocation | null {
  for (let columnIndex = 0; columnIndex < view.columns.length; columnIndex++) {
    const cardIndex = view.columns[columnIndex].cards.findIndex((card) => card.id === cardId);
    if (cardIndex !== -1) return { columnIndex, cardIndex };
  }
  return null;
}

export function findCard(view: BoardView, cardId: Id): CardSummary | null {
  const location = locateCard(view, cardId);
  return location ? view.columns[location.columnIndex].cards[location.cardIndex] : null;
}

function mapColumn(
  view: BoardView,
  columnId: Id,
  change: (column: ColumnView) => ColumnView,
): BoardView {
  return {
    ...view,
    columns: view.columns.map((column) => (column.id === columnId ? change(column) : column)),
  };
}

function insertAt<T>(items: T[], item: T, index: number): T[] {
  const clamped = Math.max(0, Math.min(index, items.length));
  return [...items.slice(0, clamped), item, ...items.slice(clamped)];
}

export function removeCard(view: BoardView, cardId: Id): BoardView {
  return {
    ...view,
    columns: view.columns.map((column) =>
      column.cards.some((card) => card.id === cardId)
        ? { ...column, cards: column.cards.filter((card) => card.id !== cardId) }
        : column,
    ),
  };
}

/** Inserts at `index`, or at the end when it is omitted. */
export function insertCard(
  view: BoardView,
  card: CardSummary,
  columnId: Id,
  index = Number.POSITIVE_INFINITY,
): BoardView {
  return mapColumn(view, columnId, (column) => ({
    ...column,
    cards: insertAt(column.cards, { ...card, columnId }, index),
  }));
}

/** `index` is the final position in the target column, like the backend's. */
export function moveCard(view: BoardView, cardId: Id, columnId: Id, index: number): BoardView {
  const card = findCard(view, cardId);
  if (!card || !view.columns.some((column) => column.id === columnId)) return view;
  return insertCard(removeCard(view, cardId), card, columnId, index);
}

export function updateCard(
  view: BoardView,
  cardId: Id,
  change: Partial<Omit<CardSummary, "id" | "columnId">>,
): BoardView {
  const location = locateCard(view, cardId);
  if (!location) return view;
  return mapColumn(view, view.columns[location.columnIndex].id, (column) => ({
    ...column,
    cards: column.cards.map((card) => (card.id === cardId ? { ...card, ...change } : card)),
  }));
}

/** Takes a deleted tag off every card. */
export function removeTagFromCards(view: BoardView, tagId: Id): BoardView {
  return {
    ...view,
    columns: view.columns.map((column) => ({
      ...column,
      cards: column.cards.map((card) =>
        card.tagIds.includes(tagId)
          ? { ...card, tagIds: card.tagIds.filter((id) => id !== tagId) }
          : card,
      ),
    })),
  };
}

export function insertColumn(
  view: BoardView,
  column: ColumnView,
  index = Number.POSITIVE_INFINITY,
): BoardView {
  return { ...view, columns: insertAt(view.columns, column, index) };
}

export function removeColumn(view: BoardView, columnId: Id): BoardView {
  return { ...view, columns: view.columns.filter((column) => column.id !== columnId) };
}

export function moveColumn(view: BoardView, columnId: Id, index: number): BoardView {
  const column = view.columns.find((c) => c.id === columnId);
  if (!column) return view;
  return insertColumn(removeColumn(view, columnId), column, index);
}

export function updateColumn(
  view: BoardView,
  columnId: Id,
  change: Partial<Omit<Column, "id">>,
): BoardView {
  return mapColumn(view, columnId, (column) => ({ ...column, ...change }));
}

export function setDoneColumn(view: BoardView, columnId: Id): BoardView {
  return {
    ...view,
    columns: view.columns.map((column) => ({ ...column, isDone: column.id === columnId })),
  };
}

export type Direction = "up" | "down" | "left" | "right";

/**
 * The card keyboard focus moves to. Left and right skip empty columns and
 * keep the row where possible.
 */
export function neighbourCard(view: BoardView, cardId: Id, direction: Direction): Id | null {
  const location = locateCard(view, cardId);
  if (!location) return null;
  const { columnIndex, cardIndex } = location;
  const cards = view.columns[columnIndex].cards;

  if (direction === "up") return cards[cardIndex - 1]?.id ?? null;
  if (direction === "down") return cards[cardIndex + 1]?.id ?? null;

  const step = direction === "left" ? -1 : 1;
  for (let i = columnIndex + step; i >= 0 && i < view.columns.length; i += step) {
    const target = view.columns[i].cards;
    if (target.length > 0) return target[Math.min(cardIndex, target.length - 1)].id;
  }
  return null;
}
