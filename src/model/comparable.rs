//! Comparables: the people the user measures themselves against.

use super::{FieldErrors, optional_bounded, required};

/// The ticker of the user's own index. Never issued to a Comparable.
pub const SELF_TICKER: &str = "PSI";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Entered by the user.
    User,
    /// Introduced by Peer Set Rebalancing.
    Archetype,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Archetype => "archetype",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "user" => Some(Self::User),
            "archetype" => Some(Self::Archetype),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comparable {
    pub id: i64,
    pub name: String,
    pub relationship: Option<String>,
    pub ticker: String,
    pub origin: Origin,
    pub archetype_template: Option<String>,
    pub claimed_at: Option<String>,
    /// In the peer set. Inactive Comparables keep their history.
    pub active: bool,
    pub created_at: String,
}

/// Validated, editable fields.
#[derive(Debug, Clone, PartialEq)]
pub struct ComparableFields {
    pub name: String,
    pub relationship: Option<String>,
}

/// Form fields, as typed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComparableDraft {
    pub name: String,
    pub relationship: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Name,
    Relationship,
}

impl ComparableDraft {
    pub fn from_comparable(c: &Comparable) -> Self {
        Self {
            name: c.name.clone(),
            relationship: c.relationship.clone().unwrap_or_default(),
        }
    }

    pub fn validate(&self) -> Result<ComparableFields, FieldErrors<Field>> {
        let mut errors = Vec::new();
        let name = required(&self.name)
            .map_err(|e| errors.push((Field::Name, e)))
            .ok();
        let relationship = optional_bounded(&self.relationship)
            .map_err(|e| errors.push((Field::Relationship, e)))
            .ok();
        match (name, relationship) {
            (Some(name), Some(relationship)) => Ok(ComparableFields { name, relationship }),
            _ => Err(errors),
        }
    }
}

/// Issues a ticker for a new Comparable: the first four letters of a
/// one-word name, otherwise the initials of up to four words, uppercased.
/// A numeral is appended if the symbol is already taken.
pub fn issue_ticker<'a>(name: &str, taken: impl IntoIterator<Item = &'a str>) -> String {
    let words: Vec<String> = name
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_uppercase())
        .collect();
    let mut base: String = match words.as_slice() {
        [] => "CMP".to_owned(),
        [one] => one.chars().take(4).collect(),
        many => many
            .iter()
            .take(4)
            .filter_map(|w| w.chars().next())
            .collect(),
    };
    if base.chars().count() < 2 {
        base = words.concat().chars().take(4).collect();
        if base.chars().count() < 2 {
            base = "CMP".to_owned();
        }
    }
    let taken: std::collections::HashSet<&str> = taken.into_iter().collect();
    let free = |t: &str| t != SELF_TICKER && !taken.contains(t);
    if free(&base) {
        return base;
    }
    (2..)
        .map(|n| format!("{base}{n}"))
        .find(|t| free(t))
        .expect("an unused ticker exists")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tickers_from_names() {
        assert_eq!(issue_ticker("Dana", []), "DANA");
        assert_eq!(issue_ticker("Bo", []), "BO");
        assert_eq!(issue_ticker("Christopher", []), "CHRI");
        assert_eq!(issue_ticker("Mark from accounting", []), "MFA");
        assert_eq!(issue_ticker("Your sister's new partner", []), "YSSN");
        assert_eq!(issue_ticker("Zoë", []), "ZOË");
        assert_eq!(issue_ticker("J. R.", []), "JR");
        assert_eq!(issue_ticker("42", []), "CMP");
    }

    #[test]
    fn tickers_are_unique_and_never_psi() {
        assert_eq!(issue_ticker("Dana", ["DANA"]), "DANA2");
        assert_eq!(issue_ticker("Dana", ["DANA", "DANA2"]), "DANA3");
        assert_eq!(issue_ticker("Psi", []), "PSI2");
        assert_eq!(issue_ticker("Paul Scott Ives", []), "PSI2");
    }

    #[test]
    fn draft_requires_a_name() {
        let draft = ComparableDraft {
            name: "  ".into(),
            relationship: "brother-in-law".into(),
        };
        assert_eq!(
            draft.validate().unwrap_err(),
            vec![(Field::Name, crate::model::FieldError::Required)]
        );
        let draft = ComparableDraft {
            name: " Dana ".into(),
            relationship: String::new(),
        };
        assert_eq!(
            draft.validate().unwrap(),
            ComparableFields {
                name: "Dana".into(),
                relationship: None
            }
        );
    }
}
