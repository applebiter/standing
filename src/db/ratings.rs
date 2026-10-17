//! Queries over the append-only `ratings` table.

use std::collections::HashMap;

use jiff::Timestamp;
use rusqlite::{Connection, params};

use super::{Result, stamp};
use crate::model::rating::{MAX, MIN, Rating, Subject};

/// Records a rating entered by the user. Earlier ratings are kept as history.
pub fn insert(
    conn: &Connection,
    subject: Subject,
    dimension_id: &str,
    value: f64,
    now: Timestamp,
) -> Result<Rating> {
    let value = value.clamp(MIN, MAX);
    let rated_at = stamp(now);
    conn.execute(
        "INSERT INTO ratings (subject_kind, comparable_id, dimension_id, value, provisional, rated_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        params![
            subject.kind(),
            subject.comparable_id(),
            dimension_id,
            value,
            rated_at
        ],
    )?;
    Ok(Rating {
        subject,
        dimension_id: dimension_id.to_owned(),
        value,
        provisional: false,
        rated_at,
    })
}

/// Records a generated estimate for a Comparable.
pub fn insert_provisional(
    conn: &Connection,
    comparable_id: i64,
    dimension_id: &str,
    value: f64,
    now: Timestamp,
) -> Result<()> {
    conn.execute(
        "INSERT INTO ratings (subject_kind, comparable_id, dimension_id, value, provisional, rated_at)
         VALUES ('comparable', ?1, ?2, ?3, 1, ?4)",
        params![comparable_id, dimension_id, value.clamp(MIN, MAX), stamp(now)],
    )?;
    Ok(())
}

/// The current rating for every subject and dimension: the latest by time of
/// rating, then by order of entry.
pub fn current(conn: &Connection) -> Result<HashMap<(Subject, String), Rating>> {
    let mut stmt = conn.prepare(
        "SELECT subject_kind, comparable_id, dimension_id, value, provisional, rated_at
         FROM (
           SELECT *, ROW_NUMBER() OVER (
             PARTITION BY subject_kind, comparable_id, dimension_id
             ORDER BY rated_at DESC, id DESC
           ) AS n
           FROM ratings
         )
         WHERE n = 1",
    )?;
    let rows = stmt.query_map([], |r| {
        let kind: String = r.get(0)?;
        let comparable_id: Option<i64> = r.get(1)?;
        let subject = Subject::from_columns(&kind, comparable_id).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                format!("unknown rating subject {kind:?}").into(),
            )
        })?;
        Ok(Rating {
            subject,
            dimension_id: r.get(2)?,
            value: r.get(3)?,
            provisional: r.get(4)?,
            rated_at: r.get(5)?,
        })
    })?;
    let mut out = HashMap::new();
    for rating in rows {
        let rating = rating?;
        out.insert((rating.subject, rating.dimension_id.clone()), rating);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{catalog, db};

    fn at(s: &str) -> Timestamp {
        s.parse().unwrap()
    }

    fn setup() -> Connection {
        let mut conn = db::open_in_memory().unwrap();
        catalog::sync(
            &mut conn,
            &catalog::parse(
                "id,sector,industry,name,definition,discovery\n\
                 MAT-014,Material Holdings,Vehicles,Boat,Watercraft.,0\n\
                 DOM-004,Domestic Operations,Living Things,Sourdough starter age,Age.,0\n",
            )
            .unwrap(),
        )
        .unwrap();
        conn
    }

    #[test]
    fn latest_rating_is_current_and_history_is_kept() {
        let conn = setup();
        insert(
            &conn,
            Subject::Own,
            "MAT-014",
            4.0,
            at("2026-10-16T09:00:00Z"),
        )
        .unwrap();
        insert(
            &conn,
            Subject::Own,
            "MAT-014",
            6.5,
            at("2026-10-16T10:00:00Z"),
        )
        .unwrap();
        insert(
            &conn,
            Subject::Own,
            "DOM-004",
            2.0,
            at("2026-10-16T10:00:00Z"),
        )
        .unwrap();
        let current = current(&conn).unwrap();
        assert_eq!(current.len(), 2);
        assert_eq!(current[&(Subject::Own, "MAT-014".into())].value, 6.5);
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM ratings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 3);
    }

    #[test]
    fn ties_resolve_to_the_later_entry_and_subjects_are_separate() {
        let mut conn = setup();
        let dana = db::comparables::insert(
            &conn,
            &crate::model::comparable::ComparableFields {
                name: "Dana".into(),
                relationship: None,
            },
            at("2026-10-16T09:00:00Z"),
        )
        .unwrap();
        let t = at("2026-10-16T11:00:00Z");
        insert(&conn, Subject::Own, "MAT-014", 3.0, t).unwrap();
        insert(&conn, Subject::Own, "MAT-014", 3.5, t).unwrap();
        insert(&conn, Subject::Comparable(dana.id), "MAT-014", 12.0, t).unwrap();
        let current = current(&conn).unwrap();
        assert_eq!(current[&(Subject::Own, "MAT-014".into())].value, 3.5);
        assert_eq!(
            current[&(Subject::Comparable(dana.id), "MAT-014".into())].value,
            10.0
        );
        db::comparables::delete(&mut conn, dana.id).unwrap();
        assert_eq!(super::current(&conn).unwrap().len(), 1);
    }
}
