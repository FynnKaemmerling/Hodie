import { invoke } from "@tauri-apps/api/core";
import type {
  Board,
  BoardView,
  CardDetail,
  CardPatch,
  CardSummary,
  Column,
  Id,
  NewCard,
  Subtask,
  SubtaskPatch,
  Tag,
  TagColor,
  TagPatch,
} from "./types";

export * from "./types";

// Thin wrappers around the commands in src-tauri/src/commands.rs.
// Rejections carry an error code string, see ./errors.ts.
export const api = {
  listBoards: () => invoke<Board[]>("list_boards"),
  getBoard: (boardId: Id) => invoke<BoardView>("get_board", { boardId }),
  createBoard: (name: string, columnNames: string[]) =>
    invoke<Board>("create_board", { name, columnNames }),
  renameBoard: (boardId: Id, name: string) => invoke<void>("rename_board", { boardId, name }),
  reorderBoard: (boardId: Id, index: number) =>
    invoke<void>("reorder_board", { boardId, index }),
  deleteBoard: (boardId: Id) => invoke<void>("delete_board", { boardId }),
  restoreBoard: (boardId: Id) => invoke<void>("restore_board", { boardId }),

  createColumn: (boardId: Id, name: string) =>
    invoke<Column>("create_column", { boardId, name }),
  renameColumn: (columnId: Id, name: string) =>
    invoke<void>("rename_column", { columnId, name }),
  reorderColumn: (columnId: Id, index: number) =>
    invoke<void>("reorder_column", { columnId, index }),
  setDoneColumn: (columnId: Id) => invoke<void>("set_done_column", { columnId }),
  deleteColumn: (columnId: Id) => invoke<void>("delete_column", { columnId }),
  restoreColumn: (columnId: Id) => invoke<void>("restore_column", { columnId }),

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
  deleteTag: (tagId: Id) => invoke<void>("delete_tag", { tagId }),
};
