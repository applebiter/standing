//! Gathers a quarter's record for the Quarterly Statement.

use std::collections::HashMap;

use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;
use rusqlite::Connection;

use super::statement::{Headwind, PeerResult, Quarter, QuarterData, Range, ThresholdChange};
use crate::copy;
use crate::db;
use crate::engine::constants::CONSERVATIVE_PEER_MARGIN;
use crate::engine::{adjustment, market, positions, run};
use crate::model::comparable::Origin;
use crate::model::event::EventKind;
use crate::model::rating::Subject;

/// Headwinds reported, weakest first.
const HEADWINDS: usize = 5;

/// Quarters from the profile's creation to today, most recent first.
pub fn quarters(conn: &Connection, today: Date, tz: &TimeZone) -> db::Result<Vec<Quarter>> {
    let first = db::user::get(conn)?
        .and_then(|p| p.created_at.parse::<Timestamp>().ok())
        .map_or(today, |t| market::day_of(t, tz));
    let mut out = Vec::new();
    let mut q = Quarter::containing(first);
    let last = Quarter::containing(today);
    while q <= last {
        out.push(q);
        q = q.next();
    }
    out.reverse();
    Ok(out)
}

fn range(points: &[(Timestamp, f64)]) -> Option<Range> {
    let (first, last) = (points.first()?, points.last()?);
    Some(Range {
        open: first.1,
        close: last.1,
        high: points.iter().map(|p| p.1).fold(f64::MIN, f64::max),
        low: points.iter().map(|p| p.1).fold(f64::MAX, f64::min),
    })
}

/// The local date of a stored timestamp, if it falls within [from, to).
fn dated_within(stamp: &str, from: Timestamp, to: Timestamp, tz: &TimeZone) -> Option<Date> {
    let t = stamp.parse::<Timestamp>().ok()?;
    (t >= from && t < to).then(|| market::day_of(t, tz))
}

/// The record for `quarter`, up to `today` if the quarter is in progress.
pub fn gather(
    conn: &Connection,
    quarter: Quarter,
    today: Date,
    tz: &TimeZone,
) -> db::Result<QuarterData> {
    let profile = db::user::get(conn)?;
    let as_of = quarter.last_day().min(today);
    let from = market::start_of(quarter.first_day(), tz);
    let to = market::start_of(as_of.tomorrow().expect("date in range"), tz);

    let psi = range(&db::snapshots::between(conn, Subject::Own, from, to)?);

    let history = db::threshold::history(conn)?; // newest first
    let threshold_at_end = history
        .iter()
        .find(|e| e.changed_at.parse::<Timestamp>().is_ok_and(|t| t < to))
        .map(|e| e.new);
    let mut threshold_changes: Vec<ThresholdChange> = history
        .iter()
        .filter_map(|e| {
            Some(ThresholdChange {
                date: dated_within(&e.changed_at, from, to, tz)?,
                prior: e.prior,
                new: e.new,
                reason: e.reason,
            })
        })
        .collect();
    threshold_changes.reverse();

    let all = db::comparables::all(conn)?;
    let names: HashMap<i64, &str> = all.iter().map(|c| (c.id, c.name.as_str())).collect();
    let mut peers = Vec::new();
    for c in all.iter().filter(|c| c.active) {
        peers.push(PeerResult {
            ticker: c.ticker.clone(),
            name: c.name.clone(),
            range: range(&db::snapshots::between(
                conn,
                Subject::Comparable(c.id),
                from,
                to,
            )?),
        });
    }
    let mut introduced: Vec<(Date, String)> = all
        .iter()
        .filter(|c| c.origin == Origin::Archetype)
        .filter_map(|c| Some((dated_within(&c.created_at, from, to, tz)?, c.name.clone())))
        .collect();
    introduced.sort();

    let dims = db::dimensions::visible(conn)?;
    let mut discovered: Vec<(Date, String)> = dims
        .iter()
        .filter_map(|d| {
            Some((
                dated_within(d.discovered_at.as_deref()?, from, to, tz)?,
                d.name.clone(),
            ))
        })
        .collect();
    discovered.sort();

    // Headwinds and positions use ratings current when the statement is prepared.
    let inputs = run::inputs(conn)?;
    let by_id: HashMap<&str, _> = dims.iter().map(|d| (d.id.as_str(), d)).collect();
    let mut headwinds: Vec<Headwind> = inputs
        .dimensions
        .iter()
        .filter_map(|d| {
            let own = adjustment::own(*inputs.own.get(&d.id)?);
            let (leader, best) = inputs
                .comparables
                .iter()
                .map(|c| {
                    let v = c
                        .ratings
                        .get(&d.id)
                        .map_or(own + CONSERVATIVE_PEER_MARGIN, |&r| adjustment::peer(r));
                    (c.id, v)
                })
                .max_by(|a, b| a.1.total_cmp(&b.1))?;
            let spread = own - best;
            let dim = by_id.get(d.id.as_str())?;
            (spread < 0.0).then(|| Headwind {
                dimension: dim.name.clone(),
                sector: dim.sector.clone(),
                spread,
                leader: names.get(&leader).copied().unwrap_or_default().to_owned(),
            })
        })
        .collect();
    headwinds.sort_by(|a, b| {
        a.spread
            .total_cmp(&b.spread)
            .then_with(|| a.dimension.cmp(&b.dimension))
    });
    headwinds.truncate(HEADWINDS);

    let mut counts: HashMap<EventKind, usize> = HashMap::new();
    for e in db::events::all(conn)? {
        if e.occurred_at >= from && e.occurred_at < to {
            *counts.entry(e.kind).or_default() += 1;
        }
    }
    let events = EventKind::ALL
        .iter()
        .map(|k| {
            (
                copy::filing::label(*k).to_owned(),
                counts.get(k).copied().unwrap_or(0),
            )
        })
        .collect();

    Ok(QuarterData {
        quarter,
        as_of,
        name: profile
            .as_ref()
            .map(|p| p.display_name.clone())
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| copy::reports::UNNAMED.to_owned()),
        psi,
        threshold_at_end,
        threshold_changes,
        headwinds,
        peers,
        introduced,
        discovered,
        positions_held: positions::held(&inputs).len(),
        events,
    })
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

    #[test]
    fn gathers_the_quarter() {
        let mut conn = db::open_in_memory().unwrap();
        catalog::sync(
            &mut conn,
            &catalog::parse(
                "id,sector,industry,name,definition,discovery\n\
                 MAT-014,Material Holdings,Vehicles,Boat,Watercraft.,0\n\
                 DOM-004,Domestic Operations,Living Things,Sourdough starter age,Age.,0\n",
            )
            .unwrap(),
        )
        .unwrap();
        let t = at("2026-10-05T14:00:00Z");
        db::user::ensure(&conn, 9, at("2026-09-20T14:00:00Z")).unwrap();
        db::user::update(&conn, "Alex", None).unwrap();
        let dana = db::comparables::insert(
            &conn,
            &ComparableFields {
                name: "Dana".into(),
                relationship: None,
            },
            t,
        )
        .unwrap();
        db::ratings::insert(&conn, Subject::Own, "MAT-014", 9.0, t).unwrap();
        db::ratings::insert(&conn, Subject::Own, "DOM-004", 2.0, t).unwrap();
        db::ratings::insert(&conn, Subject::Comparable(dana.id), "MAT-014", 3.0, t).unwrap();
        db::ratings::insert(&conn, Subject::Comparable(dana.id), "DOM-004", 9.0, t).unwrap();
        db::threshold::set(&conn, 90.0, db::threshold::Reason::Initial, t).unwrap();
        run::recompute(&mut conn, t, &tz()).unwrap();
        run::recompute(&mut conn, at("2026-10-10T14:00:00Z"), &tz()).unwrap();

        let today = jiff::civil::date(2026, 10, 16);
        let qs = quarters(&conn, today, &tz()).unwrap();
        assert_eq!(
            qs,
            [Quarter { year: 2026, q: 4 }, Quarter { year: 2026, q: 3 }]
        );

        let d = gather(&conn, qs[0], today, &tz()).unwrap();
        assert_eq!(d.name, "Alex");
        assert_eq!(d.as_of, today);
        assert!(d.psi.is_some());
        assert_eq!(d.threshold_changes.len(), 1);
        assert_eq!(d.threshold_changes[0].date, jiff::civil::date(2026, 10, 5));
        assert_eq!(d.headwinds.len(), 1);
        assert_eq!(d.headwinds[0].dimension, "Sourdough starter age");
        assert_eq!(d.headwinds[0].leader, "Dana");
        assert_eq!(d.peers[0].ticker, "DANA");
        assert_eq!(d.events.len(), EventKind::ALL.len());

        let earlier = gather(&conn, qs[1], today, &tz()).unwrap();
        assert!(earlier.psi.is_none());
        assert!(earlier.threshold_changes.is_empty());
    }
}
