//! Dimension Discovery: dimensions held back from the catalog are revealed,
//! one at a time, as the user's history lengthens.

use jiff::civil::Date;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::constants::discovery::{INTERVAL_DAYS, LEAD_MARGIN, UNRATED_LEAD};
use crate::model::rating;

/// Whether a discovery is due on `today`, given the date of the last one (or
/// of the profile's creation, before the first).
pub fn due(since: Date, today: Date) -> bool {
    i64::from((today - since).get_days()) >= INTERVAL_DAYS
}

/// A dimension to reveal, and the Comparable given the lead in it.
#[derive(Debug, Clone, PartialEq)]
pub struct Discovery {
    pub dimension_id: String,
    pub leader: Option<i64>,
}

/// Chooses a hidden dimension and, if any are active, a Comparable to lead
/// it. Reproducible from the profile seed and the number of discoveries made.
pub fn choose(
    seed: i64,
    discovered: u64,
    hidden: &[String],
    comparables: &[i64],
) -> Option<Discovery> {
    if hidden.is_empty() {
        return None;
    }
    // A distinct stream from Peer Set Rebalancing's.
    let mut rng = ChaCha8Rng::seed_from_u64(
        (seed as u64 ^ 0xD15C_0BE2_D15C_0BE2)
            .wrapping_add(discovered.wrapping_mul(0x9E37_79B9_7F4A_7C15)),
    );
    let dimension_id = hidden[rng.random_range(0..hidden.len())].clone();
    let leader =
        (!comparables.is_empty()).then(|| comparables[rng.random_range(0..comparables.len())]);
    Some(Discovery {
        dimension_id,
        leader,
    })
}

/// The leading Comparable's provisional raw rating.
pub fn lead_rating(own_raw: Option<f64>) -> f64 {
    own_raw.map_or(UNRATED_LEAD, |r| (r + LEAD_MARGIN).min(rating::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(day: i8) -> Date {
        jiff::civil::date(2026, 10, day)
    }

    #[test]
    fn every_three_calendar_days() {
        assert!(!due(d(10), d(10)));
        assert!(!due(d(10), d(12)));
        assert!(due(d(10), d(13)));
        assert!(due(d(10), d(30)));
    }

    #[test]
    fn choices_are_reproducible_and_drawn_from_what_is_hidden() {
        let hidden: Vec<String> = ["DOM-004", "PHY-017", "LEI-013"].map(String::from).to_vec();
        let a = choose(42, 0, &hidden, &[7, 9]).unwrap();
        assert_eq!(a, choose(42, 0, &hidden, &[7, 9]).unwrap());
        assert!(hidden.contains(&a.dimension_id));
        assert!(matches!(a.leader, Some(7 | 9)));
        let seen: std::collections::HashSet<_> = (0..20)
            .map(|n| choose(42, n, &hidden, &[7]).unwrap().dimension_id)
            .collect();
        assert_eq!(seen.len(), 3);
        assert_eq!(choose(42, 0, &hidden, &[]).unwrap().leader, None);
        assert_eq!(choose(42, 0, &[], &[7]), None);
    }

    #[test]
    fn the_leader_leads() {
        assert_eq!(lead_rating(None), 7.0);
        assert_eq!(lead_rating(Some(4.5)), 5.5);
        assert_eq!(lead_rating(Some(9.5)), 10.0);
        assert_eq!(lead_rating(Some(10.0)), 10.0);
    }
}
