//! Applies the methodology to the stored record: recomputation and market
//! hours.

use std::collections::HashMap;

use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;
use rusqlite::Connection;

use super::index::{self, ComparableRatings, Inputs, Reading, WeightedDimension};
use super::market;
use super::threshold::{self, Adjustment};
use crate::copy;
use crate::db::{self, notices};
use crate::model::rating::Subject;

/// Gathers the current record into the index's inputs.
pub fn inputs(conn: &Connection) -> db::Result<Inputs> {
    let dimensions: Vec<WeightedDimension> = db::dimensions::visible(conn)?
        .into_iter()
        .filter(|d| d.is_active())
        .map(|d| WeightedDimension {
            id: d.id,
            weight: d.weight,
        })
        .collect();
    let current = db::ratings::current(conn)?;
    let mut own = HashMap::new();
    let mut by_comparable: HashMap<i64, HashMap<String, f64>> = HashMap::new();
    for ((subject, dim), rating) in current {
        match subject {
            Subject::Own => {
                own.insert(dim, rating.value);
            }
            Subject::Comparable(id) => {
                by_comparable
                    .entry(id)
                    .or_default()
                    .insert(dim, rating.value);
            }
        }
    }
    let comparables = db::comparables::active(conn)?
        .into_iter()
        .map(|c| ComparableRatings {
            ratings: by_comparable.remove(&c.id).unwrap_or_default(),
            id: c.id,
        })
        .collect();
    Ok(Inputs {
        dimensions,
        own,
        comparables,
        events: db::events::all(conn)?,
    })
}

/// The result of a recomputation.
#[derive(Debug)]
pub struct Recomputed {
    pub reading: Reading,
    /// Enough Threshold adjustments made, in order.
    pub adjustments: Vec<Adjustment>,
}

/// Recomputes the index from the record, records the result, and adjusts the
/// Enough Threshold if the index has reached it.
pub fn recompute(conn: &mut Connection, now: Timestamp) -> db::Result<Recomputed> {
    let reading = index::compute(&inputs(conn)?, now);
    db::snapshots::insert_reading(conn, &reading)?;
    let adjustments = match db::threshold::current(conn)? {
        Some(t) => threshold::adjustments(t, reading.psi),
        None => Vec::new(),
    };
    if !adjustments.is_empty() {
        let tx = conn.transaction()?;
        for a in &adjustments {
            db::threshold::set(&tx, a.new, db::threshold::Reason::Adjustment, now)?;
            notices::insert(
                &tx,
                notices::kind::THRESHOLD,
                &copy::notices::threshold_adjusted(a.prior, a.new),
                now,
            )?;
        }
        tx.commit()?;
    }
    Ok(Recomputed {
        reading,
        adjustments,
    })
}

/// What a market check did.
#[derive(Debug, Default, PartialEq)]
pub struct MarketActivity {
    /// Days closed, oldest first.
    pub closed: Vec<Date>,
    /// Whether the market opened for `today`.
    pub opened: bool,
}

/// Closes every earlier market day not yet closed, posting each day's
/// summary, then opens today's market if it has not opened.
pub fn market_check(
    conn: &Connection,
    now: Timestamp,
    tz: &TimeZone,
) -> db::Result<MarketActivity> {
    let today = market::day_of(now, tz);
    let mut activity = MarketActivity::default();
    for day in db::market::unclosed_before(conn, today)? {
        close(conn, day, now, tz)?;
        activity.closed.push(day);
    }
    activity.opened = db::market::open(conn, today, now)?;
    Ok(activity)
}

fn close(conn: &Connection, day: Date, now: Timestamp, tz: &TimeZone) -> db::Result<()> {
    let start = market::start_of(day, tz);
    let end = market::start_of(day.tomorrow().expect("date in range"), tz);
    let previous = db::snapshots::last_before(conn, Subject::Own, start)?;
    let points = db::snapshots::between(conn, Subject::Own, start, end)?;
    if let Some(candle) = market::daily_candles(&points, tz, day, day, previous).first() {
        let body = copy::notices::market_close(day, candle, previous);
        notices::insert(conn, notices::kind::MARKET_CLOSE, &body, now)?;
    }
    db::market::mark_closed(conn, day, now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::model::comparable::ComparableFields;

    fn tz() -> TimeZone {
        TimeZone::get("America/New_York").unwrap()
    }

    fn at(s: &str) -> Timestamp {
        s.parse().unwrap()
    }

    fn setup() -> Connection {
        let mut conn = db::open_in_memory().unwrap();
        catalog::sync(
            &mut conn,
            &catalog::parse(
                "id,sector,industry,name,definition,discovery\n\
                 MAT-014,Material Holdings,Vehicles,Boat,Watercraft.,0\n\
                 DOM-004,Domestic Operations,Living Things,Sourdough starter age,Age.,1\n",
            )
            .unwrap(),
        )
        .unwrap();
        conn
    }

    #[test]
    fn recompute_reads_the_record_and_records_snapshots() {
        let mut conn = setup();
        let t = at("2026-10-16T14:00:00Z");
        let dana = db::comparables::insert(
            &conn,
            &ComparableFields {
                name: "Dana".into(),
                relationship: None,
            },
            t,
        )
        .unwrap();
        db::ratings::insert(&conn, Subject::Own, "MAT-014", 10.0, t).unwrap();
        db::ratings::insert(&conn, Subject::Comparable(dana.id), "MAT-014", 10.0, t).unwrap();
        // Hidden dimensions do not count, even if rated.
        db::ratings::insert(&conn, Subject::Own, "DOM-004", 0.0, t).unwrap();

        let r = recompute(&mut conn, t).unwrap().reading;
        assert!((r.psi - 75.0).abs() < 1e-9);
        assert!((r.comparables[0].value - 125.0).abs() < 1e-9);
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM index_snapshots", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 2);
    }

    #[test]
    fn removed_comparables_leave_the_index() {
        let mut conn = setup();
        let t = at("2026-10-16T14:00:00Z");
        let dana = db::comparables::insert(
            &conn,
            &ComparableFields {
                name: "Dana".into(),
                relationship: None,
            },
            t,
        )
        .unwrap();
        db::ratings::insert(&conn, Subject::Own, "MAT-014", 4.0, t).unwrap();
        db::comparables::set_active(&conn, dana.id, false).unwrap();
        let r = recompute(&mut conn, t).unwrap().reading;
        assert!(r.comparables.is_empty());
        assert!((r.psi - 100.0).abs() < 1e-9);
    }

    #[test]
    fn reaching_the_threshold_adjusts_it_with_history_and_notices() {
        let mut conn = setup();
        let t = at("2026-10-16T14:00:00Z");
        // No Comparables: PSI = 100.
        db::threshold::set(&conn, 50.0, db::threshold::Reason::Initial, t).unwrap();
        let r = recompute(&mut conn, t).unwrap();
        let steps: Vec<(f64, f64)> = r.adjustments.iter().map(|a| (a.prior, a.new)).collect();
        assert_eq!(
            steps,
            [(50.0, 60.0), (60.0, 72.0), (72.0, 87.0), (87.0, 105.0)]
        );
        assert_eq!(db::threshold::current(&conn).unwrap(), Some(105.0));
        let history = db::threshold::history(&conn).unwrap();
        assert_eq!(history.len(), 5);
        assert!(
            history[..4]
                .iter()
                .all(|e| e.reason == db::threshold::Reason::Adjustment)
        );
        let notices = notices::bodies(&conn, notices::kind::THRESHOLD);
        assert_eq!(notices.len(), 4);
        assert!(
            notices
                .iter()
                .all(|n| n.ends_with(copy::notices::THRESHOLD_REASON))
        );

        // Below the threshold now: nothing further.
        assert!(recompute(&mut conn, t).unwrap().adjustments.is_empty());
    }

    #[test]
    fn exactly_at_the_threshold_counts_as_reaching_it() {
        let mut conn = setup();
        let t = at("2026-10-16T14:00:00Z");
        db::threshold::set(&conn, 100.0, db::threshold::Reason::Initial, t).unwrap();
        let r = recompute(&mut conn, t).unwrap();
        assert_eq!(r.adjustments.len(), 1);
        assert_eq!(db::threshold::current(&conn).unwrap(), Some(120.0));
    }

    #[test]
    fn market_opens_once_a_day_and_closes_missed_days() {
        let mut conn = setup();
        db::ratings::insert(
            &conn,
            Subject::Own,
            "MAT-014",
            4.0,
            at("2026-10-14T13:00:00Z"),
        )
        .unwrap();

        // Morning of the 14th, New York.
        let monday = at("2026-10-14T13:00:00Z");
        let a = market_check(&conn, monday, &tz()).unwrap();
        assert_eq!(
            a,
            MarketActivity {
                closed: vec![],
                opened: true
            }
        );
        recompute(&mut conn, monday).unwrap();
        let again = market_check(&conn, at("2026-10-14T20:00:00Z"), &tz()).unwrap();
        assert!(!again.opened);

        // Next launch on the 16th: the 14th closes; the 15th never opened.
        let wednesday = at("2026-10-16T13:00:00Z");
        let b = market_check(&conn, wednesday, &tz()).unwrap();
        assert_eq!(b.closed, [jiff::civil::date(2026, 10, 14)]);
        assert!(b.opened);
        let summaries = notices::bodies(&conn, notices::kind::MARKET_CLOSE);
        assert_eq!(summaries.len(), 1);
        assert!(summaries[0].contains("14 October 2026"), "{}", summaries[0]);
        assert!(summaries[0].contains("100.00"), "{}", summaries[0]);
    }
}
