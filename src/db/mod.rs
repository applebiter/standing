//! Storage. Free of egui so it can be unit-tested in isolation.

pub mod dimensions;
pub mod schema;

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use rusqlite_migration::SchemaVersion;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    // The source is reported by the caller's error chain, not repeated here.
    #[error("could not create data directory {path}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not open database at {path}")]
    Open {
        path: PathBuf,
        source: rusqlite::Error,
    },
    #[error("database migration failed: {0}")]
    Migration(#[from] rusqlite_migration::Error),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, DbError>;

/// Opens (creating if necessary) the database at `path` and brings it to the
/// latest schema.
pub fn open(path: &Path) -> Result<Connection> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|source| DbError::CreateDir {
            path: dir.to_owned(),
            source,
        })?;
    }
    let mut conn = Connection::open(path).map_err(|source| DbError::Open {
        path: path.to_owned(),
        source,
    })?;
    prepare(&mut conn)?;
    Ok(conn)
}

/// Opens a fresh in-memory database with the current schema.
#[cfg(test)]
pub fn open_in_memory() -> Result<Connection> {
    let mut conn = Connection::open_in_memory()?;
    prepare(&mut conn)?;
    Ok(conn)
}

fn prepare(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    // A rollback journal, not WAL: the profile may live on a USB stick or in a
    // synced folder beside a portable copy of the app, where WAL's shared
    // memory file is unreliable. At rest the profile is a single file.
    conn.pragma_update(None, "journal_mode", "DELETE")?;
    schema::MIGRATIONS.to_latest(conn)?;
    Ok(())
}

pub fn schema_version(conn: &Connection) -> Result<usize> {
    Ok(match schema::MIGRATIONS.current_version(conn)? {
        SchemaVersion::Inside(v) | SchemaVersion::Outside(v) => v.get(),
        SchemaVersion::NoneSet => 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LATEST: usize = 1;

    #[test]
    fn fresh_database_migrates_to_latest() {
        let conn = open_in_memory().unwrap();
        assert_eq!(schema_version(&conn).unwrap(), LATEST);
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();
        for table in [
            "profile",
            "dimensions",
            "comparables",
            "ratings",
            "events",
            "threshold_history",
            "weight_log",
            "index_snapshots",
            "notices",
        ] {
            assert!(tables.contains(&table.to_owned()), "missing table {table}");
        }
    }

    #[test]
    fn on_disk_database_reopens_without_remigrating() {
        let dir = std::env::temp_dir().join(format!("standing-test-{}", std::process::id()));
        let path = dir.join("standing.sqlite3");
        drop(open(&path).unwrap());
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), LATEST);
        drop(conn);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
