// Pure, immutable updates of a CardDetail, mirrored into the board's
// CardSummary after each change.
import { TAG_COLORS, type CardDetail, type CardSummary, type Id, type Subtask, type Tag, type TagColor } from "../api";

export type SummaryFields = Omit<CardSummary, "id" | "columnId">;

/** The fields the board shows for a card, as the backend computes them. */
export function summaryFields(card: CardDetail): SummaryFields {
  return {
    title: card.title,
    priority: card.priority,
    hasNotes: card.notes !== "",
    subtasksDone: card.subtasks.filter((subtask) => subtask.done).length,
    subtasksTotal: card.subtasks.length,
    tagIds: card.tagIds,
    hasConflict: card.hasConflict,
  };
}

export function updateSubtask(
  card: CardDetail,
  subtaskId: Id,
  change: Partial<Omit<Subtask, "id">>,
): CardDetail {
  return {
    ...card,
    subtasks: card.subtasks.map((s) => (s.id === subtaskId ? { ...s, ...change } : s)),
  };
}

export function removeSubtask(card: CardDetail, subtaskId: Id): CardDetail {
  return { ...card, subtasks: card.subtasks.filter((s) => s.id !== subtaskId) };
}

/** Inserts at `index`, or at the end when it is omitted. */
export function insertSubtask(
  card: CardDetail,
  subtask: Subtask,
  index = Number.POSITIVE_INFINITY,
): CardDetail {
  const clamped = Math.max(0, Math.min(index, card.subtasks.length));
  return {
    ...card,
    subtasks: [...card.subtasks.slice(0, clamped), subtask, ...card.subtasks.slice(clamped)],
  };
}

/** `index` is the final position, like the backend's. */
export function moveSubtask(card: CardDetail, subtaskId: Id, index: number): CardDetail {
  const subtask = card.subtasks.find((s) => s.id === subtaskId);
  return subtask ? insertSubtask(removeSubtask(card, subtaskId), subtask, index) : card;
}

/** Suggests the colour used least so far, so new tags look different. */
export function nextTagColor(tags: Tag[]): TagColor {
  const uses = new Map<TagColor, number>(TAG_COLORS.map((color) => [color, 0]));
  for (const tag of tags) uses.set(tag.color, (uses.get(tag.color) ?? 0) + 1);
  return TAG_COLORS.reduce((best, color) => (uses.get(color)! < uses.get(best)! ? color : best));
}

/** Tags whose name contains `query`, ignoring case; all tags for an empty query. */
export function filterTags(tags: Tag[], query: string): Tag[] {
  const needle = query.trim().toLocaleLowerCase();
  return needle === "" ? tags : tags.filter((tag) => tag.name.toLocaleLowerCase().includes(needle));
}

export function findTagByName(tags: Tag[], name: string): Tag | undefined {
  const needle = name.trim().toLocaleLowerCase();
  return tags.find((tag) => tag.name.toLocaleLowerCase() === needle);
}
