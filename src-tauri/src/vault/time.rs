//! Timestamps in vault files: ISO 8601 in UTC with milliseconds, e.g.
//! `2026-10-23T09:14:05.123Z`. In memory they are Unix milliseconds.

const MS_PER_DAY: i64 = 86_400_000;

pub fn format(ms: i64) -> String {
    let (year, month, day) = civil_from_days(ms.div_euclid(MS_PER_DAY));
    let rest = ms.rem_euclid(MS_PER_DAY);
    let (hour, minute, second, milli) = (
        rest / 3_600_000,
        rest / 60_000 % 60,
        rest / 1000 % 60,
        rest % 1000,
    );
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{milli:03}Z")
}

/// Also accepts what people type by hand: no or more fraction digits and a
/// `±HH:MM` offset instead of `Z`.
pub fn parse(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if bytes.len() < 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !matches!(bytes[10], b'T' | b't')
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return None;
    }
    let field = |start: usize, len: usize| digits(text.get(start..start + len)?);
    let (year, month, day) = (field(0, 4)?, field(5, 2)?, field(8, 2)?);
    let (hour, minute, second) = (field(11, 2)?, field(14, 2)?, field(17, 2)?);
    if !(1..=12).contains(&month)
        || !(1..=days_in_month(year, month)).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }

    let mut rest = &text[19..];
    let mut milli = 0;
    if let Some(fraction) = rest.strip_prefix('.') {
        let len = fraction.bytes().take_while(u8::is_ascii_digit).count();
        if len == 0 {
            return None;
        }
        milli = digits(&format!("{:0<3}", &fraction[..len.min(3)]))?;
        rest = &fraction[len..];
    }

    let offset_minutes = match rest {
        "Z" | "z" => 0,
        _ => {
            let bytes = rest.as_bytes();
            if bytes.len() != 6 || bytes[3] != b':' {
                return None;
            }
            let sign = match bytes[0] {
                b'+' => 1,
                b'-' => -1,
                _ => return None,
            };
            let (hours, minutes) = (digits(&rest[1..3])?, digits(&rest[4..6])?);
            if hours > 23 || minutes > 59 {
                return None;
            }
            sign * (hours * 60 + minutes)
        }
    };

    let minutes = (days_from_civil(year, month, day) * 24 + hour) * 60 + minute - offset_minutes;
    Some(minutes * 60_000 + second * 1000 + milli)
}

fn digits(text: &str) -> Option<i64> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// Calendar conversions after Howard Hinnant's `days_from_civil` and
// `civil_from_days`; days are counted from 1970-01-01.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    (year_of_era + era * 400 + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_known_instants() {
        assert_eq!(format(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(format(1_792_746_845_123), "2026-10-23T09:14:05.123Z");
        assert_eq!(format(951_868_799_999), "2000-02-29T23:59:59.999Z");
        assert_eq!(format(-1000), "1969-12-31T23:59:59.000Z");
    }

    #[test]
    fn parses_what_it_formats() {
        for ms in [
            0,
            1,
            -1,
            951_868_799_999,
            1_792_746_845_123,
            4_102_444_800_000,
        ] {
            assert_eq!(parse(&format(ms)), Some(ms), "{ms}");
        }
    }

    #[test]
    fn parses_hand_written_variants() {
        let expected = Some(1_792_746_845_000);
        assert_eq!(parse("2026-10-23T09:14:05Z"), expected);
        assert_eq!(parse("2026-10-23t09:14:05z"), expected);
        assert_eq!(parse("2026-10-23T11:14:05+02:00"), expected);
        assert_eq!(parse("2026-10-23T08:44:05-00:30"), expected);
        assert_eq!(parse("2026-10-23T09:14:05.1Z"), Some(1_792_746_845_100));
        assert_eq!(
            parse("2026-10-23T09:14:05.123456Z"),
            Some(1_792_746_845_123)
        );
    }

    #[test]
    fn rejects_malformed_text() {
        for text in [
            "",
            "2026-10-23",
            "2026-10-23 09:14:05Z",
            "2026-10-23T09:14:05",
            "2026-13-01T00:00:00Z",
            "2025-02-29T00:00:00Z",
            "2026-10-23T24:00:00Z",
            "2026-10-23T09:14:05.Z",
            "2026-10-23T09:14:05+0200",
            "2026-10-23T09:14:05Zjunk",
            "+026-10-23T09:14:05Z",
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }
}
