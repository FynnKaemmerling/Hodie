const UNITS = ["byte", "kilobyte", "megabyte", "gigabyte"] as const;

/** A size in bytes in the user's language, e.g. "1,2 MB" (decimal units). */
export function formatBytes(bytes: number, locale: string): string {
  let value = bytes;
  let unit = 0;
  while (value >= 1000 && unit < UNITS.length - 1) {
    value /= 1000;
    unit += 1;
  }
  return new Intl.NumberFormat(locale, {
    style: "unit",
    unit: UNITS[unit],
    unitDisplay: "short",
    maximumFractionDigits: unit === 0 ? 0 : 1,
  }).format(value);
}

/** A countdown as minutes and seconds, e.g. "24:59", with the language's digits. */
export function formatCountdown(ms: number, locale: string): string {
  const totalSeconds = Math.ceil(Math.max(0, ms) / 1000);
  const minutes = new Intl.NumberFormat(locale, { minimumIntegerDigits: 2 });
  return `${minutes.format(Math.floor(totalSeconds / 60))}:${minutes.format(totalSeconds % 60)}`;
}
