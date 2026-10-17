//! Every constant in the Standing methodology. The Methodology appendix
//! quotes these values; change both together.

/// Self ratings are multiplied by this factor before use.
pub const SELF_ASSESSMENT_FACTOR: f64 = 0.85;

/// Comparable ratings are multiplied by this factor before use, correcting
/// for known reporting bias. The result is not capped.
pub const PEER_REPORTING_FACTOR: f64 = 1.10;

/// A Comparable not yet rated on a dimension is assigned the user's adjusted
/// rating plus this margin (the conservative peer estimate).
pub const CONSERVATIVE_PEER_MARGIN: f64 = 0.5;

/// Index level at parity.
pub const INDEX_BASELINE: f64 = 100.0;

/// Index points per point of composite spread.
pub const SPREAD_MULTIPLIER: f64 = 10.0;

/// Days for an event's contribution to momentum to halve.
pub const MOMENTUM_HALF_LIFE_DAYS: f64 = 7.0;

/// Event weights, in index points at the time of filing.
pub mod event_weight {
    pub const COMPLIMENT_RECEIVED: f64 = 2.0;
    pub const INVITATION_RECEIVED: f64 = 3.0;
    pub const NOT_INVITED: f64 = -4.0;
    pub const TEXT_UNANSWERED: f64 = -1.5;
    pub const ACHIEVEMENT: f64 = 5.0;
    pub const PUBLIC_RECOGNITION: f64 = 4.0;
    pub const SLIGHT_MINOR: f64 = -2.0;
    pub const SLIGHT_MAJOR: f64 = -5.0;
    /// Applied to the Comparable observed.
    pub const COMPARABLE_ACHIEVEMENT: f64 = 5.0;
    /// Applied to the Comparable observed.
    pub const COMPARABLE_THRIVING: f64 = 3.0;
}
