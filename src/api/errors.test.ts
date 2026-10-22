import { describe, expect, it } from "vitest";
import de from "../locales/de.json";
import { ERROR_CODES, errorMessageKey } from "./errors";

function lookup(key: string): unknown {
  return key
    .split(".")
    .reduce<unknown>((node, part) => (node as Record<string, unknown> | undefined)?.[part], de);
}

describe("errorMessageKey", () => {
  it("has a translation for every backend error code", () => {
    for (const code of [...ERROR_CODES, "unknown"]) {
      expect(typeof lookup(`errors.${code}`), code).toBe("string");
    }
  });

  it("maps known codes and falls back for anything else", () => {
    expect(errorMessageKey("tag.duplicate_name")).toBe("errors.tag.duplicate_name");
    expect(errorMessageKey("something else")).toBe("errors.unknown");
    expect(errorMessageKey(new Error("boom"))).toBe("errors.unknown");
  });
});
