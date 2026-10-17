//! Dimension weights and the recalibration log.

use std::collections::HashMap;

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use super::{Result, stamp};
use crate::engine::weights::Revision;

/// Weights of dimensions that apply and are not hidden, in catalog order.
pub fn applicable(conn: &Connection) -> Result<Vec<(String, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT id, weight FROM dimensions WHERE applies = 1 AND state <> 'hidden'
         ORDER BY ordinal, id",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn set(conn: &Connection, id: &str, weight: f64) -> Result<()> {
    conn.execute(
        "UPDATE dimensions SET weight = ?2 WHERE id = ?1",
        params![id, weight],
    )?;
    Ok(())
}

pub fn add(conn: &Connection, id: &str, delta: f64) -> Result<()> {
    conn.execute(
        "UPDATE dimensions SET weight = weight + ?2 WHERE id = ?1",
        params![id, delta],
    )?;
    Ok(())
}

/// One recalibration: every applicable weight as it then stood, and those
/// revised by more than the threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recalibration {
    pub weights: HashMap<String, f64>,
    pub revised: Vec<RevisedWeight>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevisedWeight {
    pub id: String,
    pub from: f64,
    pub to: f64,
}

impl From<&Revision> for RevisedWeight {
    fn from(r: &Revision) -> Self {
        Self {
            id: r.dimension_id.clone(),
            from: r.from,
            to: r.to,
        }
    }
}

pub fn log(conn: &Connection, entry: &Recalibration, now: Timestamp) -> Result<()> {
    let summary = serde_json::to_string(entry).expect("weights serialize");
    conn.execute(
        "INSERT INTO weight_log (recalibrated_at, summary) VALUES (?1, ?2)",
        params![stamp(now), summary],
    )?;
    Ok(())
}

/// The most recent recalibration and when it occurred.
pub fn last(conn: &Connection) -> Result<Option<(String, Recalibration)>> {
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT recalibrated_at, summary FROM weight_log
             ORDER BY recalibrated_at DESC, id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    Ok(row.and_then(|(at, s)| serde_json::from_str(&s).ok().map(|r| (at, r))))
}
