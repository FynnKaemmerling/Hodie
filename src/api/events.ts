// Events sent between the app's windows.

/** A window changed cards on a board; others showing it should reload. */
export const CARDS_CHANGED = "cards-changed";

export interface CardsChangedPayload {
  /** The card just added, which the board briefly highlights. */
  cardId?: number;
}
