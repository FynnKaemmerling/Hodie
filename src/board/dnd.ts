import type { UniqueIdentifier } from "@dnd-kit/core";
import type { Id } from "../api";

// Card and column ids can collide, so drag-and-drop ids carry their kind.
export type DragKind = "card" | "column";

export interface DragData {
  kind: DragKind;
  id: Id;
  /** For cards: the column they currently sit in. */
  columnId: Id;
}

export function dndId(kind: DragKind, id: Id): string {
  return `${kind}-${id}`;
}

export function parseDndId(value: UniqueIdentifier): { kind: DragKind; id: Id } | null {
  const match = /^(card|column)-(\d+)$/.exec(String(value));
  return match ? { kind: match[1] as DragKind, id: Number(match[2]) } : null;
}
