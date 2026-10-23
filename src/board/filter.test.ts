import { describe, expect, it } from "vitest";
import type { BoardView, CardSummary, Id, Priority } from "../api";
import { applyFilter, countCards, isFilterActive, NO_FILTER } from "./filter";

function card(id: Id, priority: Priority, tagIds: Id[]): CardSummary {
  return {
    id,
    columnId: "1",
    title: `Card ${id}`,
    priority,
    hasNotes: false,
    subtasksDone: 0,
    subtasksTotal: 0,
    hasConflict: false,
    tagIds,
  };
}

const view: BoardView = {
  board: { id: "1", name: "Board" },
  columns: [
    {
      id: "1",
      name: "Open",
      isDone: false,
      cards: [card("1", "high", ["7"]), card("2", "none", [])],
    },
    {
      id: "2",
      name: "Done",
      isDone: true,
      cards: [card("3", "low", ["7", "8"]), card("4", "high", ["8"])],
    },
  ],
};

const ids = (v: BoardView) => v.columns.map((c) => c.cards.map((x) => x.id));

describe("filter", () => {
  it("is inactive by default and with blank text", () => {
    expect(isFilterActive(NO_FILTER)).toBe(false);
    expect(isFilterActive({ ...NO_FILTER, query: "  " })).toBe(false);
    expect(isFilterActive({ ...NO_FILTER, priorities: ["low"] })).toBe(true);
  });

  it("keeps everything without a filter", () => {
    expect(ids(applyFilter(view, NO_FILTER, null))).toEqual([["1", "2"], ["3", "4"]]);
  });

  it("filters by search matches, tags (any) and priorities (any) combined", () => {
    expect(ids(applyFilter(view, NO_FILTER, new Set(["2", "4"])))).toEqual([["2"], ["4"]]);
    expect(ids(applyFilter(view, { ...NO_FILTER, tagIds: ["7"] }, null))).toEqual([["1"], ["3"]]);
    expect(ids(applyFilter(view, { ...NO_FILTER, priorities: ["high", "low"] }, null))).toEqual([
      ["1"],
      ["3", "4"],
    ]);
    expect(
      ids(
        applyFilter(
          view,
          { query: "x", tagIds: ["8"], priorities: ["high"] },
          new Set(["1", "4"]),
        ),
      ),
    ).toEqual([[], ["4"]]);
  });

  it("keeps all columns, even empty ones, and counts cards", () => {
    const filtered = applyFilter(view, { ...NO_FILTER, tagIds: ["99"] }, null);
    expect(filtered.columns).toHaveLength(2);
    expect(countCards(filtered)).toBe(0);
    expect(countCards(view)).toBe(4);
  });
});
