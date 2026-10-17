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
    NotANumber,
    /// Outside the range a threshold may take.
    ThresholdRange,
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

/// Parses a threshold as typed: a number from the minimum to the maximum,
/// with up to two decimal places. Thousands separators are accepted.
pub fn threshold(value: &str) -> Result<f64, FieldError> {
    use crate::engine::constants::threshold::{MAXIMUM, MINIMUM};
    let cleaned: String = value.trim().chars().filter(|c| *c != ',').collect();
    if cleaned.is_empty() {
        return Err(FieldError::Required);
    }
    let v: f64 = cleaned.parse().map_err(|_| FieldError::NotANumber)?;
    if !v.is_finite() {
        return Err(FieldError::NotANumber);
    }
    if !(MINIMUM..=MAXIMUM).contains(&v) {
        return Err(FieldError::ThresholdRange);
    }
    Ok((v * 100.0).round() / 100.0)
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

    #[test]
    fn threshold_input() {
        assert_eq!(threshold(" 120 "), Ok(120.0));
        assert_eq!(threshold("1,250.5"), Ok(1250.5));
        assert_eq!(threshold("110.456"), Ok(110.46));
        assert_eq!(threshold(""), Err(FieldError::Required));
        assert_eq!(threshold("enough"), Err(FieldError::NotANumber));
        assert_eq!(threshold("NaN"), Err(FieldError::NotANumber));
        assert_eq!(threshold("inf"), Err(FieldError::NotANumber));
        assert_eq!(threshold("0.5"), Err(FieldError::ThresholdRange));
        assert_eq!(threshold("-10"), Err(FieldError::ThresholdRange));
        assert_eq!(threshold("10000.01"), Err(FieldError::ThresholdRange));
        assert_eq!(threshold("1"), Ok(1.0));
    }
}
