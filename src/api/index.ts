import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type {
  Board,
  BoardView,
  CardDetail,
  CardPatch,
  CardSummary,
  CardVersion,
  ColorPreset,
  ColorPresetPatch,
  Column,
  Id,
  ImagePatch,
  MoodboardImage,
  MoodboardView,
  NewCard,
  PresetColors,
  PresetId,
  ShortcutInfo,
  Subtask,
  SubtaskPatch,
  Tag,
  TagColor,
  TagPatch,
  VaultInfo,
  VaultProblem,
} from "./types";

export * from "./types";

// Thin wrappers around the commands in src-tauri/src/commands.rs.
// Rejections carry an error code string, see ./errors.ts.
export const api = {
  getVault: () => invoke<VaultInfo>("get_vault"),
  openVault: (path: string) => invoke<VaultInfo>("open_vault", { path }),
  reopenVault: () => invoke<VaultInfo>("reopen_vault"),
  refreshVault: () => invoke<boolean>("refresh_vault"),
  listVaultProblems: () => invoke<VaultProblem[]>("list_vault_problems"),
  getCardConflicts: (cardId: Id) => invoke<CardVersion[]>("get_card_conflicts", { cardId }),
  /** Keeps the version at `keep` (a conflict copy), or the card as it is if null. */
  resolveCardConflict: (cardId: Id, keep: string | null) =>
    invoke<void>("resolve_card_conflict", { cardId, keep }),

  listBoards: () => invoke<Board[]>("list_boards"),
  getBoard: (boardId: Id) => invoke<BoardView>("get_board", { boardId }),
  createBoard: (name: string, columnNames: string[]) =>
    invoke<Board>("create_board", { name, columnNames }),
  renameBoard: (boardId: Id, name: string) => invoke<void>("rename_board", { boardId, name }),
  reorderBoard: (boardId: Id, index: number) =>
    invoke<void>("reorder_board", { boardId, index }),
  deleteBoard: (boardId: Id) => invoke<void>("delete_board", { boardId }),
  restoreBoard: (boardId: Id) => invoke<void>("restore_board", { boardId }),

  createMoodboard: (name: string) => invoke<Board>("create_moodboard", { name }),
  getMoodboard: (boardId: Id) => invoke<MoodboardView>("get_moodboard", { boardId }),
  /** Copies an image file onto the moodboard, its top left corner at x, y. */
  addMoodboardImage: (boardId: Id, path: string, x: number, y: number) =>
    invoke<MoodboardImage>("add_moodboard_image", { boardId, path, x, y }),
  /** Adds an image from memory (e.g. pasted); the bytes travel without JSON. */
  addMoodboardImageData: (boardId: Id, bytes: Uint8Array, x: number, y: number) =>
    invoke<MoodboardImage>("add_moodboard_image_data", bytes, {
      headers: { "x-board-id": boardId, "x-x": String(x), "x-y": String(y) },
    }),
  updateMoodboardImage: (imageId: Id, patch: ImagePatch) =>
    invoke<void>("update_moodboard_image", { imageId, patch }),
  /** Moves an image to another layer: 0 is the back. */
  reorderMoodboardImage: (imageId: Id, index: number) =>
    invoke<void>("reorder_moodboard_image", { imageId, index }),
  deleteMoodboardImage: (imageId: Id) => invoke<void>("delete_moodboard_image", { imageId }),
  restoreMoodboardImage: (imageId: Id) => invoke<void>("restore_moodboard_image", { imageId }),

  createColumn: (boardId: Id, name: string) =>
    invoke<Column>("create_column", { boardId, name }),
  renameColumn: (columnId: Id, name: string) =>
    invoke<void>("rename_column", { columnId, name }),
  reorderColumn: (columnId: Id, index: number) =>
    invoke<void>("reorder_column", { columnId, index }),
  setDoneColumn: (columnId: Id) => invoke<void>("set_done_column", { columnId }),
  deleteColumn: (columnId: Id) => invoke<void>("delete_column", { columnId }),
  restoreColumn: (columnId: Id) => invoke<void>("restore_column", { columnId }),

  searchCards: (boardId: Id, query: string) => invoke<Id[]>("search_cards", { boardId, query }),
  getCard: (cardId: Id) => invoke<CardDetail>("get_card", { cardId }),
  createCard: (card: NewCard) => invoke<CardSummary>("create_card", { card }),
  updateCard: (cardId: Id, patch: CardPatch) => invoke<void>("update_card", { cardId, patch }),
  moveCard: (cardId: Id, columnId: Id, index: number) =>
    invoke<void>("move_card", { cardId, columnId, index }),
  deleteCard: (cardId: Id) => invoke<void>("delete_card", { cardId }),
  restoreCard: (cardId: Id) => invoke<void>("restore_card", { cardId }),
  archiveCard: (cardId: Id) => invoke<void>("archive_card", { cardId }),
  unarchiveCard: (cardId: Id) => invoke<void>("unarchive_card", { cardId }),
  archiveDoneCards: (boardId: Id) => invoke<Id[]>("archive_done_cards", { boardId }),
  listArchivedCards: (boardId: Id) => invoke<CardSummary[]>("list_archived_cards", { boardId }),
  setCardTags: (cardId: Id, tagIds: Id[]) => invoke<Id[]>("set_card_tags", { cardId, tagIds }),

  createSubtask: (cardId: Id, title: string) =>
    invoke<Subtask>("create_subtask", { cardId, title }),
  updateSubtask: (subtaskId: Id, patch: SubtaskPatch) =>
    invoke<void>("update_subtask", { subtaskId, patch }),
  reorderSubtask: (subtaskId: Id, index: number) =>
    invoke<void>("reorder_subtask", { subtaskId, index }),
  deleteSubtask: (subtaskId: Id) => invoke<void>("delete_subtask", { subtaskId }),

  listTags: () => invoke<Tag[]>("list_tags"),
  createTag: (name: string, color: TagColor) => invoke<Tag>("create_tag", { name, color }),
  updateTag: (tagId: Id, patch: TagPatch) => invoke<void>("update_tag", { tagId, patch }),
  countTagCards: (tagId: Id) => invoke<number>("count_tag_cards", { tagId }),
  deleteTag: (tagId: Id) => invoke<void>("delete_tag", { tagId }),

  listColorPresets: () => invoke<ColorPreset[]>("list_color_presets"),
  createColorPreset: (name: string, colors: PresetColors) =>
    invoke<ColorPreset>("create_color_preset", { name, colors }),
  updateColorPreset: (presetId: PresetId, patch: ColorPresetPatch) =>
    invoke<void>("update_color_preset", { presetId, patch }),
  deleteColorPreset: (presetId: PresetId) => invoke<void>("delete_color_preset", { presetId }),
  getColorPresetChoice: () => invoke<string>("get_color_preset_choice"),
  setColorPresetChoice: (choice: string) => invoke<void>("set_color_preset_choice", { choice }),

  getPomodoroSound: () => invoke<string | null>("get_pomodoro_sound"),
  setPomodoroSound: (sound: string) => invoke<void>("set_pomodoro_sound", { sound }),

  getStorageUsage: () => invoke<number>("get_storage_usage"),

  getQuickAddShortcut: () => invoke<ShortcutInfo>("get_quick_add_shortcut"),
  setQuickAddShortcut: (shortcut: string) =>
    invoke<ShortcutInfo>("set_quick_add_shortcut", { shortcut }),
};

/** URL under which the backend serves an image of a moodboard. */
export function imageUrl(boardId: Id, file: string): string {
  return convertFileSrc(`${boardId}/${file}`, "hodie-image");
}
