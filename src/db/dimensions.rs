//! Queries over the `dimensions` table.

use rusqlite::{Connection, Row, params};

use super::Result;
use crate::model::dimension::{Dimension, DimensionState};

const COLUMNS: &str =
    "id, sector, industry, name, definition, applies, state, weight, discovered_at";

fn from_row(r: &Row<'_>) -> rusqlite::Result<Dimension> {
    let state: String = r.get(6)?;
    Ok(Dimension {
        id: r.get(0)?,
        sector: r.get(1)?,
        industry: r.get(2)?,
        name: r.get(3)?,
        definition: r.get(4)?,
        applies: r.get(5)?,
        state: DimensionState::parse(&state).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Text,
                format!("unknown dimension state {state:?}").into(),
            )
        })?,
        weight: r.get(7)?,
        discovered_at: r.get(8)?,
    })
}

/// Every dimension not held back for discovery, in catalog order.
pub fn visible(conn: &Connection) -> Result<Vec<Dimension>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM dimensions WHERE state <> 'hidden' ORDER BY ordinal, id"
    ))?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Sets the user's "Applies to me" toggle.
pub fn set_applies(conn: &Connection, id: &str, applies: bool) -> Result<()> {
    conn.execute(
        "UPDATE dimensions SET applies = ?2 WHERE id = ?1",
        params![id, applies],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{catalog, db};

    fn seeded() -> Connection {
        let mut conn = db::open_in_memory().unwrap();
        let entries = catalog::parse(
            "id,sector,industry,name,definition,discovery\n\
             PHY-001,Physical Assets,Bearing,Posture,Stance.,0\n\
             PHY-002,Physical Assets,Bearing,Gait,Walk.,1\n\
             PHY-003,Physical Assets,Bearing,Stride,Step.,0\n",
        )
        .unwrap();
        catalog::sync(&mut conn, &entries).unwrap();
        conn
    }

    #[test]
    fn hidden_dimensions_are_not_visible() {
        let conn = seeded();
        let ids: Vec<String> = visible(&conn).unwrap().into_iter().map(|d| d.id).collect();
        assert_eq!(ids, ["PHY-001", "PHY-003"]);
    }

    #[test]
    fn visible_follows_catalog_order() {
        let mut conn = db::open_in_memory().unwrap();
        let entries = catalog::parse(
            "id,sector,industry,name,definition,discovery\n\
             MAT-001,Material Holdings,Vehicles,Boat,Watercraft.,0\n\
             CAR-001,Career and Credentials,Position,Job title,Title.,0\n",
        )
        .unwrap();
        catalog::sync(&mut conn, &entries).unwrap();
        let ids: Vec<String> = visible(&conn).unwrap().into_iter().map(|d| d.id).collect();
        assert_eq!(ids, ["MAT-001", "CAR-001"]);
    }

    #[test]
    fn excluded_dimensions_are_visible_but_inactive() {
        let conn = seeded();
        set_applies(&conn, "PHY-003", false).unwrap();
        let dims = visible(&conn).unwrap();
        assert_eq!(dims.len(), 2);
        assert!(!dims[1].applies);
        assert!(!dims[1].is_active());
        assert!(dims[0].is_active());
    }
}
