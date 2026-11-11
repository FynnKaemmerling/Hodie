import { describe, expect, it } from "vitest";
import { formatBytes, formatCountdown } from "./format";

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

describe("formatCountdown", () => {
  it("shows minutes and seconds, rounding up to the next second", () => {
    expect(formatCountdown(25 * 60_000, "de")).toBe("25:00");
    expect(formatCountdown(61_500, "en")).toBe("01:02");
    expect(formatCountdown(1, "en")).toBe("00:01");
    expect(formatCountdown(0, "en")).toBe("00:00");
  });
});
