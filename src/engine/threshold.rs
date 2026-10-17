//! The Enough Threshold: the index level at which the user will feel secure.

use super::constants::threshold::{ADJUSTMENT_FACTOR, MINIMUM};

/// An adjustment from one threshold to the next.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Adjustment {
    pub prior: f64,
    pub new: f64,
}

/// The threshold that follows `t`: `t` × the adjustment factor, rounded up to
/// the next whole point. Products are first rounded to six decimal places, so
/// representation error never adds a point (110 × 1.20 is 132, not 133).
pub fn next(t: f64) -> f64 {
    ((t * ADJUSTMENT_FACTOR * 1e6).round() / 1e6).ceil()
}

/// The adjustments required while the index stands at or above the
/// threshold, in order. Empty if the index is below it.
pub fn adjustments(threshold: f64, psi: f64) -> Vec<Adjustment> {
    // Bounded, so that no input can stall the recomputation.
    const MAX_STEPS: usize = 1_000;
    let mut out = Vec::new();
    let mut t = threshold.max(MINIMUM);
    while psi >= t && out.len() < MAX_STEPS {
        let new = next(t);
        out.push(Adjustment { prior: t, new });
        t = new;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_rounds_up_to_whole_points() {
        assert_eq!(next(100.0), 120.0);
        assert_eq!(next(110.0), 132.0);
        assert_eq!(next(110.5), 133.0);
        assert_eq!(next(5.0), 6.0);
        assert_eq!(next(2.0), 3.0);
        assert_eq!(next(1.0), 2.0);
    }

    #[test]
    fn below_the_threshold_nothing_happens() {
        assert!(adjustments(120.0, 119.99).is_empty());
    }

    #[test]
    fn reaching_the_threshold_raises_it() {
        assert_eq!(
            adjustments(120.0, 120.0),
            [Adjustment {
                prior: 120.0,
                new: 144.0
            }]
        );
    }

    #[test]
    fn a_jump_over_several_thresholds_records_each_step() {
        let steps = adjustments(100.0, 150.0);
        let pairs: Vec<(f64, f64)> = steps.iter().map(|a| (a.prior, a.new)).collect();
        assert_eq!(pairs, [(100.0, 120.0), (120.0, 144.0), (144.0, 173.0)]);
    }

    #[test]
    fn always_terminates_above_the_index() {
        for (t, psi) in [(1.0, 1_000.0), (0.0, 50.0), (-5.0, 3.0), (7.5, 7.5)] {
            let steps = adjustments(t, psi);
            assert!(steps.last().is_none_or(|a| a.new > psi), "{t} {psi}");
        }
        assert!(adjustments(1.0, f64::INFINITY).len() <= 1_000);
        assert!(adjustments(100.0, f64::NAN).is_empty());
    }
}
