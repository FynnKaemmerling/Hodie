import { describe, expect, it } from "vitest";
import type { BoardView, CardSummary, Id } from "../api";
import {
  insertCard,
  locateCard,
  moveCard,
  moveColumn,
  neighbourCard,
  removeCard,
  setDoneColumn,
  updateCard,
} from "./boardState";

function card(id: Id, columnId: Id): CardSummary {
  return {
    id,
    columnId,
    title: `Card ${id}`,
    priority: "none",
    hasNotes: false,
    subtasksDone: 0,
    subtasksTotal: 0,
    tagIds: [],
  };
}

// Columns 1, 2, 3 with cards [10, 11, 12], [], [30].
function board(): BoardView {
  return {
    board: { id: 1, name: "Board" },
    columns: [
      { id: 1, name: "Open", isDone: false, cards: [card(10, 1), card(11, 1), card(12, 1)] },
      { id: 2, name: "Doing", isDone: false, cards: [] },
      { id: 3, name: "Done", isDone: true, cards: [card(30, 3)] },
    ],
  };
}

function ids(view: BoardView): Id[][] {
  return view.columns.map((column) => column.cards.map((c) => c.id));
}

describe("boardState", () => {
  it("locates cards", () => {
    expect(locateCard(board(), 12)).toEqual({ columnIndex: 0, cardIndex: 2 });
    expect(locateCard(board(), 99)).toBeNull();
  });

  it("moves a card within its column", () => {
    expect(ids(moveCard(board(), 12, 1, 0))).toEqual([[12, 10, 11], [], [30]]);
    expect(ids(moveCard(board(), 10, 1, 99))).toEqual([[11, 12, 10], [], [30]]);
  });

  it("moves a card to another column and updates its columnId", () => {
    const view = moveCard(board(), 11, 2, 0);
    expect(ids(view)).toEqual([[10, 12], [11], [30]]);
    expect(view.columns[1].cards[0].columnId).toBe(2);
  });

  it("ignores unknown cards and columns", () => {
    const original = board();
    expect(moveCard(original, 99, 2, 0)).toBe(original);
    expect(moveCard(original, 10, 99, 0)).toBe(original);
  });

  it("does not mutate the input", () => {
    const original = board();
    moveCard(original, 10, 2, 0);
    removeCard(original, 30);
    expect(ids(original)).toEqual([[10, 11, 12], [], [30]]);
  });

  it("removes and re-inserts cards", () => {
    const removed = removeCard(board(), 11);
    expect(ids(removed)).toEqual([[10, 12], [], [30]]);
    expect(ids(insertCard(removed, card(11, 1), 1, 1))).toEqual([[10, 11, 12], [], [30]]);
    expect(ids(insertCard(removed, card(40, 2), 2))).toEqual([[10, 12], [40], [30]]);
  });

  it("updates card fields", () => {
    const view = updateCard(board(), 30, { title: "Renamed", priority: "high" });
    expect(view.columns[2].cards[0]).toMatchObject({ title: "Renamed", priority: "high" });
  });

  it("moves columns and sets the done column", () => {
    expect(moveColumn(board(), 3, 0).columns.map((c) => c.id)).toEqual([3, 1, 2]);
    expect(setDoneColumn(board(), 2).columns.map((c) => c.isDone)).toEqual([false, true, false]);
  });

  describe("neighbourCard", () => {
    it("moves up and down within a column", () => {
      expect(neighbourCard(board(), 11, "up")).toBe(10);
      expect(neighbourCard(board(), 11, "down")).toBe(12);
      expect(neighbourCard(board(), 10, "up")).toBeNull();
      expect(neighbourCard(board(), 12, "down")).toBeNull();
    });

    it("skips empty columns and clamps the row", () => {
      expect(neighbourCard(board(), 12, "right")).toBe(30);
      expect(neighbourCard(board(), 30, "left")).toBe(10);
      expect(neighbourCard(board(), 10, "left")).toBeNull();
      expect(neighbourCard(board(), 30, "right")).toBeNull();
    });
  });
});
