import { describe, expect, it } from "vitest";
import { formatBytes } from "./format";

// Intl may separate number and unit with a narrow no-break space.
const plain = (text: string) => text.replace(/\s/g, " ");

describe("formatBytes", () => {
  it("picks a unit that keeps the number small", () => {
    expect(plain(formatBytes(512, "en"))).toBe("512 byte");
    expect(plain(formatBytes(1_234_567, "en"))).toBe("1.2 MB");
    expect(plain(formatBytes(3_000_000_000, "en"))).toBe("3 GB");
  });

  it("follows the language", () => {
    expect(plain(formatBytes(1_500, "de"))).toBe("1,5 kB");
  });
});
