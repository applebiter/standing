//! Domain types. Free of egui and storage.

pub mod comparable;
pub mod dimension;
pub mod event;
pub mod rating;
pub mod user;

/// Why a form field was rejected. Shared by every form; the wording lives in `copy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldError {
    Required,
    TooLong,
}

/// Field errors from validating a form, in form order.
pub type FieldErrors<F> = Vec<(F, FieldError)>;

/// Trims surrounding whitespace; empty becomes `None`.
pub fn optional(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/// Longest name or note accepted by any single-line field.
pub const MAX_FIELD_CHARS: usize = 120;

/// Validates a required single-line field.
pub fn required(value: &str) -> Result<String, FieldError> {
    let value = value.trim();
    if value.is_empty() {
        Err(FieldError::Required)
    } else if value.chars().count() > MAX_FIELD_CHARS {
        Err(FieldError::TooLong)
    } else {
        Ok(value.to_owned())
    }
}

/// Validates an optional single-line field.
pub fn optional_bounded(value: &str) -> Result<Option<String>, FieldError> {
    match optional(value) {
        Some(v) if v.chars().count() > MAX_FIELD_CHARS => Err(FieldError::TooLong),
        v => Ok(v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_validation() {
        assert_eq!(required("  Dana "), Ok("Dana".to_owned()));
        assert_eq!(required("   "), Err(FieldError::Required));
        assert_eq!(required(&"x".repeat(121)), Err(FieldError::TooLong));
        assert_eq!(optional_bounded(" "), Ok(None));
        assert_eq!(
            optional_bounded(" college roommate "),
            Ok(Some("college roommate".to_owned()))
        );
    }
}
