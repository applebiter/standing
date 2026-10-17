//! The `market_days` table: which days the market opened, and whether each
//! has been closed.

use jiff::Timestamp;
use jiff::civil::Date;
use rusqlite::{Connection, params};

use super::{Result, stamp};

/// Records today's open. Returns `false` if the market had already opened today.
pub fn open(conn: &Connection, day: Date, now: Timestamp) -> Result<bool> {
    let n = conn.execute(
        "INSERT INTO market_days (day, opened_at) VALUES (?1, ?2) ON CONFLICT (day) DO NOTHING",
        params![day.to_string(), stamp(now)],
    )?;
    Ok(n == 1)
}

/// Days that opened before `today` and have not been closed, oldest first.
pub fn unclosed_before(conn: &Connection, today: Date) -> Result<Vec<Date>> {
    let mut stmt = conn
        .prepare("SELECT day FROM market_days WHERE closed_at IS NULL AND day < ?1 ORDER BY day")?;
    let rows = stmt.query_map([today.to_string()], |r| r.get::<_, String>(0))?;
    let mut out = Vec::new();
    for row in rows {
        if let Ok(day) = row?.parse() {
            out.push(day);
        }
    }
    Ok(out)
}

pub fn mark_closed(conn: &Connection, day: Date, now: Timestamp) -> Result<()> {
    conn.execute(
        "UPDATE market_days SET closed_at = ?2 WHERE day = ?1",
        params![day.to_string(), stamp(now)],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn opens_once_per_day_and_closes() {
        let conn = db::open_in_memory().unwrap();
        let now: Timestamp = "2026-10-16T12:00:00Z".parse().unwrap();
        let (d15, d16) = (
            jiff::civil::date(2026, 10, 15),
            jiff::civil::date(2026, 10, 16),
        );
        assert!(open(&conn, d15, now).unwrap());
        assert!(!open(&conn, d15, now).unwrap());
        assert!(open(&conn, d16, now).unwrap());
        assert_eq!(unclosed_before(&conn, d16).unwrap(), [d15]);
        mark_closed(&conn, d15, now).unwrap();
        assert!(unclosed_before(&conn, d16).unwrap().is_empty());
    }
}
