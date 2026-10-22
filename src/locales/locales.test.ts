import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";

type Messages = { [key: string]: string | Messages };

function flattenKeys(messages: Messages, prefix = ""): string[] {
  return Object.entries(messages).flatMap(([key, value]) => {
    const path = prefix ? `${prefix}.${key}` : key;
    return typeof value === "string" ? [path] : flattenKeys(value, path);
  });
}

const reference = flattenKeys(de).sort();
const locales: Record<string, Messages> = { en };

describe("locales", () => {
  for (const [name, messages] of Object.entries(locales)) {
    it(`${name} has exactly the same keys as de`, () => {
      expect(flattenKeys(messages).sort()).toEqual(reference);
    });
  }

  it("has no empty translations", () => {
    for (const messages of [de, ...Object.values(locales)]) {
      const empty = flattenKeys(messages).filter((path) => {
        const value = path
          .split(".")
          .reduce<string | Messages>((node, key) => (node as Messages)[key], messages);
        return typeof value === "string" && value.trim() === "";
      });
      expect(empty).toEqual([]);
    }
  });
});
