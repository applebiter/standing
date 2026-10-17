//! Composite Standing, momentum and the Personal Standing Index.

use std::collections::HashMap;

use jiff::Timestamp;

use super::adjustment;
use super::constants::{
    CONSERVATIVE_PEER_MARGIN, INDEX_BASELINE, MOMENTUM_HALF_LIFE_DAYS, SPREAD_MULTIPLIER,
};
use crate::model::event::Event;

/// An applicable dimension and its current weight.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDimension {
    pub id: String,
    pub weight: f64,
}

/// Current raw ratings for one active Comparable, by dimension.
#[derive(Debug, Clone, PartialEq)]
pub struct ComparableRatings {
    pub id: i64,
    pub ratings: HashMap<String, f64>,
}

/// Everything the index is computed from.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Inputs {
    /// Dimensions that apply and are not hidden.
    pub dimensions: Vec<WeightedDimension>,
    /// The user's current raw ratings, by dimension.
    pub own: HashMap<String, f64>,
    /// Active Comparables only.
    pub comparables: Vec<ComparableRatings>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComparableReading {
    pub id: i64,
    /// `None` until the user has rated themselves on an applicable dimension.
    pub composite: Option<f64>,
    pub momentum: f64,
    /// The Comparable's ticker value.
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub computed_at: Timestamp,
    /// `None` until the user has rated themselves on an applicable dimension.
    pub own_composite: Option<f64>,
    pub momentum: f64,
    /// The Personal Standing Index.
    pub psi: f64,
    pub comparables: Vec<ComparableReading>,
}

impl Reading {
    pub fn comparable(&self, id: i64) -> Option<&ComparableReading> {
        self.comparables.iter().find(|c| c.id == id)
    }

    /// The user's spread against a Comparable, in index points: positive when
    /// the user leads.
    pub fn spread(&self, id: i64) -> Option<f64> {
        let own = self.own_composite?;
        let theirs = self.comparable(id)?.composite?;
        Some(SPREAD_MULTIPLIER * (own - theirs))
    }
}

/// Weighted mean of the user's adjusted ratings, over applicable dimensions
/// the user has rated.
pub fn own_composite(inputs: &Inputs) -> Option<f64> {
    weighted_mean(
        inputs
            .dimensions
            .iter()
            .filter_map(|d| Some((d.weight, adjustment::own(*inputs.own.get(&d.id)?)))),
    )
}

/// Weighted mean of a Comparable's adjusted ratings, over the same dimensions
/// as the user's composite. Where the Comparable is unrated, the conservative
/// peer estimate (the user's adjusted rating plus the margin) is used.
pub fn comparable_composite(inputs: &Inputs, c: &ComparableRatings) -> Option<f64> {
    weighted_mean(inputs.dimensions.iter().filter_map(|d| {
        let own = adjustment::own(*inputs.own.get(&d.id)?);
        let theirs = c
            .ratings
            .get(&d.id)
            .map_or(own + CONSERVATIVE_PEER_MARGIN, |&raw| adjustment::peer(raw));
        Some((d.weight, theirs))
    }))
}

fn weighted_mean(items: impl Iterator<Item = (f64, f64)>) -> Option<f64> {
    let (sum, weights) = items.fold((0.0, 0.0), |(s, w), (weight, value)| {
        (s + weight * value, w + weight)
    });
    (weights > 0.0).then(|| sum / weights)
}

/// Sum of event weights, each halved for every half-life since it occurred.
/// Events dated in the future count at full weight.
pub fn momentum<'a>(events: impl IntoIterator<Item = &'a Event>, now: Timestamp) -> f64 {
    events
        .into_iter()
        .map(|e| {
            let age_days = now.duration_since(e.occurred_at).as_secs_f64() / 86_400.0;
            e.kind.weight() * 0.5_f64.powf(age_days.max(0.0) / MOMENTUM_HALF_LIFE_DAYS)
        })
        .sum()
}

/// Computes the Personal Standing Index and every active Comparable's ticker.
pub fn compute(inputs: &Inputs, now: Timestamp) -> Reading {
    let own_composite = own_composite(inputs);
    let own_events = inputs
        .events
        .iter()
        .filter(|e| !e.kind.concerns_comparable());
    let own_momentum = momentum(own_events, now);

    let comparables: Vec<ComparableReading> = inputs
        .comparables
        .iter()
        .map(|c| {
            let composite = comparable_composite(inputs, c);
            let observed = inputs
                .events
                .iter()
                .filter(|e| e.kind.concerns_comparable() && e.comparable_id == Some(c.id));
            let momentum = momentum(observed, now);
            let spread = match (composite, own_composite) {
                (Some(theirs), Some(own)) => SPREAD_MULTIPLIER * (theirs - own),
                _ => 0.0,
            };
            ComparableReading {
                id: c.id,
                composite,
                momentum,
                value: INDEX_BASELINE + spread + momentum,
            }
        })
        .collect();

    let peer_composites: Vec<f64> = comparables.iter().filter_map(|c| c.composite).collect();
    let standing = match own_composite {
        Some(own) if !peer_composites.is_empty() => {
            let mean = peer_composites.iter().sum::<f64>() / peer_composites.len() as f64;
            SPREAD_MULTIPLIER * (own - mean)
        }
        _ => 0.0,
    };

    Reading {
        computed_at: now,
        own_composite,
        momentum: own_momentum,
        psi: INDEX_BASELINE + standing + own_momentum,
        comparables,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::event::EventKind;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn at(s: &str) -> Timestamp {
        s.parse().unwrap()
    }

    fn dims(spec: &[(&str, f64)]) -> Vec<WeightedDimension> {
        spec.iter()
            .map(|&(id, weight)| WeightedDimension {
                id: id.into(),
                weight,
            })
            .collect()
    }

    fn ratings(spec: &[(&str, f64)]) -> HashMap<String, f64> {
        spec.iter().map(|&(d, v)| (d.to_owned(), v)).collect()
    }

    fn event(kind: EventKind, comparable_id: Option<i64>, when: &str) -> Event {
        Event {
            id: 0,
            kind,
            comparable_id,
            note: None,
            occurred_at: at(when),
        }
    }

    #[test]
    fn own_composite_is_weighted_over_rated_dimensions() {
        let inputs = Inputs {
            dimensions: dims(&[("A", 1.0), ("B", 3.0), ("C", 2.0)]),
            own: ratings(&[("A", 10.0), ("B", 6.0)]),
            ..Default::default()
        };
        // (1 × 8.5 + 3 × 5.1) / 4 = 5.95
        assert!(close(own_composite(&inputs).unwrap(), 5.95));
    }

    #[test]
    fn ratings_on_inapplicable_dimensions_are_ignored() {
        let inputs = Inputs {
            dimensions: dims(&[("A", 1.0)]),
            own: ratings(&[("A", 4.0), ("Z", 10.0)]),
            ..Default::default()
        };
        assert!(close(own_composite(&inputs).unwrap(), 3.4));
    }

    #[test]
    fn comparable_composite_imputes_and_excludes() {
        let c = ComparableRatings {
            id: 1,
            // Rated on A; unrated on B (imputed); rated on C, which the user
            // has not rated (excluded).
            ratings: ratings(&[("A", 6.0), ("C", 10.0)]),
        };
        let inputs = Inputs {
            dimensions: dims(&[("A", 1.0), ("B", 1.0), ("C", 1.0)]),
            own: ratings(&[("A", 6.0), ("B", 4.0)]),
            comparables: vec![c.clone()],
            ..Default::default()
        };
        // A: 6 × 1.10 = 6.6. B: 4 × 0.85 + 0.5 = 3.9. Mean 5.25.
        assert!(close(comparable_composite(&inputs, &c).unwrap(), 5.25));
    }

    #[test]
    fn nothing_rated_means_no_composites() {
        let inputs = Inputs {
            dimensions: dims(&[("A", 1.0)]),
            comparables: vec![ComparableRatings {
                id: 1,
                ratings: ratings(&[("A", 9.0)]),
            }],
            ..Default::default()
        };
        let r = compute(&inputs, at("2026-10-16T12:00:00Z"));
        assert_eq!(r.own_composite, None);
        assert_eq!(r.comparables[0].composite, None);
        assert!(close(r.psi, 100.0));
        assert!(close(r.comparables[0].value, 100.0));
        assert_eq!(r.spread(1), None);
    }

    #[test]
    fn psi_against_the_mean_of_comparables() {
        let inputs = Inputs {
            dimensions: dims(&[("A", 1.0)]),
            own: ratings(&[("A", 10.0)]),
            comparables: vec![
                ComparableRatings {
                    id: 1,
                    ratings: ratings(&[("A", 5.0)]),
                },
                ComparableRatings {
                    id: 2,
                    ratings: ratings(&[("A", 10.0)]),
                },
            ],
            ..Default::default()
        };
        let r = compute(&inputs, at("2026-10-16T12:00:00Z"));
        // Own 8.5; peers 5.5 and 11.0, mean 8.25. PSI = 100 + 10 × 0.25.
        assert!(close(r.psi, 102.5));
        // Tickers: 100 + 10 × (5.5 − 8.5) and 100 + 10 × (11.0 − 8.5).
        assert!(close(r.comparables[0].value, 70.0));
        assert!(close(r.comparables[1].value, 125.0));
        assert!(close(r.spread(1).unwrap(), 30.0));
        assert!(close(r.spread(2).unwrap(), -25.0));
    }

    #[test]
    fn equal_raw_ratings_place_the_user_behind() {
        let inputs = Inputs {
            dimensions: dims(&[("A", 1.0)]),
            own: ratings(&[("A", 7.0)]),
            comparables: vec![ComparableRatings {
                id: 1,
                ratings: ratings(&[("A", 7.0)]),
            }],
            ..Default::default()
        };
        let r = compute(&inputs, at("2026-10-16T12:00:00Z"));
        // 10 × (5.95 − 7.7) = −17.5
        assert!(close(r.psi, 82.5));
    }

    #[test]
    fn momentum_halves_every_seven_days() {
        let now = at("2026-10-16T12:00:00Z");
        let e = |when| vec![event(EventKind::Achievement, None, when)];
        assert!(close(momentum(&e("2026-10-16T12:00:00Z"), now), 5.0));
        assert!(close(momentum(&e("2026-10-09T12:00:00Z"), now), 2.5));
        assert!(close(momentum(&e("2026-10-02T12:00:00Z"), now), 1.25));
        assert!(close(
            momentum(&e("2026-10-15T00:00:00Z"), now),
            5.0 * 0.5_f64.powf(1.5 / 7.0)
        ));
        // Future-dated events count at full weight.
        assert!(close(momentum(&e("2026-10-20T12:00:00Z"), now), 5.0));
    }

    #[test]
    fn events_move_the_right_ticker() {
        let now = at("2026-10-16T12:00:00Z");
        let inputs = Inputs {
            dimensions: dims(&[("A", 1.0)]),
            own: ratings(&[("A", 10.0)]),
            comparables: vec![
                ComparableRatings {
                    id: 1,
                    ratings: ratings(&[("A", 10.0)]),
                },
                ComparableRatings {
                    id: 2,
                    ratings: ratings(&[("A", 10.0)]),
                },
            ],
            events: vec![
                event(EventKind::SlightMajor, None, "2026-10-16T12:00:00Z"),
                event(
                    EventKind::ComparableThriving,
                    Some(2),
                    "2026-10-09T12:00:00Z",
                ),
            ],
        };
        let r = compute(&inputs, now);
        // Own 8.5, peers 11.0: 100 − 25 − 5.
        assert!(close(r.momentum, -5.0));
        assert!(close(r.psi, 70.0));
        assert!(close(r.comparables[0].value, 125.0));
        assert!(close(r.comparables[1].momentum, 1.5));
        assert!(close(r.comparables[1].value, 126.5));
    }

    #[test]
    fn no_comparables_means_baseline_plus_momentum() {
        let inputs = Inputs {
            dimensions: dims(&[("A", 1.0)]),
            own: ratings(&[("A", 3.0)]),
            events: vec![event(
                EventKind::InvitationReceived,
                None,
                "2026-10-16T12:00:00Z",
            )],
            ..Default::default()
        };
        let r = compute(&inputs, at("2026-10-16T12:00:00Z"));
        assert!(close(r.psi, 103.0));
    }
}
