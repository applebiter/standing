//! Queries over the `comparables` table.

use jiff::Timestamp;
use rusqlite::{Connection, Row, params};

use super::{Result, stamp};
use crate::model::comparable::{Comparable, ComparableFields, Origin, issue_ticker};

const COLUMNS: &str = "id, name, relationship, ticker, origin, archetype_template, claimed_at, \
                       active, created_at";

fn from_row(r: &Row<'_>) -> rusqlite::Result<Comparable> {
    let origin: String = r.get(4)?;
    Ok(Comparable {
        id: r.get(0)?,
        name: r.get(1)?,
        relationship: r.get(2)?,
        ticker: r.get(3)?,
        origin: Origin::parse(&origin).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                format!("unknown comparable origin {origin:?}").into(),
            )
        })?,
        archetype_template: r.get(5)?,
        claimed_at: r.get(6)?,
        active: r.get(7)?,
        created_at: r.get(8)?,
    })
}

/// Every Comparable, active first, then in order of entry.
pub fn all(conn: &Connection) -> Result<Vec<Comparable>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM comparables ORDER BY active DESC, id"
    ))?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// The current peer set, in order of entry.
pub fn active(conn: &Connection) -> Result<Vec<Comparable>> {
    Ok(all(conn)?.into_iter().filter(|c| c.active).collect())
}

/// Adds a Comparable entered by the user, issuing it a ticker.
pub fn insert(conn: &Connection, fields: &ComparableFields, now: Timestamp) -> Result<Comparable> {
    let tickers = tickers(conn)?;
    let ticker = issue_ticker(&fields.name, tickers.iter().map(String::as_str));
    conn.execute(
        "INSERT INTO comparables (name, relationship, ticker, origin, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            fields.name,
            fields.relationship,
            ticker,
            Origin::User.as_str(),
            stamp(now)
        ],
    )?;
    let id = conn.last_insert_rowid();
    Ok(conn.query_row(
        &format!("SELECT {COLUMNS} FROM comparables WHERE id = ?1"),
        [id],
        from_row,
    )?)
}

/// Adds a Comparable introduced from the user's extended social graph.
pub fn insert_archetype(
    conn: &Connection,
    name: &str,
    template: &str,
    now: Timestamp,
) -> Result<Comparable> {
    let tickers = tickers(conn)?;
    let ticker = issue_ticker(name, tickers.iter().map(String::as_str));
    conn.execute(
        "INSERT INTO comparables (name, ticker, origin, archetype_template, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            name,
            ticker,
            Origin::Archetype.as_str(),
            template,
            stamp(now)
        ],
    )?;
    let id = conn.last_insert_rowid();
    Ok(conn.query_row(
        &format!("SELECT {COLUMNS} FROM comparables WHERE id = ?1"),
        [id],
        from_row,
    )?)
}

/// Identifies an introduced Comparable: amends name and relationship, records
/// the claim, and confirms the ratings recorded for them so far.
pub fn claim(
    conn: &mut Connection,
    id: i64,
    fields: &ComparableFields,
    now: Timestamp,
) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE comparables SET name = ?2, relationship = ?3, claimed_at = ?4 WHERE id = ?1",
        params![id, fields.name, fields.relationship, stamp(now)],
    )?;
    tx.execute(
        "UPDATE ratings SET provisional = 0 WHERE comparable_id = ?1",
        [id],
    )?;
    tx.commit()?;
    Ok(())
}

/// Amends name and relationship. The ticker is unchanged.
pub fn update(conn: &Connection, id: i64, fields: &ComparableFields) -> Result<()> {
    conn.execute(
        "UPDATE comparables SET name = ?2, relationship = ?3 WHERE id = ?1",
        params![id, fields.name, fields.relationship],
    )?;
    Ok(())
}

/// Removes a Comparable from, or restores it to, the peer set.
pub fn set_active(conn: &Connection, id: i64, active: bool) -> Result<()> {
    conn.execute(
        "UPDATE comparables SET active = ?2 WHERE id = ?1",
        params![id, active],
    )?;
    Ok(())
}

/// Permanently deletes a Comparable and everything recorded about them.
pub fn delete(conn: &mut Connection, id: i64) -> Result<()> {
    let tx = conn.transaction()?;
    for table in ["ratings", "events", "index_snapshots"] {
        tx.execute(
            &format!("DELETE FROM {table} WHERE comparable_id = ?1"),
            [id],
        )?;
    }
    tx.execute("DELETE FROM comparables WHERE id = ?1", [id])?;
    tx.commit()?;
    Ok(())
}

fn tickers(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT ticker FROM comparables")?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn now() -> Timestamp {
        "2026-10-16T09:00:00Z".parse().unwrap()
    }

    fn fields(name: &str) -> ComparableFields {
        ComparableFields {
            name: name.into(),
            relationship: Some("college roommate".into()),
        }
    }

    #[test]
    fn insert_issues_unique_tickers() {
        let conn = db::open_in_memory().unwrap();
        let a = insert(&conn, &fields("Dana"), now()).unwrap();
        let b = insert(&conn, &fields("Dana"), now()).unwrap();
        assert_eq!((a.ticker.as_str(), b.ticker.as_str()), ("DANA", "DANA2"));
        assert_eq!(a.origin, Origin::User);
        assert!(a.active);
    }

    #[test]
    fn rename_keeps_ticker_and_deactivation_keeps_row() {
        let conn = db::open_in_memory().unwrap();
        let a = insert(&conn, &fields("Dana"), now()).unwrap();
        update(&conn, a.id, &fields("Dana R.")).unwrap();
        set_active(&conn, a.id, false).unwrap();
        let all = all(&conn).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Dana R.");
        assert_eq!(all[0].ticker, "DANA");
        assert!(active(&conn).unwrap().is_empty());
    }

    #[test]
    fn claiming_confirms_provisional_ratings() {
        let mut conn = db::open_in_memory().unwrap();
        crate::catalog::sync(
            &mut conn,
            &crate::catalog::parse(
                "id,sector,industry,name,definition,discovery\n\
                 MAT-014,Material Holdings,Vehicles,Boat,Watercraft.,0\n",
            )
            .unwrap(),
        )
        .unwrap();
        let a = insert_archetype(&conn, "Your landlord", "landlord", now()).unwrap();
        assert_eq!(a.origin, Origin::Archetype);
        assert_eq!(a.archetype_template.as_deref(), Some("landlord"));
        assert_eq!(a.ticker, "YL");
        db::ratings::insert_provisional(&conn, a.id, "MAT-014", 8.5, now()).unwrap();
        let provisional = |conn: &Connection| -> bool {
            db::ratings::current(conn)
                .unwrap()
                .values()
                .next()
                .unwrap()
                .provisional
        };
        assert!(provisional(&conn));

        claim(&mut conn, a.id, &fields("Gordon"), now()).unwrap();
        let c = &all(&conn).unwrap()[0];
        assert_eq!((c.name.as_str(), c.ticker.as_str()), ("Gordon", "YL"));
        assert!(c.claimed_at.is_some());
        assert!(!provisional(&conn));
    }

    #[test]
    fn delete_removes_everything_recorded() {
        let mut conn = db::open_in_memory().unwrap();
        crate::catalog::sync(
            &mut conn,
            &crate::catalog::parse(
                "id,sector,industry,name,definition,discovery\n\
                 MAT-014,Material Holdings,Vehicles,Boat,Watercraft.,0\n",
            )
            .unwrap(),
        )
        .unwrap();
        let a = insert(&conn, &fields("Dana"), now()).unwrap();
        let b = insert(&conn, &fields("Sam"), now()).unwrap();
        for id in [a.id, b.id] {
            crate::db::ratings::insert(
                &conn,
                crate::model::rating::Subject::Comparable(id),
                "MAT-014",
                7.0,
                now(),
            )
            .unwrap();
        }
        delete(&mut conn, a.id).unwrap();
        let count = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap() };
        assert_eq!(count("SELECT COUNT(*) FROM comparables"), 1);
        assert_eq!(count("SELECT COUNT(*) FROM ratings"), 1);
    }
}
