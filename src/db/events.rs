//! Queries over the `events` table.

use rusqlite::Connection;

use super::Result;
use crate::model::event::{Event, EventKind};

fn conversion_error(column: usize, message: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(column, rusqlite::types::Type::Text, message.into())
}

/// Every event on record, oldest first.
pub fn all(conn: &Connection) -> Result<Vec<Event>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, comparable_id, note, occurred_at FROM events ORDER BY occurred_at, id",
    )?;
    let rows = stmt.query_map([], |r| {
        let kind: String = r.get(1)?;
        let occurred_at: String = r.get(4)?;
        Ok(Event {
            id: r.get(0)?,
            kind: EventKind::parse(&kind)
                .ok_or_else(|| conversion_error(1, format!("unknown event kind {kind:?}")))?,
            comparable_id: r.get(2)?,
            note: r.get(3)?,
            occurred_at: occurred_at
                .parse()
                .map_err(|e| conversion_error(4, format!("{e}")))?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Records an event.
#[cfg(test)] // Filed from the Daily Filing screen once it is installed.
pub fn insert(
    conn: &Connection,
    kind: EventKind,
    comparable_id: Option<i64>,
    note: Option<&str>,
    occurred_at: jiff::Timestamp,
) -> Result<i64> {
    use rusqlite::params;

    use super::stamp;
    conn.execute(
        "INSERT INTO events (kind, comparable_id, note, occurred_at) VALUES (?1, ?2, ?3, ?4)",
        params![kind.as_str(), comparable_id, note, stamp(occurred_at)],
    )?;
    Ok(conn.last_insert_rowid())
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;

    use super::*;
    use crate::db;

    #[test]
    fn events_round_trip_in_order() {
        let conn = db::open_in_memory().unwrap();
        let later: Timestamp = "2026-10-16T12:00:00Z".parse().unwrap();
        let earlier: Timestamp = "2026-10-15T12:00:00Z".parse().unwrap();
        insert(
            &conn,
            EventKind::SlightMinor,
            None,
            Some("Seated last"),
            later,
        )
        .unwrap();
        insert(&conn, EventKind::Achievement, None, None, earlier).unwrap();
        let events = all(&conn).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, EventKind::Achievement);
        assert_eq!(events[0].occurred_at, earlier);
        assert_eq!(events[1].note.as_deref(), Some("Seated last"));
    }
}
