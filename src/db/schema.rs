use rusqlite_migration::{M, Migrations};

/// Append-only. Never edit a migration that has shipped; add a new one.
const MIGRATIONS_SLICE: &[M<'_>] = &[M::up(
    r#"
CREATE TABLE profile (
  id                  INTEGER PRIMARY KEY CHECK (id = 1),
  display_name        TEXT NOT NULL,
  field               TEXT,
  rng_seed            INTEGER NOT NULL,
  created_at          TEXT NOT NULL,
  onboarding_complete INTEGER NOT NULL DEFAULT 0
);

-- Seeded from the SCCD catalog. Rows are inserted for new identifiers only;
-- `applies`, `state` and `weight` belong to the user once a row exists.
-- `ordinal` is the entry's position in the catalog, for display order.
CREATE TABLE dimensions (
  id            TEXT PRIMARY KEY,
  ordinal       INTEGER NOT NULL,
  sector        TEXT NOT NULL,
  industry      TEXT NOT NULL,
  name          TEXT NOT NULL,
  definition    TEXT NOT NULL,
  applies       INTEGER NOT NULL DEFAULT 1,
  state         TEXT NOT NULL CHECK (state IN ('active', 'hidden', 'discovered')),
  weight        REAL NOT NULL DEFAULT 1.0 CHECK (weight > 0),
  discovered_at TEXT
) WITHOUT ROWID;

CREATE TABLE comparables (
  id                 INTEGER PRIMARY KEY AUTOINCREMENT,
  name               TEXT NOT NULL,
  relationship       TEXT,
  origin             TEXT NOT NULL CHECK (origin IN ('user', 'archetype')),
  archetype_template TEXT,
  claimed_at         TEXT,
  active             INTEGER NOT NULL DEFAULT 1,
  created_at         TEXT NOT NULL
);

-- Append-only. The latest row per subject and dimension is the current rating.
CREATE TABLE ratings (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  subject_kind  TEXT NOT NULL CHECK (subject_kind IN ('self', 'comparable')),
  comparable_id INTEGER REFERENCES comparables(id),
  dimension_id  TEXT NOT NULL REFERENCES dimensions(id),
  value         REAL NOT NULL CHECK (value BETWEEN 0 AND 10),
  provisional   INTEGER NOT NULL DEFAULT 0,
  rated_at      TEXT NOT NULL,
  CHECK ((subject_kind = 'self') = (comparable_id IS NULL))
);
CREATE INDEX idx_ratings_current
  ON ratings(subject_kind, comparable_id, dimension_id, rated_at);

CREATE TABLE events (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  kind          TEXT NOT NULL,
  comparable_id INTEGER REFERENCES comparables(id),
  note          TEXT,
  occurred_at   TEXT NOT NULL
);
CREATE INDEX idx_events_occurred_at ON events(occurred_at);

-- The latest row is the current threshold.
CREATE TABLE threshold_history (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  prior_value REAL,
  new_value   REAL NOT NULL CHECK (new_value >= 1),
  reason      TEXT NOT NULL CHECK (reason IN ('initial', 'user', 'ratchet')),
  changed_at  TEXT NOT NULL
);

CREATE TABLE weight_log (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  recalibrated_at TEXT NOT NULL,
  summary         TEXT NOT NULL
);

CREATE TABLE index_snapshots (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  subject_kind  TEXT NOT NULL CHECK (subject_kind IN ('self', 'comparable')),
  comparable_id INTEGER REFERENCES comparables(id),
  value         REAL NOT NULL,
  computed_at   TEXT NOT NULL
);
CREATE INDEX idx_snapshots_subject
  ON index_snapshots(subject_kind, comparable_id, computed_at);

CREATE TABLE notices (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  kind       TEXT NOT NULL,
  body       TEXT NOT NULL,
  created_at TEXT NOT NULL,
  read_at    TEXT
);
"#,
)];

pub const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_valid() {
        MIGRATIONS.validate().unwrap();
    }
}
