import { describe, expect, it } from "vitest";
import type { CardDetail, Tag } from "../api";
import {
  filterTags,
  findTagByName,
  insertSubtask,
  moveSubtask,
  nextTagColor,
  removeSubtask,
  summaryFields,
  updateSubtask,
} from "./cardState";

function card(): CardDetail {
  return {
    id: "1",
    columnId: "1",
    title: "Card",
    notes: "",
    priority: "none",
    createdAt: 0,
    updatedAt: 0,
    archivedAt: null,
    subtasks: [
      { id: "1", title: "A", done: true },
      { id: "2", title: "B", done: false },
      { id: "3", title: "C", done: false },
    ],
    tagIds: ["5"],
    hasConflict: false,
  };
}

const subtaskIds = (c: CardDetail) => c.subtasks.map((s) => s.id);

describe("cardState", () => {
  it("derives the summary the board shows", () => {
    expect(summaryFields(card())).toEqual({
      title: "Card",
      priority: "none",
      hasNotes: false,
      subtasksDone: 1,
      subtasksTotal: 3,
      tagIds: ["5"],
      hasConflict: false,
    });
    expect(summaryFields({ ...card(), notes: " " }).hasNotes).toBe(true);
  });

  it("updates, removes, inserts and moves subtasks", () => {
    expect(updateSubtask(card(), "2", { done: true }).subtasks[1].done).toBe(true);
    expect(subtaskIds(removeSubtask(card(), "2"))).toEqual(["1", "3"]);
    expect(subtaskIds(insertSubtask(card(), { id: "4", title: "D", done: false }, 1))).toEqual([
      "1", "4", "2", "3",
    ]);
    expect(subtaskIds(insertSubtask(card(), { id: "4", title: "D", done: false }))).toEqual([
      "1", "2", "3", "4",
    ]);
    expect(subtaskIds(moveSubtask(card(), "3", 0))).toEqual(["3", "1", "2"]);
    expect(subtaskIds(moveSubtask(card(), "1", 99))).toEqual(["2", "3", "1"]);
  });

  describe("tags", () => {
    const tags: Tag[] = [
      { id: "1", name: "Work", color: "gray" },
      { id: "2", name: "Home", color: "red" },
      { id: "3", name: "Homework", color: "gray" },
    ];

    it("suggests the least used colour", () => {
      expect(nextTagColor([])).toBe("gray");
      expect(nextTagColor(tags)).toBe("orange");
    });

    it("filters and finds by name ignoring case", () => {
      expect(filterTags(tags, " home").map((t) => t.id)).toEqual(["2", "3"]);
      expect(filterTags(tags, "")).toHaveLength(3);
      expect(findTagByName(tags, "WORK ")?.id).toBe("1");
      expect(findTagByName(tags, "Wor")).toBeUndefined();
    });
  });
});
