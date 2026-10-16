//! The user's own profile. There is exactly one.

use super::{FieldErrors, optional_bounded};

#[derive(Debug, Clone, PartialEq)]
pub struct UserProfile {
    pub display_name: String,
    /// The user's line of work.
    pub field: Option<String>,
    /// Seeds every generated figure, so results are reproducible.
    pub rng_seed: i64,
    pub created_at: String,
    pub onboarding_complete: bool,
}

/// Editable profile fields, as typed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProfileDraft {
    pub display_name: String,
    pub field: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileField {
    DisplayName,
    Field,
}

impl ProfileDraft {
    pub fn from_profile(p: &UserProfile) -> Self {
        Self {
            display_name: p.display_name.clone(),
            field: p.field.clone().unwrap_or_default(),
        }
    }

    /// Display name (required) and field (optional), trimmed.
    pub fn validate(&self) -> Result<(String, Option<String>), FieldErrors<ProfileField>> {
        let mut errors = Vec::new();
        let name = super::required(&self.display_name)
            .map_err(|e| errors.push((ProfileField::DisplayName, e)))
            .ok();
        let field = optional_bounded(&self.field)
            .map_err(|e| errors.push((ProfileField::Field, e)))
            .ok();
        match (name, field) {
            (Some(n), Some(f)) => Ok((n, f)),
            _ => Err(errors),
        }
    }
}
