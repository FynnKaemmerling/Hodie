//! Fractional indexing: string sort keys that allow inserting between any two
//! neighbours, so moving an item only rewrites that item's key.
//!
//! Keys use base-62 digits in ASCII order, so SQLite's default byte-wise
//! comparison sorts them correctly. A valid key is non-empty and never ends
//! in the zero digit, which guarantees there is always room before it.

const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: usize = DIGITS.len();

/// The neighbouring keys are malformed or not in ascending order.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidKeys;

fn digit_value(byte: u8) -> Option<usize> {
    DIGITS.iter().position(|&digit| digit == byte)
}

pub fn is_valid(key: &str) -> bool {
    !key.is_empty() && key.bytes().all(|byte| digit_value(byte).is_some()) && !key.ends_with('0')
}

/// Returns a key that sorts strictly between `before` and `after`.
/// `None` stands for the start or end of the list.
pub fn key_between(before: Option<&str>, after: Option<&str>) -> Result<String, InvalidKeys> {
    if before.is_some_and(|key| !is_valid(key)) || after.is_some_and(|key| !is_valid(key)) {
        return Err(InvalidKeys);
    }
    if let (Some(before), Some(after)) = (before, after)
        && before >= after
    {
        return Err(InvalidKeys);
    }
    Ok(midpoint(
        before.unwrap_or("").as_bytes(),
        after.map(str::as_bytes),
    ))
}

// `a < b`, both valid or `a` empty; `b == None` means "past the end".
fn midpoint(a: &[u8], b: Option<&[u8]>) -> String {
    if let Some(b) = b {
        // Copy the common prefix, treating a missing digit in `a` as zero.
        let mut n = 0;
        while n < b.len() && a.get(n).copied().unwrap_or(b'0') == b[n] {
            n += 1;
        }
        if n > 0 {
            let rest = midpoint(a.get(n..).unwrap_or(&[]), Some(&b[n..]));
            return format!("{}{rest}", String::from_utf8_lossy(&b[..n]));
        }
    }

    let digit_a = a.first().and_then(|&byte| digit_value(byte)).unwrap_or(0);
    let digit_b = b.and_then(|b| digit_value(b[0])).unwrap_or(BASE);

    if digit_b - digit_a > 1 {
        return char::from(DIGITS[(digit_a + digit_b).div_ceil(2)]).to_string();
    }
    match b {
        // The first digit of `b` alone already sorts between `a` and `b`.
        Some(b) if b.len() > 1 => char::from(b[0]).to_string(),
        _ => format!(
            "{}{}",
            char::from(DIGITS[digit_a]),
            midpoint(a.get(1..).unwrap_or(&[]), None)
        ),
    }
}

/// Key for inserting at `index` into `siblings` (sorted, without the moved item).
/// An index past the end appends.
pub fn key_at(siblings: &[String], index: usize) -> Result<String, InvalidKeys> {
    let index = index.min(siblings.len());
    let before = index.checked_sub(1).map(|i| siblings[i].as_str());
    let after = siblings.get(index).map(String::as_str);
    key_between(before, after)
}

/// Key for appending after the last of `siblings` (sorted).
pub fn key_after_last(siblings: &[String]) -> Result<String, InvalidKeys> {
    key_at(siblings, siblings.len())
}

/// Returns `wanted` if no sibling uses it, otherwise a key right after it.
/// Used to put restored items back where they were.
pub fn free_key(siblings: &[String], wanted: &str) -> Result<String, InvalidKeys> {
    if !siblings.iter().any(|key| key == wanted) {
        return if is_valid(wanted) {
            Ok(wanted.to_owned())
        } else {
            Err(InvalidKeys)
        };
    }
    let next = siblings
        .iter()
        .filter(|key| key.as_str() > wanted)
        .min()
        .map(String::as_str);
    key_between(Some(wanted), next)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_between(before: Option<&str>, after: Option<&str>) -> String {
        let key = key_between(before, after).unwrap();
        assert!(is_valid(&key), "invalid key {key:?}");
        if let Some(before) = before {
            assert!(before < key.as_str(), "{before:?} < {key:?}");
        }
        if let Some(after) = after {
            assert!(key.as_str() < after, "{key:?} < {after:?}");
        }
        key
    }

    #[test]
    fn first_key_is_in_the_middle() {
        assert_eq!(key_between(None, None).unwrap(), "V");
    }

    #[test]
    fn appending_keeps_order() {
        let mut last: Option<String> = None;
        for _ in 0..1000 {
            last = Some(assert_between(last.as_deref(), None));
        }
    }

    #[test]
    fn prepending_keeps_order() {
        let mut first: Option<String> = None;
        for _ in 0..1000 {
            first = Some(assert_between(None, first.as_deref()));
        }
    }

    #[test]
    fn repeated_inserts_between_two_keys() {
        let low = "V".to_owned();
        let mut high = "W".to_owned();
        for _ in 0..200 {
            high = assert_between(Some(&low), Some(&high));
        }
        let mut low = "V".to_owned();
        let high = "W".to_owned();
        for _ in 0..200 {
            low = assert_between(Some(&low), Some(&high));
        }
    }

    #[test]
    fn handles_adjacent_and_prefix_keys() {
        assert_between(Some("1"), Some("2"));
        assert_between(Some("A"), Some("A1"));
        assert_between(Some("A1"), Some("B"));
        assert_between(None, Some("1"));
        assert_between(None, Some("01"));
        assert_between(Some("z"), None);
        assert_between(Some("zz"), None);
        assert_between(Some("Az"), Some("B"));
    }

    #[test]
    fn rejects_invalid_input() {
        assert_eq!(key_between(Some("B"), Some("A")), Err(InvalidKeys));
        assert_eq!(key_between(Some("A"), Some("A")), Err(InvalidKeys));
        assert_eq!(key_between(Some("A0"), None), Err(InvalidKeys));
        assert_eq!(key_between(Some(""), None), Err(InvalidKeys));
        assert_eq!(key_between(Some("A-"), None), Err(InvalidKeys));
    }

    #[test]
    fn key_at_inserts_at_index() {
        let siblings = vec!["A".to_owned(), "M".to_owned(), "a".to_owned()];
        let start = key_at(&siblings, 0).unwrap();
        assert!(start.as_str() < "A");
        let middle = key_at(&siblings, 2).unwrap();
        assert!("M" < middle.as_str() && middle.as_str() < "a");
        let end = key_at(&siblings, 99).unwrap();
        assert!(end.as_str() > "a");
        assert_eq!(key_at(&[], 0).unwrap(), "V");
    }

    #[test]
    fn free_key_avoids_collisions() {
        let siblings = vec!["A".to_owned(), "M".to_owned()];
        assert_eq!(free_key(&siblings, "F").unwrap(), "F");
        let moved = free_key(&siblings, "A").unwrap();
        assert!("A" < moved.as_str() && moved.as_str() < "M");
        assert!(free_key(&siblings, "M").unwrap().as_str() > "M");
    }
}
