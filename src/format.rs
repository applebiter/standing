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

/// Day and month, for chart axes: `16 Oct`.
pub fn date_day_month(d: jiff::civil::Date) -> String {
    d.strftime("%-d %b").to_string()
}

/// A signed percentage with a true minus sign: `+1.30%`, `\u{2212}0.42%`.
pub fn signed_percent(value: f64) -> String {
    format!("{}%", signed(value))
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

/// An index level: `97.42`.
pub fn index(value: f64) -> String {
    format!("{value:.2}")
}

/// A signed change in index points, with a true minus sign: `+1.25`, `\u{2212}0.40`.
pub fn signed(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded > 0.0 {
        format!("+{rounded:.2}")
    } else if rounded < 0.0 {
        format!("\u{2212}{:.2}", -rounded)
    } else {
        "0.00".to_owned()
    }
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

    #[test]
    fn index_values() {
        assert_eq!(index(97.4249), "97.42");
        assert_eq!(signed(1.249), "+1.25");
        assert_eq!(signed(-0.4), "\u{2212}0.40");
        assert_eq!(signed(-0.001), "0.00");
    }
}
