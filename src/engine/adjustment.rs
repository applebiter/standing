//! The Self-Assessment Adjustment.

use super::constants::{PEER_REPORTING_FACTOR, SELF_ASSESSMENT_FACTOR};

/// A self rating, adjusted.
pub fn own(raw: f64) -> f64 {
    raw * SELF_ASSESSMENT_FACTOR
}

/// A Comparable's rating, adjusted. Not capped: a raw 10 becomes 11.
pub fn peer(raw: f64) -> f64 {
    raw * PEER_REPORTING_FACTOR
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn adjustment_factors() {
        assert!(close(own(10.0), 8.5));
        assert!(close(own(0.0), 0.0));
        assert!(close(peer(10.0), 11.0));
        assert!(close(peer(5.0), 5.5));
    }

    #[test]
    fn parity_requires_a_raw_lead() {
        // Equal raw ratings never adjust to parity.
        for raw in [1.0, 5.0, 10.0] {
            assert!(own(raw) < peer(raw));
        }
    }
}
