//! The Enough Threshold. The latest `threshold_history` row is current.

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};

use super::{Result, stamp};

/// Why the threshold changed. Stored as its string value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// First set, during onboarding or on the Threshold screen.
    Initial,
    /// Set by the user.
    User,
    /// Adjusted to reflect current market conditions.
    Adjustment,
}

impl Reason {
    fn as_str(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::User => "user",
            Self::Adjustment => "adjustment",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "initial" => Some(Self::Initial),
            "user" => Some(Self::User),
            "adjustment" => Some(Self::Adjustment),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub prior: Option<f64>,
    pub new: f64,
    pub reason: Reason,
    pub changed_at: String,
}

/// The current Enough Threshold, if one has been set.
pub fn current(conn: &Connection) -> Result<Option<f64>> {
    Ok(conn
        .query_row(
            "SELECT new_value FROM threshold_history ORDER BY changed_at DESC, id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?)
}

/// Records a new threshold, with the current one as its prior value.
pub fn set(conn: &Connection, value: f64, reason: Reason, now: Timestamp) -> Result<()> {
    let prior = current(conn)?;
    conn.execute(
        "INSERT INTO threshold_history (prior_value, new_value, reason, changed_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![prior, value, reason.as_str(), stamp(now)],
    )?;
    Ok(())
}

/// Every change, newest first.
pub fn history(conn: &Connection) -> Result<Vec<Entry>> {
    let mut stmt = conn.prepare(
        "SELECT prior_value, new_value, reason, changed_at FROM threshold_history
         ORDER BY changed_at DESC, id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        let reason: String = r.get(2)?;
        Ok(Entry {
            prior: r.get(0)?,
            new: r.get(1)?,
            reason: Reason::parse(&reason).ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    format!("unknown threshold reason {reason:?}").into(),
                )
            })?,
            changed_at: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn latest_entry_is_current() {
        let conn = db::open_in_memory().unwrap();
        assert_eq!(current(&conn).unwrap(), None);
        conn.execute_batch(
            "INSERT INTO threshold_history (prior_value, new_value, reason, changed_at)
             VALUES (NULL, 110, 'initial', '2026-10-15T10:00:00.000000Z'),
                    (110, 132, 'adjustment', '2026-10-16T10:00:00.000000Z');",
        )
        .unwrap();
        assert_eq!(current(&conn).unwrap(), Some(132.0));
    }

    #[test]
    fn set_records_the_prior_value() {
        let conn = db::open_in_memory().unwrap();
        let t = |s: &str| -> Timestamp { s.parse().unwrap() };
        set(&conn, 110.0, Reason::Initial, t("2026-10-15T10:00:00Z")).unwrap();
        set(&conn, 90.0, Reason::User, t("2026-10-16T10:00:00Z")).unwrap();
        let h = history(&conn).unwrap();
        assert_eq!(h.len(), 2);
        assert_eq!(
            (h[0].prior, h[0].new, h[0].reason),
            (Some(110.0), 90.0, Reason::User)
        );
        assert_eq!((h[1].prior, h[1].reason), (None, Reason::Initial));
    }

    #[test]
    fn thresholds_below_one_are_refused() {
        let conn = db::open_in_memory().unwrap();
        let now: Timestamp = "2026-10-16T10:00:00Z".parse().unwrap();
        assert!(set(&conn, 0.5, Reason::User, now).is_err());
    }
}
