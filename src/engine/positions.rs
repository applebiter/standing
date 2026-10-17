//! Positions Held: dimensions on which the user leads every active Comparable.

use super::adjustment;
use super::constants::{CONSERVATIVE_PEER_MARGIN, volatility};
use super::index::Inputs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Volatility {
    Low,
    Moderate,
    High,
    Extreme,
}

impl Volatility {
    /// Buckets the narrowest margin by which a position is held.
    pub fn of_margin(margin: f64) -> Self {
        if margin >= volatility::LOW_FROM {
            Self::Low
        } else if margin >= volatility::MODERATE_FROM {
            Self::Moderate
        } else if margin >= volatility::HIGH_FROM {
            Self::High
        } else {
            Self::Extreme
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub dimension_id: String,
    /// The user's adjusted rating less the highest Comparable's, after adjustment.
    pub margin: f64,
    pub volatility: Volatility,
}

/// Applicable dimensions on which the user's adjusted rating exceeds every
/// active Comparable's, widest margin first. A Comparable not yet rated on a
/// dimension is taken at the conservative peer estimate. With no active
/// Comparables, no position is held.
pub fn held(inputs: &Inputs) -> Vec<Position> {
    if inputs.comparables.is_empty() {
        return Vec::new();
    }
    let mut positions: Vec<Position> = inputs
        .dimensions
        .iter()
        .filter_map(|d| {
            let own = adjustment::own(*inputs.own.get(&d.id)?);
            let best = inputs
                .comparables
                .iter()
                .map(|c| {
                    c.ratings
                        .get(&d.id)
                        .map_or(own + CONSERVATIVE_PEER_MARGIN, |&raw| adjustment::peer(raw))
                })
                .fold(f64::MIN, f64::max);
            let margin = own - best;
            (margin > 0.0).then(|| Position {
                dimension_id: d.id.clone(),
                margin,
                volatility: Volatility::of_margin(margin),
            })
        })
        .collect();
    positions.sort_by(|a, b| b.margin.total_cmp(&a.margin));
    positions
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::engine::index::{ComparableRatings, WeightedDimension};

    fn inputs(own: &[(&str, f64)], peers: &[&[(&str, f64)]]) -> Inputs {
        let map = |r: &[(&str, f64)]| -> HashMap<String, f64> {
            r.iter().map(|&(d, v)| (d.to_owned(), v)).collect()
        };
        Inputs {
            dimensions: ["A", "B", "C"]
                .iter()
                .map(|id| WeightedDimension {
                    id: (*id).into(),
                    weight: 1.0,
                })
                .collect(),
            own: map(own),
            comparables: peers
                .iter()
                .enumerate()
                .map(|(i, r)| ComparableRatings {
                    id: i as i64,
                    ratings: map(r),
                })
                .collect(),
            events: vec![],
        }
    }

    #[test]
    fn held_only_when_ahead_of_everyone() {
        let i = inputs(
            &[("A", 10.0), ("B", 10.0), ("C", 6.0)],
            &[
                &[("A", 2.0), ("B", 2.0), ("C", 1.0)],
                &[("A", 7.0), ("B", 8.0)],
            ],
        );
        let held = held(&i);
        // A: 8.5 − 7.7 = 0.8. B: 8.5 − 8.8 < 0. C: second Comparable unrated,
        // taken at 5.1 + 0.5, so not held.
        assert_eq!(held.len(), 1);
        assert_eq!(held[0].dimension_id, "A");
        assert!((held[0].margin - 0.8).abs() < 1e-9);
        assert_eq!(held[0].volatility, Volatility::High);
    }

    #[test]
    fn no_comparables_no_positions() {
        assert!(held(&inputs(&[("A", 10.0)], &[])).is_empty());
    }

    #[test]
    fn volatility_buckets() {
        assert_eq!(Volatility::of_margin(3.0), Volatility::Low);
        assert_eq!(Volatility::of_margin(1.5), Volatility::Moderate);
        assert_eq!(Volatility::of_margin(0.5), Volatility::High);
        assert_eq!(Volatility::of_margin(0.49), Volatility::Extreme);
    }

    #[test]
    fn widest_margin_first() {
        let i = inputs(&[("A", 10.0), ("B", 10.0)], &[&[("A", 1.0), ("B", 5.0)]]);
        let ids: Vec<_> = held(&i).into_iter().map(|p| p.dimension_id).collect();
        assert_eq!(ids, ["A", "B"]);
    }
}
