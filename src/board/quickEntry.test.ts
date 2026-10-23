import { describe, expect, it } from "vitest";
import type { Tag } from "../api";
import { parseQuickEntry, resolveTags } from "./quickEntry";

describe("parseQuickEntry", () => {
  it("keeps plain text as the title", () => {
    expect(parseQuickEntry("  Reisekosten   abrechnen ")).toEqual({
      title: "Reisekosten abrechnen",
      priority: "none",
      tagNames: [],
    });
  });

  it("reads tags and priority anywhere in the text", () => {
    expect(parseQuickEntry("#arbeit Angebot !!! schreiben #Kunde")).toEqual({
      title: "Angebot schreiben",
      priority: "high",
      tagNames: ["arbeit", "Kunde"],
    });
    expect(parseQuickEntry("Zahnarzt anrufen !").priority).toBe("low");
    expect(parseQuickEntry("Zahnarzt anrufen !!").priority).toBe("medium");
  });

  it("uses the last priority and drops duplicate tags", () => {
    const entry = parseQuickEntry("A ! #x #X !!!");
    expect(entry.priority).toBe("high");
    expect(entry.tagNames).toEqual(["x"]);
  });

  it("leaves markers that are part of words or stand alone in the title", () => {
    expect(parseQuickEntry("Wow!! # C#").title).toBe("Wow!! # C#");
    expect(parseQuickEntry("!!!!").title).toBe("!!!!");
  });

  it("returns an empty title when only markers were typed", () => {
    expect(parseQuickEntry("#tag !!").title).toBe("");
  });
});

describe("resolveTags", () => {
  const tags: Tag[] = [
    { id: "1", name: "Arbeit", color: "blue" },
    { id: "2", name: "Kunde", color: "red" },
  ];

  it("matches existing tags ignoring case and lists the rest", () => {
    expect(resolveTags(["arbeit", "Neu", "KUNDE"], tags)).toEqual({
      ids: ["1", "2"],
      missing: ["Neu"],
    });
  });
});
