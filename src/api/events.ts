// Events sent between the app's windows.

import type { Id } from "./types";

/** A window changed cards on a board; others showing it should reload. */
export const CARDS_CHANGED = "cards-changed";

/** Another folder was opened as the vault; every window starts over. */
export const VAULT_CHANGED = "vault-changed";

export interface CardsChangedPayload {
  /** The card just added, which the board briefly highlights. */
  cardId?: Id;
}
