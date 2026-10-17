//! The Spotlight Monitor: the number of people currently thinking about the
//! user, with its margin of error.

use jiff::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpotlightReading {
    pub taken_at: Timestamp,
    /// People currently thinking about the user.
    pub count: u64,
    /// Margin of error, in people.
    pub margin: u64,
    /// When the count was last above zero, if ever.
    pub last_nonzero: Option<Timestamp>,
}

/// Takes a reading. The monitor samples once per second.
pub fn read(now: Timestamp) -> SpotlightReading {
    SpotlightReading {
        taken_at: now,
        count: 0,
        margin: 0,
        last_nonzero: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readings_are_exact() {
        let now: Timestamp = "2026-10-16T12:00:00Z".parse().unwrap();
        let r = read(now);
        assert_eq!(r.taken_at, now);
        assert_eq!((r.count, r.margin, r.last_nonzero), (0, 0, None));
    }
}
