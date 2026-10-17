//! The Enough Threshold. The latest `threshold_history` row is current.

use rusqlite::{Connection, OptionalExtension};

use super::Result;

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
                    (110, 132, 'ratchet', '2026-10-16T10:00:00.000000Z');",
        )
        .unwrap();
        assert_eq!(current(&conn).unwrap(), Some(132.0));
    }
}
