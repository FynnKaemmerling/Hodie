// Narrowing the board down by search text, tags and priority.
import type { BoardView, CardSummary, Id, Priority } from "../api";

export interface CardFilter {
  query: string;
  /** Cards with any of these tags; empty means no tag filter. */
  tagIds: Id[];
  /** Cards with any of these priorities; empty means no priority filter. */
  priorities: Priority[];
}

export const NO_FILTER: CardFilter = { query: "", tagIds: [], priorities: [] };

export function isFilterActive(filter: CardFilter): boolean {
  return filter.query.trim() !== "" || filter.tagIds.length > 0 || filter.priorities.length > 0;
}

/**
 * Keeps only matching cards. `searchMatches` holds the ids the backend found
 * for the query (title and notes); `null` means no search is running.
 */
export function applyFilter(
  view: BoardView,
  filter: CardFilter,
  searchMatches: ReadonlySet<Id> | null,
): BoardView {
  const matches = (card: CardSummary) =>
    (searchMatches === null || searchMatches.has(card.id)) &&
    (filter.tagIds.length === 0 || card.tagIds.some((id) => filter.tagIds.includes(id))) &&
    (filter.priorities.length === 0 || filter.priorities.includes(card.priority));

  return {
    ...view,
    columns: view.columns.map((column) => ({ ...column, cards: column.cards.filter(matches) })),
  };
}

export function countCards(view: BoardView): number {
  return view.columns.reduce((sum, column) => sum + column.cards.length, 0);
}
