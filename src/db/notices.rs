//! Notices: rebalancing, discovery, threshold adjustments, recalibration and
//! market closes.

use jiff::Timestamp;
use rusqlite::{Connection, params};

use super::{Result, stamp};

/// Kinds of notice. Stored as their string value.
pub mod kind {
    pub const MARKET_CLOSE: &str = "market_close";
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
    pub body: String,
    pub created_at: String,
}

/// The most recent notices, newest first.
pub fn recent(conn: &Connection, limit: usize) -> Result<Vec<Notice>> {
    let mut stmt = conn.prepare(
        "SELECT body, created_at FROM notices ORDER BY created_at DESC, id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit as i64], |r| {
        Ok(Notice {
            body: r.get(0)?,
            created_at: r.get(1)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
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
