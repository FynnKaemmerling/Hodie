// Error codes sent by the backend (src-tauri/src/error.rs). Each has a
// translation under `errors.<code>` in the locale files.
export const ERROR_CODES = [
  "not_found",
  "validation.empty",
  "board.no_columns",
  "column.is_done",
  "card.archived",
  "card.not_archived",
  "tag.duplicate_name",
  "tag.invalid_color",
  "shortcut.invalid",
  "shortcut.unavailable",
  "db.too_new",
  "db.error",
  "internal",
] as const;

export type ErrorCode = (typeof ERROR_CODES)[number];

/** Translation key for anything a command rejected with. */
export function errorMessageKey(error: unknown): string {
  return ERROR_CODES.includes(error as ErrorCode) ? `errors.${error as string}` : "errors.unknown";
}
