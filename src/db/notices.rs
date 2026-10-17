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
