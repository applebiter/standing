//! The Quarterly Statement: a quarter's record, set out as tables.

use jiff::civil::Date;
use serde::Serialize;

use crate::db::threshold::Reason;
use crate::{copy, format};

/// A calendar quarter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quarter {
    pub year: i16,
    /// 1 to 4.
    pub q: i8,
}

impl Quarter {
    pub fn containing(d: Date) -> Self {
        Self {
            year: d.year(),
            q: (d.month() - 1) / 3 + 1,
        }
    }

    pub fn first_day(self) -> Date {
        jiff::civil::date(self.year, (self.q - 1) * 3 + 1, 1)
    }

    pub fn last_day(self) -> Date {
        self.next().first_day().yesterday().expect("date in range")
    }

    pub fn next(self) -> Self {
        if self.q == 4 {
            Self {
                year: self.year + 1,
                q: 1,
            }
        } else {
            Self {
                year: self.year,
                q: self.q + 1,
            }
        }
    }

    pub fn label(self) -> String {
        format!("Q{} {}", self.q, self.year)
    }
}

/// Opening, high, low and closing values of an index over the quarter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThresholdChange {
    pub date: Date,
    pub prior: Option<f64>,
    pub new: f64,
    pub reason: Reason,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Headwind {
    pub dimension: String,
    pub sector: String,
    /// The user's adjusted rating less the leading Comparable's.
    pub spread: f64,
    pub leader: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeerResult {
    pub ticker: String,
    pub name: String,
    pub range: Option<Range>,
}

/// Everything the statement is prepared from.
#[derive(Debug, Clone, PartialEq)]
pub struct QuarterData {
    pub quarter: Quarter,
    /// The last day covered: the quarter's end, or today if earlier.
    pub as_of: Date,
    pub name: String,
    pub psi: Option<Range>,
    pub threshold_at_end: Option<f64>,
    /// Changes in the quarter, oldest first.
    pub threshold_changes: Vec<ThresholdChange>,
    /// Weakest first.
    pub headwinds: Vec<Headwind>,
    pub peers: Vec<PeerResult>,
    /// Comparables introduced in the quarter, with the date and current name.
    pub introduced: Vec<(Date, String)>,
    /// Dimensions discovered in the quarter.
    pub discovered: Vec<(Date, String)>,
    pub positions_held: usize,
    /// (event label, count), in a fixed order.
    pub events: Vec<(String, usize)>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Column {
    pub label: String,
    /// Right-aligned.
    pub numeric: bool,
    /// Share of the table's width, from 0 to 1.
    pub width: f32,
}

/// One table of the statement.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Section {
    pub title: String,
    pub note: Option<String>,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<String>>,
    /// Shown in place of the table when there are no rows.
    pub empty: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Statement {
    pub title: String,
    pub holder: String,
    pub period: String,
    pub sections: Vec<Section>,
}

/// Columns as (label, numeric, share of width).
fn columns(spec: &[(&str, bool, f32)]) -> Vec<Column> {
    spec.iter()
        .map(|&(label, numeric, width)| Column {
            label: label.to_owned(),
            numeric,
            width,
        })
        .collect()
}

fn change(open: f64, close: f64) -> String {
    let change = close - open;
    let pct = if open.abs() > f64::EPSILON {
        100.0 * change / open.abs()
    } else {
        0.0
    };
    format!(
        "{} ({})",
        format::signed(change),
        format::signed_percent(pct)
    )
}

/// Prepares the statement.
pub fn prepare(d: &QuarterData) -> Statement {
    use copy::statement as c;
    let none = || copy::comparables::NO_VALUE.to_owned();

    let mut summary = Vec::new();
    match d.psi {
        Some(r) => {
            summary.push(vec![c::OPENING.to_owned(), format::index(r.open)]);
            summary.push(vec![c::CLOSING.to_owned(), format::index(r.close)]);
            summary.push(vec![c::CHANGE.to_owned(), change(r.open, r.close)]);
            summary.push(vec![
                c::RANGE.to_owned(),
                format!("{} to {}", format::index(r.low), format::index(r.high)),
            ]);
        }
        None => summary.push(vec![c::CLOSING.to_owned(), none()]),
    }
    summary.push(vec![
        c::THRESHOLD.to_owned(),
        d.threshold_at_end.map_or_else(none, format::index),
    ]);
    summary.push(vec![
        c::SHORTFALL.to_owned(),
        match (d.threshold_at_end, d.psi) {
            (Some(t), Some(r)) => format::index((t - r.close).max(0.0)),
            _ => none(),
        },
    ]);
    summary.push(vec![
        c::POSITIONS.to_owned(),
        format::thousands(d.positions_held as u64),
    ]);

    let sections = vec![
        Section {
            title: c::SUMMARY.to_owned(),
            note: None,
            columns: columns(&[(c::COL_MEASURE, false, 0.65), (c::COL_VALUE, true, 0.35)]),
            rows: summary,
            empty: String::new(),
        },
        Section {
            title: c::THRESHOLD_SECTION.to_owned(),
            note: None,
            columns: columns(&[
                (c::COL_DATE, false, 0.16),
                (c::COL_PRIOR, true, 0.12),
                (c::COL_NEW, true, 0.12),
                (c::COL_REASON, false, 0.60),
            ]),
            rows: d
                .threshold_changes
                .iter()
                .map(|t| {
                    vec![
                        format::date_short(t.date),
                        t.prior.map_or_else(none, format::index),
                        format::index(t.new),
                        copy::threshold::reason(t.reason).to_owned(),
                    ]
                })
                .collect(),
            empty: c::NO_THRESHOLD_CHANGES.to_owned(),
        },
        Section {
            title: c::HEADWINDS.to_owned(),
            note: Some(c::SPREAD_NOTE.to_owned()),
            columns: columns(&[
                (c::COL_DIMENSION, false, 0.42),
                (c::COL_SECTOR, false, 0.26),
                (c::COL_SPREAD, true, 0.12),
                (c::COL_LEADER, false, 0.20),
            ]),
            rows: d
                .headwinds
                .iter()
                .map(|h| {
                    vec![
                        h.dimension.clone(),
                        h.sector.clone(),
                        format::signed(h.spread),
                        h.leader.clone(),
                    ]
                })
                .collect(),
            empty: c::NO_HEADWINDS.to_owned(),
        },
        Section {
            title: c::PEERS.to_owned(),
            note: None,
            columns: columns(&[
                (c::COL_TICKER, false, 0.14),
                (c::COL_COMPARABLE, false, 0.38),
                (c::COL_OPEN, true, 0.16),
                (c::COL_CLOSE, true, 0.16),
                (c::COL_CHANGE, true, 0.16),
            ]),
            rows: d
                .peers
                .iter()
                .map(|p| match p.range {
                    Some(r) => vec![
                        p.ticker.clone(),
                        p.name.clone(),
                        format::index(r.open),
                        format::index(r.close),
                        format::signed(r.close - r.open),
                    ],
                    None => vec![p.ticker.clone(), p.name.clone(), none(), none(), none()],
                })
                .collect(),
            empty: c::NO_PEERS.to_owned(),
        },
        Section {
            title: c::INTRODUCED.to_owned(),
            note: None,
            columns: columns(&[(c::COL_DATE, false, 0.2), (c::COL_COMPARABLE, false, 0.8)]),
            rows: d
                .introduced
                .iter()
                .map(|(date, name)| vec![format::date_short(*date), name.clone()])
                .collect(),
            empty: c::NO_INTRODUCED.to_owned(),
        },
        Section {
            title: c::DISCOVERED.to_owned(),
            note: None,
            columns: columns(&[(c::COL_DATE, false, 0.2), (c::COL_DIMENSION, false, 0.8)]),
            rows: d
                .discovered
                .iter()
                .map(|(date, name)| vec![format::date_short(*date), name.clone()])
                .collect(),
            empty: c::NO_DISCOVERED.to_owned(),
        },
        Section {
            title: c::FILINGS.to_owned(),
            note: None,
            columns: columns(&[(c::COL_EVENT, false, 0.7), (c::COL_FILINGS, true, 0.3)]),
            rows: d
                .events
                .iter()
                .filter(|(_, n)| *n > 0)
                .map(|(label, n)| vec![label.clone(), format::thousands(*n as u64)])
                .collect(),
            empty: c::NO_FILINGS.to_owned(),
        },
    ];

    Statement {
        title: c::title(&d.quarter.label()),
        holder: d.name.clone(),
        period: format!(
            "{} to {}",
            format::date_long(d.quarter.first_day()),
            format::date_long(d.as_of)
        ),
        sections,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn sample() -> QuarterData {
        let day = |d| jiff::civil::date(2026, 10, d);
        QuarterData {
            quarter: Quarter { year: 2026, q: 4 },
            as_of: day(16),
            name: "Alex Morgan".into(),
            psi: Some(Range {
                open: 99.87,
                high: 99.87,
                low: 81.05,
                close: 87.22,
            }),
            threshold_at_end: Some(111.0),
            threshold_changes: vec![
                ThresholdChange {
                    date: day(1),
                    prior: Some(137.0),
                    new: 92.0,
                    reason: Reason::User,
                },
                ThresholdChange {
                    date: day(1),
                    prior: Some(92.0),
                    new: 111.0,
                    reason: Reason::Adjustment,
                },
            ],
            headwinds: vec![
                Headwind {
                    dimension: "Height (when standing next to them)".into(),
                    sector: "Physical Assets".into(),
                    spread: -4.2,
                    leader: "Gordon Pratt".into(),
                },
                Headwind {
                    dimension: "Vehicle interior condition".into(),
                    sector: "Material Holdings".into(),
                    spread: -3.1,
                    leader: "Dana".into(),
                },
            ],
            peers: vec![
                PeerResult {
                    ticker: "DANA".into(),
                    name: "Dana".into(),
                    range: Some(Range {
                        open: 93.97,
                        high: 99.0,
                        low: 92.0,
                        close: 96.27,
                    }),
                },
                PeerResult {
                    ticker: "MB".into(),
                    name: "Marcus Bell".into(),
                    range: None,
                },
            ],
            introduced: vec![],
            discovered: vec![(day(14), "Peaked (has not yet)".into())],
            positions_held: 0,
            events: vec![
                ("Compliment received".into(), 3),
                ("Slight (minor)".into(), 0),
            ],
        }
    }

    #[test]
    fn quarters() {
        let q = Quarter::containing(jiff::civil::date(2026, 11, 3));
        assert_eq!(q, Quarter { year: 2026, q: 4 });
        assert_eq!(q.first_day(), jiff::civil::date(2026, 10, 1));
        assert_eq!(q.last_day(), jiff::civil::date(2026, 12, 31));
        assert_eq!(q.next(), Quarter { year: 2027, q: 1 });
        assert_eq!(q.label(), "Q4 2026");
    }

    #[test]
    fn rows_match_their_columns() {
        let s = prepare(&sample());
        for section in &s.sections {
            for row in &section.rows {
                assert_eq!(row.len(), section.columns.len(), "{}", section.title);
            }
        }
    }

    #[test]
    fn column_shares_fill_the_width() {
        for section in prepare(&sample()).sections {
            let total: f32 = section.columns.iter().map(|c| c.width).sum();
            assert!((total - 1.0).abs() < 1e-4, "{}: {total}", section.title);
        }
    }

    #[test]
    fn empty_sections_say_so() {
        let s = prepare(&sample());
        let introduced = s
            .sections
            .iter()
            .find(|x| x.title == copy::statement::INTRODUCED)
            .unwrap();
        assert!(introduced.rows.is_empty());
        assert!(!introduced.empty.is_empty());
        let filings = s
            .sections
            .iter()
            .find(|x| x.title == copy::statement::FILINGS)
            .unwrap();
        assert_eq!(filings.rows.len(), 1, "events with no filings are omitted");
    }
}
