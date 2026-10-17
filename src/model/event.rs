//! Status events recorded in the Daily Filing.

use jiff::Timestamp;

use crate::engine::constants::event_weight as w;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    ComplimentReceived,
    InvitationReceived,
    NotInvited,
    TextUnanswered,
    Achievement,
    PublicRecognition,
    SlightMinor,
    SlightMajor,
    ComparableAchievement,
    ComparableThriving,
}

impl EventKind {
    pub const ALL: [Self; 10] = [
        Self::ComplimentReceived,
        Self::InvitationReceived,
        Self::NotInvited,
        Self::TextUnanswered,
        Self::Achievement,
        Self::PublicRecognition,
        Self::SlightMinor,
        Self::SlightMajor,
        Self::ComparableAchievement,
        Self::ComparableThriving,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ComplimentReceived => "compliment_received",
            Self::InvitationReceived => "invitation_received",
            Self::NotInvited => "not_invited",
            Self::TextUnanswered => "text_unanswered",
            Self::Achievement => "achievement",
            Self::PublicRecognition => "public_recognition",
            Self::SlightMinor => "slight_minor",
            Self::SlightMajor => "slight_major",
            Self::ComparableAchievement => "comparable_achievement",
            Self::ComparableThriving => "comparable_thriving",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == s)
    }

    /// Index points at the time of filing.
    pub fn weight(self) -> f64 {
        match self {
            Self::ComplimentReceived => w::COMPLIMENT_RECEIVED,
            Self::InvitationReceived => w::INVITATION_RECEIVED,
            Self::NotInvited => w::NOT_INVITED,
            Self::TextUnanswered => w::TEXT_UNANSWERED,
            Self::Achievement => w::ACHIEVEMENT,
            Self::PublicRecognition => w::PUBLIC_RECOGNITION,
            Self::SlightMinor => w::SLIGHT_MINOR,
            Self::SlightMajor => w::SLIGHT_MAJOR,
            Self::ComparableAchievement => w::COMPARABLE_ACHIEVEMENT,
            Self::ComparableThriving => w::COMPARABLE_THRIVING,
        }
    }

    /// Whether the event is observed of a Comparable rather than the user.
    pub fn concerns_comparable(self) -> bool {
        matches!(self, Self::ComparableAchievement | Self::ComparableThriving)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id: i64,
    pub kind: EventKind,
    /// Set for events observed of a Comparable.
    pub comparable_id: Option<i64>,
    pub note: Option<String>,
    pub occurred_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_round_trip_and_target_correctly() {
        for k in EventKind::ALL {
            assert_eq!(EventKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(EventKind::parse("unknown"), None);
        let observed: Vec<_> = EventKind::ALL
            .into_iter()
            .filter(|k| k.concerns_comparable())
            .collect();
        assert_eq!(
            observed,
            [
                EventKind::ComparableAchievement,
                EventKind::ComparableThriving
            ]
        );
    }
}
