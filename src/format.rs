//! Number formatting shared by the UI and reports.

/// Formats an integer with comma thousands separators: `1284` → `1,284`.
pub fn thousands(n: impl Into<u128>) -> String {
    let digits = n.into().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Long-form date as shown throughout the UI: `16 October 2026`.
pub fn date_long(d: jiff::civil::Date) -> String {
    d.strftime("%-d %B %Y").to_string()
}

/// Compact date for tables: `16 Oct 2026`.
pub fn date_short(d: jiff::civil::Date) -> String {
    d.strftime("%-d %b %Y").to_string()
}

/// The local calendar date of a stored timestamp, if it parses.
pub fn local_date(stamp: &str) -> Option<jiff::civil::Date> {
    let t: jiff::Timestamp = stamp.parse().ok()?;
    Some(t.to_zoned(jiff::tz::TimeZone::system()).date())
}

/// A rating on the 0–10 scale: `7.5`, `10.0`.
pub fn rating(value: f64) -> String {
    format!("{value:.1}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_separators() {
        assert_eq!(thousands(0u32), "0");
        assert_eq!(thousands(480u32), "480");
        assert_eq!(thousands(1284u32), "1,284");
        assert_eq!(thousands(1_000_000u32), "1,000,000");
    }

    #[test]
    fn dates_and_ratings() {
        let d = jiff::civil::date(2026, 10, 6);
        assert_eq!(date_long(d), "6 October 2026");
        assert_eq!(date_short(d), "6 Oct 2026");
        assert!(local_date("2026-10-16T12:00:00.000000Z").is_some());
        assert!(local_date("not a time").is_none());
        assert_eq!(rating(7.5), "7.5");
        assert_eq!(rating(10.0), "10.0");
    }
}
