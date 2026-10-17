//! The Quarterly Earnings Call: a transcript in which the user, as Chief
//! Executive, presents the quarter to analysts.

use jiff::civil::Date;
use serde::Serialize;

use super::phrases::{self as ph, fill, pick};
use crate::format;

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

    /// A number unique to the quarter, for choosing phrasing.
    fn ordinal(self) -> u64 {
        (self.year as u64) * 4 + self.q as u64
    }
}

/// Opening and closing values of an index over the quarter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
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

/// Everything the call is prepared from.
#[derive(Debug, Clone, PartialEq)]
pub struct QuarterData {
    pub quarter: Quarter,
    /// The last day covered: the quarter's end, or today if earlier.
    pub as_of: Date,
    pub name: String,
    pub seed: i64,
    pub psi: Option<Range>,
    pub threshold_at_end: Option<f64>,
    /// (prior, new) for each adjustment in the quarter.
    pub adjustments: Vec<(f64, f64)>,
    /// Weakest first.
    pub headwinds: Vec<Headwind>,
    pub peers: Vec<PeerResult>,
    pub introduced: Vec<String>,
    pub discovered: Vec<String>,
    pub positions_held: usize,
    /// (event label, count), in a fixed order.
    pub events: Vec<(String, usize)>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Figure {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Operator,
    Executive,
    Analyst,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Turn {
    pub speaker: String,
    pub role: Role,
    pub paragraphs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Section {
    pub title: String,
    pub turns: Vec<Turn>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EarningsCall {
    pub title: String,
    pub company: String,
    pub period: String,
    pub figures: Vec<Figure>,
    pub sections: Vec<Section>,
    /// Supplementary tables: headwinds by spread, and filings by type.
    pub headwinds: Vec<Figure>,
    pub filings: Vec<Figure>,
}

const OPERATOR: &str = "Operator";

fn executive(name: &str) -> String {
    format!("{name}, Chief Executive")
}

fn analyst(firm: &str) -> String {
    format!("Analyst, {firm}")
}

/// "once", "twice", "3 times".
fn times(n: usize) -> String {
    match n {
        1 => "once".to_owned(),
        2 => "twice".to_owned(),
        n => format!("{n} times"),
    }
}

/// "A", "A and B", "A, B and C".
fn list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

fn pct(change: f64, base: f64) -> String {
    let p = if base.abs() > f64::EPSILON {
        100.0 * change / base.abs()
    } else {
        0.0
    };
    format::signed_percent(p)
}

/// Prepares the transcript.
pub fn prepare(d: &QuarterData) -> EarningsCall {
    let n = d.quarter.ordinal().wrapping_add(d.seed as u64);
    let quarter = d.quarter.label();
    let exec = executive(&d.name);

    // Highlights.
    let mut figures = Vec::new();
    let dash = || "\u{2014}".to_owned();
    match d.psi {
        Some(r) => {
            let change = r.close - r.open;
            figures.extend([
                ("Opening index", format::index(r.open)),
                ("Closing index", format::index(r.close)),
                (
                    "Change",
                    format!("{} ({})", format::signed(change), pct(change, r.open)),
                ),
                (
                    "Quarter range",
                    format!("{} to {}", format::index(r.low), format::index(r.high)),
                ),
            ]);
        }
        None => figures.push(("Closing index", dash())),
    }
    figures.push((
        "Enough Threshold",
        d.threshold_at_end.map_or_else(dash, format::index),
    ));
    figures.push(("Threshold revisions", d.adjustments.len().to_string()));
    figures.push(("Positions held", d.positions_held.to_string()));
    let figures = figures
        .into_iter()
        .map(|(label, value)| Figure {
            label: label.to_owned(),
            value,
        })
        .collect();

    // Prepared remarks.
    let mut remarks = vec![pick(&ph::GREETING, n, 1).to_owned()];
    match d.psi {
        Some(r) => {
            let change = r.close - r.open;
            let bank = if change >= 0.005 {
                &ph::PERFORMANCE_UP[..]
            } else if change <= -0.005 {
                &ph::PERFORMANCE_DOWN[..]
            } else {
                &ph::PERFORMANCE_FLAT[..]
            };
            remarks.push(fill(
                pick(bank, n, 2),
                &[
                    ("open", &format::index(r.open)),
                    ("close", &format::index(r.close)),
                    ("change", &format::signed(change)),
                    ("pct", &pct(change, r.open)),
                    ("high", &format::index(r.high)),
                    ("low", &format::index(r.low)),
                ],
            ));
        }
        None => remarks.push(ph::NO_TRADING.to_owned()),
    }
    if let Some(to) = d.threshold_at_end {
        let to = format::index(to);
        remarks.push(match (d.adjustments.first(), d.adjustments.last()) {
            (Some(first), Some(last)) => fill(
                pick(&ph::THRESHOLD_REVISED, n, 3),
                &[
                    ("count", &times(d.adjustments.len())),
                    ("from", &format::index(first.0)),
                    ("to", &format::index(last.1)),
                ],
            ),
            _ => fill(pick(&ph::THRESHOLD_UNCHANGED, n, 3), &[("to", &to)]),
        });
    }
    if !d.headwinds.is_empty() {
        let names: Vec<&str> = d
            .headwinds
            .iter()
            .take(3)
            .map(|h| h.dimension.as_str())
            .collect();
        remarks.push(fill(
            pick(&ph::HEADWINDS, n, 4),
            &[("list", &list(&names).to_lowercase())],
        ));
        let first = &d.headwinds[0];
        let sector = d.headwinds.get(1).unwrap_or(first).sector.to_lowercase();
        remarks.push(fill(
            pick(&ph::INITIATIVES, n, 5),
            &[
                ("first", &first.dimension.to_lowercase()),
                ("sector", &sector),
            ],
        ));
    }
    let gainers = gainers(d);
    if let Some((p, change)) = gainers.first() {
        remarks.push(fill(
            pick(&ph::PEER_GAINER, n, 6),
            &[
                ("name", &ph::in_sentence(&p.name)),
                ("ticker", &p.ticker),
                ("change", &format::index(*change)),
            ],
        ));
    }
    if !d.introduced.is_empty() {
        let names: Vec<String> = d.introduced.iter().map(|n| ph::in_sentence(n)).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let count = match d.introduced.len() {
            1 => "one new Comparable".to_owned(),
            k => format!("{k} new Comparables"),
        };
        remarks.push(fill(
            pick(&ph::PEER_ADDED, n, 7),
            &[("count", &count), ("names", &list(&names))],
        ));
    }
    if !d.discovered.is_empty() {
        let names: Vec<&str> = d.discovered.iter().map(String::as_str).collect();
        remarks.push(fill(
            pick(&ph::DISCOVERIES, n, 8),
            &[("names", &list(&names).to_lowercase())],
        ));
    }
    remarks.push(pick(&ph::HANDOVER, n, 9).to_owned());

    let opening = vec![
        Turn {
            speaker: OPERATOR.to_owned(),
            role: Role::Operator,
            paragraphs: vec![fill(
                pick(&ph::OPERATOR_OPENING, n, 0),
                &[("quarter", &quarter), ("name", &d.name)],
            )],
        },
        Turn {
            speaker: exec.clone(),
            role: Role::Executive,
            paragraphs: remarks,
        },
    ];

    // Questions and answers.
    // Each question, and whether it concerns a particular Comparable.
    let mut questions: Vec<(String, bool)> = Vec::new();
    for (i, h) in d.headwinds.iter().take(2).enumerate() {
        questions.push((
            fill(
                pick(&ph::Q_HEADWIND, n, 20 + i as u64),
                &[
                    ("dimension", &h.dimension.to_lowercase()),
                    ("leader", &ph::in_sentence(&h.leader)),
                    ("Leader", &ph::opening(&h.leader)),
                ],
            ),
            false,
        ));
    }
    if let Some(t) = d.threshold_at_end {
        let close = d.psi.map_or(t, |r| r.close);
        questions.push((
            fill(
                pick(&ph::Q_THRESHOLD, n, 22),
                &[
                    ("to", &format::index(t)),
                    ("shortfall", &format::index((t - close).max(0.0))),
                ],
            ),
            false,
        ));
    }
    if let Some((p, change)) = gainers.first() {
        questions.push((
            fill(
                pick(&ph::Q_PEER, n, 23),
                &[
                    ("name", &ph::in_sentence(&p.name)),
                    ("Name", &ph::opening(&p.name)),
                    ("change", &format::index(*change)),
                ],
            ),
            true,
        ));
    }
    let positions = match d.positions_held {
        0 => "no positions".to_owned(),
        1 => "one position".to_owned(),
        k => format!("{k} positions"),
    };
    questions.push((
        fill(pick(&ph::Q_POSITIONS, n, 24), &[("count", &positions)]),
        false,
    ));
    questions.push((pick(&ph::Q_SPOTLIGHT, n, 25).to_owned(), false));

    let mut qa = Vec::new();
    let firm_start = (n % ph::FIRMS.len() as u64) as usize;
    let evasion_start = (n.wrapping_mul(31) % ph::EVASIONS.len() as u64) as usize;
    for (i, (question, about_peer)) in questions.into_iter().enumerate() {
        let firm = ph::FIRMS[(firm_start + i) % ph::FIRMS.len()];
        qa.push(Turn {
            speaker: OPERATOR.to_owned(),
            role: Role::Operator,
            paragraphs: vec![fill(
                if i == 0 {
                    ph::OPERATOR_FIRST_QUESTION
                } else {
                    pick(&ph::OPERATOR_QUESTION, n, 30 + i as u64)
                },
                &[("firm", firm)],
            )],
        });
        qa.push(Turn {
            speaker: analyst(firm),
            role: Role::Analyst,
            paragraphs: vec![question],
        });
        qa.push(Turn {
            speaker: exec.clone(),
            role: Role::Executive,
            paragraphs: vec![
                if about_peer {
                    pick(&ph::PEER_EVASIONS, n, 50)
                } else {
                    ph::EVASIONS[(evasion_start + i * 5) % ph::EVASIONS.len()]
                }
                .to_owned(),
            ],
        });
    }
    qa.push(Turn {
        speaker: OPERATOR.to_owned(),
        role: Role::Operator,
        paragraphs: vec![fill(
            pick(&ph::OPERATOR_TO_CLOSING, n, 39),
            &[("name", &d.name)],
        )],
    });
    qa.push(Turn {
        speaker: exec.clone(),
        role: Role::Executive,
        paragraphs: vec![pick(&ph::CLOSING_REMARK, n, 40).to_owned()],
    });
    qa.push(Turn {
        speaker: OPERATOR.to_owned(),
        role: Role::Operator,
        paragraphs: vec![pick(&ph::OPERATOR_CLOSING, n, 41).to_owned()],
    });

    EarningsCall {
        title: format!("{quarter} Earnings Call"),
        company: exec,
        period: format!(
            "{} to {}",
            format::date_long(d.quarter.first_day()),
            format::date_long(d.as_of)
        ),
        figures,
        sections: vec![
            Section {
                title: "Prepared Remarks".to_owned(),
                turns: opening,
            },
            Section {
                title: "Question-and-Answer Session".to_owned(),
                turns: qa,
            },
        ],
        headwinds: d
            .headwinds
            .iter()
            .map(|h| Figure {
                label: h.dimension.clone(),
                value: format::signed(h.spread),
            })
            .collect(),
        filings: d
            .events
            .iter()
            .filter(|(_, count)| *count > 0)
            .map(|(label, count)| Figure {
                label: label.clone(),
                value: format::thousands(*count as u64),
            })
            .collect(),
    }
}

/// Comparables whose index rose, largest gain first.
fn gainers(d: &QuarterData) -> Vec<(&PeerResult, f64)> {
    let mut out: Vec<(&PeerResult, f64)> = d
        .peers
        .iter()
        .filter_map(|p| {
            let r = p.range?;
            let change = r.close - r.open;
            (change > 0.0).then_some((p, change))
        })
        .collect();
    out.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then_with(|| a.0.ticker.cmp(&b.0.ticker))
    });
    out
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn sample() -> QuarterData {
        QuarterData {
            quarter: Quarter { year: 2026, q: 4 },
            as_of: jiff::civil::date(2026, 10, 16),
            name: "Alex".into(),
            seed: 42,
            psi: Some(Range {
                open: 96.0,
                high: 101.25,
                low: 88.5,
                close: 90.84,
            }),
            threshold_at_end: Some(105.0),
            adjustments: vec![(50.0, 60.0), (60.0, 72.0), (72.0, 87.0), (87.0, 105.0)],
            headwinds: vec![
                Headwind {
                    dimension: "Sourdough starter age".into(),
                    sector: "Domestic Operations".into(),
                    spread: -4.2,
                    leader: "Dana".into(),
                },
                Headwind {
                    dimension: "Parallel parking under observation".into(),
                    sector: "Procedural Competence".into(),
                    spread: -3.1,
                    leader: "Marcus Bell".into(),
                },
                Headwind {
                    dimension: "Hairline trajectory".into(),
                    sector: "Physical Assets".into(),
                    spread: -2.5,
                    leader: "Dana".into(),
                },
            ],
            peers: vec![
                PeerResult {
                    ticker: "DANA".into(),
                    name: "Dana".into(),
                    range: Some(Range {
                        open: 104.0,
                        high: 111.0,
                        low: 103.0,
                        close: 110.48,
                    }),
                },
                PeerResult {
                    ticker: "MB".into(),
                    name: "Marcus Bell".into(),
                    range: Some(Range {
                        open: 114.0,
                        high: 114.0,
                        low: 112.0,
                        close: 113.62,
                    }),
                },
            ],
            introduced: vec!["Your manager's manager".into()],
            discovered: vec!["Cat's apparent affection".into()],
            positions_held: 1,
            events: vec![("Compliment received".into(), 3)],
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
    fn the_same_quarter_reads_the_same() {
        assert_eq!(prepare(&sample()), prepare(&sample()));
    }

    #[test]
    fn every_placeholder_is_filled_in_every_variant() {
        for seed in 0..60 {
            for up in [false, true] {
                let mut d = sample();
                d.seed = seed;
                if up {
                    d.psi = d.psi.map(|r| Range {
                        close: r.open + 3.0,
                        ..r
                    });
                    d.adjustments.clear();
                }
                let call = prepare(&d);
                for turn in call.sections.iter().flat_map(|s| &s.turns) {
                    for p in &turn.paragraphs {
                        assert!(!p.contains('{') && !p.contains('}'), "seed {seed}: {p}");
                    }
                }
            }
        }
    }

    #[test]
    fn an_empty_quarter_still_reads_cleanly() {
        let d = QuarterData {
            psi: None,
            threshold_at_end: None,
            adjustments: vec![],
            headwinds: vec![],
            peers: vec![],
            introduced: vec![],
            discovered: vec![],
            positions_held: 0,
            events: vec![],
            ..sample()
        };
        let call = prepare(&d);
        let text: Vec<&String> = call
            .sections
            .iter()
            .flat_map(|s| &s.turns)
            .flat_map(|t| &t.paragraphs)
            .collect();
        assert!(text.iter().any(|p| p.as_str() == ph::NO_TRADING));
        assert!(text.iter().all(|p| !p.contains('{')));
    }

    #[test]
    fn introduced_comparables_read_naturally_in_a_sentence() {
        assert_eq!(
            ph::in_sentence("Your manager's manager"),
            "your manager's manager"
        );
        assert_eq!(
            ph::in_sentence("A person your age who retired last year"),
            "a person your age who retired last year"
        );
        assert_eq!(
            ph::in_sentence("Dana's brother-in-law (the one with the boat)"),
            "Dana's brother-in-law (the one with the boat)"
        );
        assert_eq!(ph::in_sentence("Marcus Bell"), "Marcus Bell");
        assert_eq!(ph::opening("your landlord"), "Your landlord");
    }

    #[test]
    fn the_first_question_is_introduced_as_the_first() {
        let call = prepare(&sample());
        let operator_lines: Vec<&str> = call.sections[1]
            .turns
            .iter()
            .filter(|t| t.role == Role::Operator)
            .map(|t| t.paragraphs[0].as_str())
            .collect();
        assert!(operator_lines[0].starts_with("Our first question"));
        assert!(operator_lines[1..].iter().all(|l| !l.contains("first")));
    }

    #[test]
    fn a_flat_empty_quarter_reads_plainly() {
        let d = QuarterData {
            psi: Some(Range {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
            }),
            threshold_at_end: Some(200.0),
            adjustments: vec![],
            headwinds: vec![],
            peers: vec![],
            introduced: vec![],
            discovered: vec![],
            positions_held: 0,
            ..sample()
        };
        for seed in 0..20 {
            let call = prepare(&QuarterData { seed, ..d.clone() });
            let text: String = call
                .sections
                .iter()
                .flat_map(|s| &s.turns)
                .flat_map(|t| t.paragraphs.clone())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(text.contains("unchanged"), "{text}");
            assert!(!text.contains("0 positions"));
            assert!(!text.contains("individual Comparables"));
        }
    }

    #[test]
    fn lists_read_naturally() {
        assert_eq!(list(&["A"]), "A");
        assert_eq!(list(&["A", "B"]), "A and B");
        assert_eq!(list(&["A", "B", "C"]), "A, B and C");
        assert_eq!(times(1), "once");
        assert_eq!(times(4), "4 times");
    }
}
