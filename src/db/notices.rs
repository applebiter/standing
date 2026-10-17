//! Notices: rebalancing, discovery, threshold adjustments, recalibration and
//! market closes.

use jiff::Timestamp;
use rusqlite::{Connection, params};

use super::{Result, stamp};

/// Kinds of notice. Stored as their string value.
pub mod kind {
    pub const MARKET_CLOSE: &str = "market_close";
    pub const THRESHOLD: &str = "threshold";
    pub const REBALANCE: &str = "rebalance";
    pub const DISCOVERY: &str = "discovery";
    pub const RECALIBRATION: &str = "recalibration";
}

pub fn insert(conn: &Connection, kind: &str, body: &str, now: Timestamp) -> Result<()> {
    conn.execute(
        "INSERT INTO notices (kind, body, created_at) VALUES (?1, ?2, ?3)",
        params![kind, body, stamp(now)],
    )?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct Notice {
    pub id: i64,
    pub kind: String,
    pub body: String,
    pub read: bool,
    pub created_at: String,
}

/// The most recent notices, newest first.
pub fn recent(conn: &Connection, limit: usize) -> Result<Vec<Notice>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, body, read_at IS NOT NULL, created_at FROM notices
         ORDER BY created_at DESC, id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit as i64], |r| {
        Ok(Notice {
            id: r.get(0)?,
            kind: r.get(1)?,
            body: r.get(2)?,
            read: r.get(3)?,
            created_at: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn unread_count(conn: &Connection) -> Result<usize> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM notices WHERE read_at IS NULL",
        [],
        |r| r.get(0),
    )?;
    Ok(n as usize)
}

pub fn mark_all_read(conn: &Connection, now: Timestamp) -> Result<()> {
    conn.execute(
        "UPDATE notices SET read_at = ?1 WHERE read_at IS NULL",
        [stamp(now)],
    )?;
    Ok(())
}

#[cfg(test)]
pub fn bodies(conn: &Connection, kind: &str) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT body FROM notices WHERE kind = ?1 ORDER BY id")
        .unwrap();
    stmt.query_map([kind], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn unread_until_marked() {
        let conn = db::open_in_memory().unwrap();
        let now: Timestamp = "2026-10-16T12:00:00Z".parse().unwrap();
        insert(
            &conn,
            kind::DISCOVERY,
            "New dimension added: Laugh quality.",
            now,
        )
        .unwrap();
        insert(&conn, kind::THRESHOLD, "Enough Threshold revised.", now).unwrap();
        assert_eq!(unread_count(&conn).unwrap(), 2);
        let listed = recent(&conn, 10).unwrap();
        assert_eq!(listed[0].kind, kind::THRESHOLD);
        assert!(!listed[0].read);
        mark_all_read(&conn, now).unwrap();
        assert_eq!(unread_count(&conn).unwrap(), 0);
        assert!(recent(&conn, 10).unwrap().iter().all(|n| n.read));
    }
}
