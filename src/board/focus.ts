import type { Id } from "../api";

// Elements may re-mount after a move, so focus them after the next render.

export function focusCard(cardId: Id) {
  requestAnimationFrame(() => {
    document.querySelector<HTMLElement>(`[data-card-id="${cardId}"]`)?.focus();
  });
}

export function focusColumn(columnId: Id) {
  requestAnimationFrame(() => {
    document.querySelector<HTMLElement>(`[data-column-id="${columnId}"]`)?.focus();
  });
}
