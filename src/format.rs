//! Number formatting shared by the UI and reports.

/// Formats an integer with comma thousands separators: `1284` → `1,284`.
pub fn thousands(n: impl Into<u128>) -> String {
    let digits = n.into().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_separators() {
        assert_eq!(thousands(0u32), "0");
        assert_eq!(thousands(480u32), "480");
        assert_eq!(thousands(1284u32), "1,284");
        assert_eq!(thousands(1_000_000u32), "1,000,000");
    }
}
