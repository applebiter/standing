//! Creates a demonstration profile: about two months of use, replayed day by
//! day through the same methodology the application applies.
//!
//! ```sh
//! cargo run --release --example demo -- demo
//! STANDING_HOME=demo cargo run --release
//! ```
//!
//! Dates count back from the day the program is run. Every figure in the
//! resulting profile is computed, not entered.

use std::path::PathBuf;

use anyhow::{Context, bail};
use jiff::civil::{Date, DateTime};
use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rusqlite::Connection;

use standing::db::{self, threshold::Reason};
use standing::engine::run;
use standing::model::comparable::{ComparableFields, Origin};
use standing::model::event::EventKind;
use standing::model::rating::Subject;
use standing::{catalog, engine};

const DAYS: i64 = 60;
const SEED: i64 = 20_261_016;

/// Industries that do not apply to the demonstration profile.
const NOT_APPLICABLE: [&str; 2] = ["Children", "Partners"];

struct Demo {
    conn: Connection,
    tz: TimeZone,
    rng: ChaCha8Rng,
    now: Timestamp,
}

impl Demo {
    fn at(&self, day: Date, hour: i8, minute: i8) -> Timestamp {
        DateTime::from_parts(day, jiff::civil::time(hour, minute, 0, 0))
            .to_zoned(self.tz.clone())
            .expect("a local time")
            .timestamp()
    }

    fn open(&mut self, t: Timestamp) -> anyhow::Result<()> {
        run::market_check(&self.conn, t, &self.tz)?;
        self.recompute(t)
    }

    fn recompute(&mut self, t: Timestamp) -> anyhow::Result<()> {
        run::recompute(&mut self.conn, t, &self.tz)?;
        Ok(())
    }

    fn rate(
        &mut self,
        subject: Subject,
        dimension: &str,
        value: f64,
        t: Timestamp,
    ) -> anyhow::Result<()> {
        let value = (value * 2.0).round() / 2.0;
        run::record_rating(
            &mut self.conn,
            subject,
            dimension,
            value.clamp(0.0, 10.0),
            t,
        )?;
        Ok(())
    }

    fn file(
        &mut self,
        kind: EventKind,
        comparable: Option<i64>,
        note: Option<&str>,
        t: Timestamp,
    ) -> anyhow::Result<()> {
        db::events::insert(&self.conn, kind, comparable, note, t)?;
        Ok(())
    }

    fn applicable(&self) -> anyhow::Result<Vec<String>> {
        Ok(db::dimensions::visible(&self.conn)?
            .into_iter()
            .filter(|d| d.is_active())
            .map(|d| d.id)
            .collect())
    }

    fn psi(&self, t: Timestamp) -> anyhow::Result<f64> {
        let inputs = run::inputs(&self.conn)?;
        Ok(engine::index::compute(&inputs, t).psi)
    }
}

fn main() -> anyhow::Result<()> {
    let dir: PathBuf = std::env::args()
        .nth(1)
        .context("usage: demo DIRECTORY")?
        .into();
    let path = dir.join("standing.sqlite3");
    if path.exists() {
        bail!("{} already exists; remove it first", path.display());
    }
    let mut conn = db::open(&path)?;
    catalog::sync(&mut conn, &catalog::seed()?)?;

    let tz = TimeZone::system();
    let now = Timestamp::now();
    let today = engine::market::day_of(now, &tz);
    let first = today.saturating_sub((DAYS - 1).days());
    let mut demo = Demo {
        conn,
        tz,
        rng: ChaCha8Rng::seed_from_u64(SEED as u64),
        now,
    };

    // Day one: onboarding.
    let t = demo.at(first, 8, 41);
    db::user::ensure(&demo.conn, SEED, t)?;
    db::user::update(&demo.conn, "Alex Morgan", Some("accounting"))?;
    for d in db::dimensions::visible(&demo.conn)? {
        if NOT_APPLICABLE.contains(&d.industry.as_str()) {
            db::dimensions::set_applies(&demo.conn, &d.id, false)?;
        }
    }
    let mut peers = Vec::new();
    for (name, relationship) in [
        ("Dana", "college roommate"),
        ("Marcus Bell", "brother-in-law"),
        ("Priya", "former colleague"),
    ] {
        let fields = ComparableFields {
            name: name.into(),
            relationship: Some(relationship.into()),
        };
        peers.push(db::comparables::insert(&demo.conn, &fields, t)?.id);
    }
    db::threshold::set(&demo.conn, 95.0, Reason::Initial, demo.at(first, 8, 52))?;
    db::user::complete_onboarding(&demo.conn)?;
    demo.open(demo.at(first, 8, 53))?;

    // The first ratings: the user, and each Comparable on most of the same
    // dimensions, somewhat ahead.
    let dims = demo.applicable()?;
    let rated: Vec<String> = dims.iter().take(110).cloned().collect();
    let mut own: std::collections::HashMap<String, f64> = Default::default();
    for (i, d) in rated.iter().enumerate() {
        let t = demo.at(first, 9, 0) + (i as i64 * 23).seconds();
        let v = demo.rng.random_range(3.0..8.5);
        own.insert(d.clone(), v);
        demo.rate(Subject::Own, d, v, t)?;
    }
    for (k, &peer) in peers.iter().enumerate() {
        let lean = [0.4, 0.7, 0.1][k];
        for (i, d) in rated.iter().enumerate() {
            if demo.rng.random_bool(0.75) {
                let t = demo.at(first, 20, 0) + ((k * 200 + i) as i64 * 11).seconds();
                let v = own[d] + lean + demo.rng.random_range(-1.5..1.5);
                demo.rate(Subject::Comparable(peer), d, v, t)?;
            }
        }
    }
    demo.recompute(demo.at(first, 21, 30))?;

    let notes = [
        "At the barbecue",
        "Group chat",
        "Seated at the end of the table",
        "Mentioned twice",
        "In front of the team",
        "Read at 14:02",
    ];
    for offset in 1..DAYS {
        let day = first.saturating_add(offset.days());
        let minute = demo.rng.random_range(5..55);
        let opening = demo.at(day, 7, minute);
        if opening > demo.now {
            break;
        }
        if offset == DAYS - 1 {
            // Notices before today have been read.
            db::notices::mark_all_read(&demo.conn, opening)?;
        }
        demo.open(opening)?;

        // Ordinary filings.
        for _ in 0..demo.rng.random_range(0..4) {
            let (hour, minute) = (demo.rng.random_range(9..21), demo.rng.random_range(0..60));
            let t = demo.at(day, hour, minute);
            if t > demo.now {
                continue;
            }
            let kind = match demo.rng.random_range(0..100) {
                0..22 => EventKind::ComplimentReceived,
                22..34 => EventKind::TextUnanswered,
                34..46 => EventKind::SlightMinor,
                46..54 => EventKind::InvitationReceived,
                54..62 => EventKind::NotInvited,
                62..66 => EventKind::SlightMajor,
                66..83 => EventKind::ComparableThriving,
                _ => EventKind::ComparableAchievement,
            };
            let comparable = kind
                .concerns_comparable()
                .then(|| peers[demo.rng.random_range(0..peers.len())]);
            let note = demo
                .rng
                .random_bool(0.3)
                .then(|| notes[demo.rng.random_range(0..notes.len())]);
            demo.file(kind, comparable, note, t)?;
            demo.recompute(t)?;
        }

        // Dwelling on a few dimensions now and then.
        if offset % 6 == 2 {
            let focus = rated[(offset as usize * 7) % 12].clone();
            for k in 0..5 {
                let t = demo.at(day, 22, 10 + k);
                if t > demo.now {
                    break;
                }
                let v = own[&focus] + demo.rng.random_range(-1.0..1.0);
                demo.rate(Subject::Own, &focus, v, t)?;
            }
            demo.recompute(demo.at(day, 22, 20).min(demo.now))?;
        }

        // A good week, and a reassessment of oneself at the end of it.
        if (24..=28).contains(&offset) {
            let t = demo.at(day, 12, 15);
            if t <= demo.now {
                let kind = [
                    EventKind::Achievement,
                    EventKind::PublicRecognition,
                    EventKind::InvitationReceived,
                ][offset as usize % 3];
                demo.file(kind, None, None, t)?;
                demo.recompute(t)?;
            }
        }
        if offset == 28 {
            for (i, d) in rated.iter().enumerate() {
                let t = demo.at(day, 19, 0) + (i as i64 * 17).seconds();
                if t > demo.now {
                    break;
                }
                let v = (own[d] + 4.0).min(10.0);
                own.insert(d.clone(), v);
                demo.rate(Subject::Own, d, v, t)?;
            }
            demo.recompute(demo.at(day, 20, 0).min(demo.now))?;
        }

        // The newcomer is identified.
        if offset == 36 {
            let t = demo.at(day, 18, 30);
            if t <= demo.now {
                let introduced = db::comparables::all(&demo.conn)?
                    .into_iter()
                    .find(|c| c.origin == Origin::Archetype && c.claimed_at.is_none());
                if let Some(c) = introduced {
                    let fields = ComparableFields {
                        name: "Gordon Pratt".into(),
                        relationship: Some(standing::model::comparable::in_sentence(&c.name)),
                    };
                    db::comparables::claim(&mut demo.conn, c.id, &fields, t)?;
                    demo.recompute(t)?;
                }
            }
        }

        // The user revises the threshold to something attainable.
        if offset == 44 {
            let t = demo.at(day, 21, 5);
            if t <= demo.now {
                let target = (demo.psi(t)?.floor() - 2.0).max(1.0);
                db::threshold::set(&demo.conn, target, Reason::User, t)?;
                demo.recompute(t)?;
            }
        }
    }

    let profile = db::user::get(&demo.conn)?.context("profile")?;
    let comparables = db::comparables::all(&demo.conn)?;
    let history = db::threshold::history(&demo.conn)?;
    println!(
        "Demonstration profile for {} written to {} ({} days, {} Comparables, threshold {}).",
        profile.display_name,
        path.display(),
        DAYS,
        comparables.len(),
        history.first().map_or(0.0, |e| e.new)
    );
    Ok(())
}
