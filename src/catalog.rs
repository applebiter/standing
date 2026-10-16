//! The SCCD seed catalog: loading, validation and synchronisation into the
//! database.

use std::collections::{HashMap, HashSet};

use rusqlite::{Connection, params};
use serde::Deserialize;

use crate::model::dimension::DimensionState;

/// The seed catalog shipped with this build.
const SEED: &str = include_str!("../data/dimensions.csv");

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Entry {
    pub id: String,
    pub sector: String,
    pub industry: String,
    pub name: String,
    pub definition: String,
    /// `1` if the entry is held back for Dimension Discovery.
    #[serde(deserialize_with = "flag")]
    pub discovery: bool,
}

fn flag<'de, D: serde::Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    match u8::deserialize(d)? {
        0 => Ok(false),
        1 => Ok(true),
        n => Err(serde::de::Error::custom(format!(
            "expected 0 or 1, found {n}"
        ))),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("catalog row {row}: {source}")]
    Parse { row: usize, source: csv::Error },
    #[error("catalog entry {id}: {problem}")]
    Invalid { id: String, problem: &'static str },
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

/// Parses and validates the seed catalog shipped with this build.
pub fn seed() -> Result<Vec<Entry>, CatalogError> {
    parse(SEED)
}

/// Parses and validates a catalog in the seed's CSV format.
///
/// Identifiers have the form `ABC-123` and are unique; the three-letter
/// prefix is consistent within a sector; every text field is non-empty.
pub fn parse(text: &str) -> Result<Vec<Entry>, CatalogError> {
    let mut reader = csv::Reader::from_reader(text.as_bytes());
    let mut entries: Vec<Entry> = Vec::new();
    let mut ids = HashSet::new();
    let mut prefixes = HashMap::new();
    for (i, record) in reader.deserialize().enumerate() {
        let entry: Entry = record.map_err(|source| CatalogError::Parse { row: i + 2, source })?;
        let invalid = |problem| CatalogError::Invalid {
            id: entry.id.clone(),
            problem,
        };
        let Some((prefix, number)) = entry.id.split_once('-') else {
            return Err(invalid("identifier is not of the form ABC-123"));
        };
        if prefix.len() != 3
            || !prefix.bytes().all(|b| b.is_ascii_uppercase())
            || number.len() != 3
            || !number.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(invalid("identifier is not of the form ABC-123"));
        }
        if [
            &entry.sector,
            &entry.industry,
            &entry.name,
            &entry.definition,
        ]
        .iter()
        .any(|f| f.trim().is_empty())
        {
            return Err(invalid("a required field is empty"));
        }
        if !ids.insert(entry.id.clone()) {
            return Err(invalid("identifier is listed more than once"));
        }
        if *prefixes
            .entry(entry.sector.clone())
            .or_insert_with(|| prefix.to_owned())
            != prefix
        {
            return Err(invalid("identifier prefix differs from its sector's"));
        }
        entries.push(entry);
    }
    Ok(entries)
}

/// Brings the `dimensions` table into line with `entries`.
///
/// New identifiers are inserted: hidden if flagged for discovery, otherwise
/// active. For existing identifiers only the catalog position and descriptive
/// text are refreshed; the user's `applies`, `state` and `weight` are never
/// changed. Rows absent from the catalog are left in place, so identifiers are
/// never reused.
/// Returns the number of dimensions inserted.
pub fn sync(conn: &mut Connection, entries: &[Entry]) -> Result<usize, CatalogError> {
    let tx = conn.transaction()?;
    let mut inserted = 0;
    {
        let mut insert = tx.prepare(
            "INSERT INTO dimensions (id, ordinal, sector, industry, name, definition, state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT (id) DO NOTHING",
        )?;
        let mut refresh = tx.prepare(
            "UPDATE dimensions
             SET ordinal = ?2, sector = ?3, industry = ?4, name = ?5, definition = ?6
             WHERE id = ?1
               AND (ordinal, sector, industry, name, definition) IS NOT (?2, ?3, ?4, ?5, ?6)",
        )?;
        for (ordinal, e) in (0_i64..).zip(entries) {
            let state = if e.discovery {
                DimensionState::Hidden
            } else {
                DimensionState::Active
            };
            let n = insert.execute(params![
                e.id,
                ordinal,
                e.sector,
                e.industry,
                e.name,
                e.definition,
                state.as_str()
            ])?;
            if n == 0 {
                refresh.execute(params![
                    e.id,
                    ordinal,
                    e.sector,
                    e.industry,
                    e.name,
                    e.definition
                ])?;
            }
            inserted += n;
        }
    }
    tx.commit()?;
    Ok(inserted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    const HEADER: &str = "id,sector,industry,name,definition,discovery\n";

    fn csv(rows: &[&str]) -> String {
        format!("{HEADER}{}\n", rows.join("\n"))
    }

    #[test]
    fn seed_catalog_is_valid() {
        let entries = seed().unwrap();
        assert!(entries.len() >= 500);
        assert!(entries.iter().any(|e| e.discovery));
        assert!(entries.iter().any(|e| !e.discovery));
    }

    #[test]
    fn rejects_malformed_entries() {
        let cases = [
            "PHY17,Physical Assets,Bearing,Posture,Stance.,0",
            "PHY-017,Physical Assets,Bearing,,Stance.,0",
            "PHY-017,Physical Assets,Bearing,Posture,Stance.,2",
        ];
        for row in cases {
            assert!(parse(&csv(&[row])).is_err(), "accepted {row}");
        }
        let duplicate = csv(&[
            "PHY-001,Physical Assets,Bearing,Posture,Stance.,0",
            "PHY-001,Physical Assets,Bearing,Gait,Walk.,0",
        ]);
        assert!(parse(&duplicate).is_err());
        let mixed_prefix = csv(&[
            "PHY-001,Physical Assets,Bearing,Posture,Stance.,0",
            "BOD-002,Physical Assets,Bearing,Gait,Walk.,0",
        ]);
        assert!(parse(&mixed_prefix).is_err());
    }

    #[test]
    fn sync_inserts_new_and_preserves_user_choices() {
        let mut conn = db::open_in_memory().unwrap();
        let v1 = parse(&csv(&[
            "PHY-001,Physical Assets,Bearing,Posture,Stance.,0",
            "PHY-002,Physical Assets,Bearing,Gait,Walk.,1",
        ]))
        .unwrap();
        assert_eq!(sync(&mut conn, &v1).unwrap(), 2);
        let state: String = conn
            .query_row(
                "SELECT state FROM dimensions WHERE id = 'PHY-002'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(state, "hidden");

        conn.execute(
            "UPDATE dimensions SET applies = 0, weight = 1.3 WHERE id = 'PHY-001'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE dimensions SET state = 'discovered' WHERE id = 'PHY-002'",
            [],
        )
        .unwrap();

        let v2 = parse(&csv(&[
            "PHY-001,Physical Assets,Bearing,Posture,Uprightness of stance.,1",
            "PHY-002,Physical Assets,Bearing,Gait,Walk.,1",
            "PHY-003,Physical Assets,Bearing,Stride,Length of step.,0",
        ]))
        .unwrap();
        assert_eq!(sync(&mut conn, &v2).unwrap(), 1);
        assert_eq!(sync(&mut conn, &v2).unwrap(), 0);

        let row = |id: &str| -> (bool, String, f64, String) {
            conn.query_row(
                "SELECT applies, state, weight, definition FROM dimensions WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap()
        };
        assert_eq!(
            row("PHY-001"),
            (false, "active".into(), 1.3, "Uprightness of stance.".into())
        );
        assert_eq!(row("PHY-002").1, "discovered");
        assert_eq!(row("PHY-003").1, "active");
    }

    #[test]
    fn sync_keeps_rows_dropped_from_the_catalog() {
        let mut conn = db::open_in_memory().unwrap();
        let v1 = parse(&csv(&["PHY-001,Physical Assets,Bearing,Posture,Stance.,0"])).unwrap();
        sync(&mut conn, &v1).unwrap();
        sync(&mut conn, &[]).unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM dimensions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }
}
