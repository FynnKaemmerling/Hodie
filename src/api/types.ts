// Mirrors src-tauri/src/model.rs.

/** ULIDs of boards, columns, cards, subtasks and tags in the vault. */
export type Id = string;
/** Colour presets keep the row id of the app's database. */
export type PresetId = number;

export type Priority = "none" | "low" | "medium" | "high";

export const TAG_COLORS = [
  "gray",
  "red",
  "orange",
  "yellow",
  "green",
  "teal",
  "blue",
  "purple",
] as const;
export type TagColor = (typeof TAG_COLORS)[number];

export interface Board {
  id: Id;
  name: string;
}

export interface Column {
  id: Id;
  name: string;
  isDone: boolean;
}

export interface CardSummary {
  id: Id;
  columnId: Id;
  title: string;
  priority: Priority;
  hasNotes: boolean;
  subtasksDone: number;
  subtasksTotal: number;
  tagIds: Id[];
}

export interface ColumnView extends Column {
  cards: CardSummary[];
}

export interface BoardView {
  board: Board;
  columns: ColumnView[];
}

export interface Subtask {
  id: Id;
  title: string;
  done: boolean;
}

/** Timestamps are Unix milliseconds. */
export interface CardDetail {
  id: Id;
  columnId: Id;
  title: string;
  notes: string;
  priority: Priority;
  createdAt: number;
  updatedAt: number;
  archivedAt: number | null;
  subtasks: Subtask[];
  tagIds: Id[];
}

export interface Tag {
  id: Id;
  name: string;
  color: TagColor;
}

/** The quick entry shortcut, in the format "Control+Shift+Space". */
export interface ShortcutInfo {
  shortcut: string;
  defaultShortcut: string;
  /** False when another program owns the shortcut. */
  registered: boolean;
}

export interface NewCard {
  columnId: Id;
  title: string;
  priority?: Priority;
  tagIds?: Id[];
}

export interface CardPatch {
  title?: string;
  notes?: string;
  priority?: Priority;
}

export interface SubtaskPatch {
  title?: string;
  done?: boolean;
}

export interface TagPatch {
  name?: string;
  color?: TagColor;
}

/** Accent and base colour (#rrggbb) of a colour preset for each mode. */
export interface PresetColors {
  lightAccent: string;
  lightBase: string;
  darkAccent: string;
  darkBase: string;
}

/** A colour preset the user created. */
export interface ColorPreset {
  id: PresetId;
  name: string;
  colors: PresetColors;
}

export interface ColorPresetPatch {
  name?: string;
  colors?: PresetColors;
}
