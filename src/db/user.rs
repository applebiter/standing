//! The single-row `profile` table.

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};

use super::{Result, stamp};
use crate::model::user::UserProfile;

pub fn get(conn: &Connection) -> Result<Option<UserProfile>> {
    Ok(conn
        .query_row(
            "SELECT display_name, field, rng_seed, created_at, onboarding_complete
             FROM profile WHERE id = 1",
            [],
            |r| {
                Ok(UserProfile {
                    display_name: r.get(0)?,
                    field: r.get(1)?,
                    rng_seed: r.get(2)?,
                    created_at: r.get(3)?,
                    onboarding_complete: r.get(4)?,
                })
            },
        )
        .optional()?)
}

/// Returns the profile, creating an empty one with `seed` if none exists.
pub fn ensure(conn: &Connection, seed: i64, now: Timestamp) -> Result<UserProfile> {
    conn.execute(
        "INSERT INTO profile (id, display_name, rng_seed, created_at)
         VALUES (1, '', ?1, ?2)
         ON CONFLICT (id) DO NOTHING",
        params![seed, stamp(now)],
    )?;
    Ok(get(conn)?.expect("profile row exists"))
}

pub fn update(conn: &Connection, display_name: &str, field: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE profile SET display_name = ?1, field = ?2 WHERE id = 1",
        params![display_name, field],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn ensure_creates_once_and_keeps_the_seed() {
        let conn = db::open_in_memory().unwrap();
        let now: Timestamp = "2026-10-16T09:00:00Z".parse().unwrap();
        assert!(get(&conn).unwrap().is_none());
        let first = ensure(&conn, 42, now).unwrap();
        assert_eq!(first.rng_seed, 42);
        assert_eq!(first.display_name, "");
        assert!(!first.onboarding_complete);

        update(&conn, "Alex", Some("Accounting")).unwrap();
        let again = ensure(&conn, 7, now).unwrap();
        assert_eq!(again.rng_seed, 42);
        assert_eq!(again.display_name, "Alex");
        assert_eq!(again.field.as_deref(), Some("Accounting"));
    }
}
