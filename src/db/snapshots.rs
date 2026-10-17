//! Index values, recorded at every recomputation.

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};

use super::{Result, stamp};
use crate::engine::index::Reading;
use crate::model::rating::Subject;

/// Records the user's index and every active Comparable's ticker.
pub fn insert_reading(conn: &mut Connection, reading: &Reading) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO index_snapshots (subject_kind, comparable_id, value, computed_at)
             VALUES (?1, ?2, ?3, ?4)",
        )?;
        let at = stamp(reading.computed_at);
        stmt.execute(params![Subject::Own.kind(), None::<i64>, reading.psi, at])?;
        for c in &reading.comparables {
            let subject = Subject::Comparable(c.id);
            stmt.execute(params![
                subject.kind(),
                subject.comparable_id(),
                c.value,
                at
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

fn subject_clause() -> &'static str {
    "subject_kind = ?1 AND comparable_id IS ?2"
}

/// Values for one subject from `from` (inclusive) to `to` (exclusive), oldest first.
pub fn between(
    conn: &Connection,
    subject: Subject,
    from: Timestamp,
    to: Timestamp,
) -> Result<Vec<(Timestamp, f64)>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT computed_at, value FROM index_snapshots
         WHERE {} AND computed_at >= ?3 AND computed_at < ?4
         ORDER BY computed_at, id",
        subject_clause()
    ))?;
    let rows = stmt.query_map(
        params![
            subject.kind(),
            subject.comparable_id(),
            stamp(from),
            stamp(to)
        ],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)),
    )?;
    let mut out = Vec::new();
    for row in rows {
        let (at, value) = row?;
        if let Ok(t) = at.parse() {
            out.push((t, value));
        }
    }
    Ok(out)
}

/// When the first value for one subject was recorded.
pub fn first_at(conn: &Connection, subject: Subject) -> Result<Option<Timestamp>> {
    let first: Option<String> = conn
        .query_row(
            &format!(
                "SELECT MIN(computed_at) FROM index_snapshots WHERE {}",
                subject_clause()
            ),
            params![subject.kind(), subject.comparable_id()],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    Ok(first.and_then(|s| s.parse().ok()))
}

/// The last value for one subject before `before`.
pub fn last_before(conn: &Connection, subject: Subject, before: Timestamp) -> Result<Option<f64>> {
    Ok(conn
        .query_row(
            &format!(
                "SELECT value FROM index_snapshots
                 WHERE {} AND computed_at < ?3
                 ORDER BY computed_at DESC, id DESC LIMIT 1",
                subject_clause()
            ),
            params![subject.kind(), subject.comparable_id(), stamp(before)],
            |r| r.get(0),
        )
        .optional()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::engine::index::ComparableReading;

    fn at(s: &str) -> Timestamp {
        s.parse().unwrap()
    }

    fn reading(when: &str, psi: f64) -> Reading {
        Reading {
            computed_at: at(when),
            own_composite: None,
            momentum: 0.0,
            psi,
            comparables: vec![],
        }
    }

    #[test]
    fn readings_are_recorded_per_subject() {
        let mut conn = db::open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO comparables (id, name, ticker, origin, created_at)
             VALUES (7, 'Dana', 'DANA', 'user', '2026-10-16T00:00:00.000000Z')",
            [],
        )
        .unwrap();
        let mut r = reading("2026-10-16T10:00:00Z", 97.5);
        r.comparables.push(ComparableReading {
            id: 7,
            composite: None,
            momentum: 0.0,
            value: 104.0,
        });
        insert_reading(&mut conn, &r).unwrap();
        insert_reading(&mut conn, &reading("2026-10-16T11:00:00Z", 96.0)).unwrap();

        let all = |s| {
            between(
                &conn,
                s,
                at("2026-10-16T00:00:00Z"),
                at("2026-10-17T00:00:00Z"),
            )
            .unwrap()
        };
        assert_eq!(
            all(Subject::Own),
            [
                (at("2026-10-16T10:00:00Z"), 97.5),
                (at("2026-10-16T11:00:00Z"), 96.0)
            ]
        );
        assert_eq!(
            all(Subject::Comparable(7)),
            [(at("2026-10-16T10:00:00Z"), 104.0)]
        );
        assert_eq!(
            last_before(&conn, Subject::Own, at("2026-10-16T11:00:00Z")).unwrap(),
            Some(97.5)
        );
        assert_eq!(
            last_before(&conn, Subject::Own, at("2026-10-16T10:00:00Z")).unwrap(),
            None
        );
    }
}
