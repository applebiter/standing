//! Ratings on the 0–10 scale.

/// Whose rating it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Subject {
    /// The user.
    Own,
    Comparable(i64),
}

impl Subject {
    pub fn kind(self) -> &'static str {
        match self {
            Self::Own => "self",
            Self::Comparable(_) => "comparable",
        }
    }

    pub fn comparable_id(self) -> Option<i64> {
        match self {
            Self::Own => None,
            Self::Comparable(id) => Some(id),
        }
    }

    pub fn from_columns(kind: &str, comparable_id: Option<i64>) -> Option<Self> {
        match (kind, comparable_id) {
            ("self", None) => Some(Self::Own),
            ("comparable", Some(id)) => Some(Self::Comparable(id)),
            _ => None,
        }
    }
}

pub const MIN: f64 = 0.0;
pub const MAX: f64 = 10.0;
/// Ratings are entered in half-point increments.
pub const STEP: f64 = 0.5;

#[derive(Debug, Clone, PartialEq)]
pub struct Rating {
    pub subject: Subject,
    pub dimension_id: String,
    pub value: f64,
    /// A generated estimate, not entered by the user.
    pub provisional: bool,
    pub rated_at: String,
}
