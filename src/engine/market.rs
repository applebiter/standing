//! Market days and daily candles.
//!
//! The market opens at the first launch on each local calendar day and closes
//! at local midnight. A day missed entirely carries the previous close.

use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;

/// One market day: open, high, low and close.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    pub day: Date,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Candle {
    fn flat(day: Date, value: f64) -> Self {
        Self {
            day,
            open: value,
            high: value,
            low: value,
            close: value,
        }
    }
}

/// The local calendar day of an instant.
pub fn day_of(t: Timestamp, tz: &TimeZone) -> Date {
    t.to_zoned(tz.clone()).date()
}

/// The instant a local calendar day begins.
pub fn start_of(day: Date, tz: &TimeZone) -> Timestamp {
    day.to_zoned(tz.clone())
        .expect("every calendar day in range has a start")
        .timestamp()
}

/// Builds one candle per day from `first` to `last` inclusive, from index
/// values in chronological order. `prior_close` is the last value before
/// `first`, if any. Days without values carry the previous close; days before
/// any value are omitted.
pub fn daily_candles(
    points: &[(Timestamp, f64)],
    tz: &TimeZone,
    first: Date,
    last: Date,
    prior_close: Option<f64>,
) -> Vec<Candle> {
    let mut candles = Vec::new();
    let mut previous = prior_close;
    let mut remaining = points;
    let mut day = first;
    while day <= last {
        let end = start_of(day.tomorrow().expect("date in range"), tz);
        let split = remaining.partition_point(|(t, _)| *t < end);
        let (today, rest) = remaining.split_at(split);
        let today: Vec<f64> = today
            .iter()
            .filter(|(t, _)| day_of(*t, tz) == day)
            .map(|(_, v)| *v)
            .collect();
        remaining = rest;
        if let (Some(&open), Some(&close)) = (today.first(), today.last()) {
            let candle = Candle {
                day,
                open,
                high: today.iter().copied().fold(f64::MIN, f64::max),
                low: today.iter().copied().fold(f64::MAX, f64::min),
                close,
            };
            previous = Some(close);
            candles.push(candle);
        } else if let Some(close) = previous {
            candles.push(Candle::flat(day, close));
        }
        day = day.tomorrow().expect("date in range");
    }
    candles
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tz() -> TimeZone {
        TimeZone::get("America/New_York").unwrap()
    }

    fn at(s: &str) -> Timestamp {
        s.parse().unwrap()
    }

    fn d(day: i8) -> Date {
        jiff::civil::date(2026, 10, day)
    }

    #[test]
    fn days_follow_local_time() {
        // 03:00 UTC on the 17th is 23:00 on the 16th in New York.
        assert_eq!(day_of(at("2026-10-17T03:00:00Z"), &tz()), d(16));
        assert_eq!(start_of(d(17), &tz()), at("2026-10-17T04:00:00Z"));
    }

    #[test]
    fn candles_take_open_high_low_close() {
        let points = [
            (at("2026-10-16T13:00:00Z"), 98.0),
            (at("2026-10-16T15:00:00Z"), 101.5),
            (at("2026-10-16T18:00:00Z"), 96.25),
            (at("2026-10-17T03:00:00Z"), 99.0),
        ];
        let c = daily_candles(&points, &tz(), d(16), d(16), None);
        assert_eq!(
            c,
            [Candle {
                day: d(16),
                open: 98.0,
                high: 101.5,
                low: 96.25,
                close: 99.0
            }]
        );
    }

    #[test]
    fn missed_days_carry_the_previous_close() {
        let points = [
            (at("2026-10-14T13:00:00Z"), 100.0),
            (at("2026-10-14T14:00:00Z"), 97.0),
            (at("2026-10-16T13:00:00Z"), 99.0),
        ];
        let c = daily_candles(&points, &tz(), d(13), d(17), None);
        let closes: Vec<(Date, f64, f64)> = c.iter().map(|c| (c.day, c.open, c.close)).collect();
        assert_eq!(
            closes,
            [
                (d(14), 100.0, 97.0),
                (d(15), 97.0, 97.0),
                (d(16), 99.0, 99.0),
                (d(17), 99.0, 99.0)
            ]
        );
    }

    #[test]
    fn prior_close_fills_leading_days() {
        let c = daily_candles(&[], &tz(), d(16), d(17), Some(91.0));
        assert_eq!(c, [Candle::flat(d(16), 91.0), Candle::flat(d(17), 91.0)]);
        assert!(daily_candles(&[], &tz(), d(16), d(17), None).is_empty());
    }
}
