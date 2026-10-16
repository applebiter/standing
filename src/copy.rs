//! Every user-facing string in Standing.
//!
//! Tone: institutional, exact, unhurried. Match the wording of the existing
//! strings when adding or changing one.

pub const APP_NAME: &str = "Standing";

/// Edition of the Standard Classification of Comparison Dimensions.
pub const SCCD_VERSION: &str = "4.2";

pub mod nav {
    pub const DASHBOARD: &str = "Dashboard";
    pub const COMPARABLES: &str = "Comparables";
    pub const RATINGS: &str = "Ratings";
    pub const DAILY_FILING: &str = "Daily Filing";
    pub const CATALOG: &str = "Dimension Catalog";
    pub const THRESHOLD: &str = "Enough Threshold";
    pub const REPORTS: &str = "Reports";
    pub const SETTINGS: &str = "Settings";
    pub const SECTION_MARKET: &str = "Market";
    pub const SECTION_REFERENCE: &str = "Reference";
    pub const SECTION_SYSTEM: &str = "System";
}

pub mod status {
    pub const DATABASE_OK: &str = "Database operational";
    pub const SCHEMA_VERSION: &str = "Schema version";
    pub const BUILD: &str = "Build";
}

/// Titles and descriptions for screens whose modules are not yet installed.
pub mod pending {
    pub const BODY: &str = "This module is not installed in the current build.";

    pub const DASHBOARD: (&str, &str) =
        ("Dashboard", "Personal Standing Index and market summary.");
    pub const COMPARABLES: (&str, &str) = (
        "Comparables",
        "Your peer set, with spreads and recent activity.",
    );
    pub const RATINGS: (&str, &str) = (
        "Ratings",
        "Assessments of yourself and your Comparables, by dimension.",
    );
    pub const DAILY_FILING: (&str, &str) =
        ("Daily Filing", "Status events for the current market day.");
    pub const THRESHOLD: (&str, &str) = (
        "Enough Threshold",
        "The index level at which you will feel secure.",
    );
    pub const REPORTS: (&str, &str) = ("Reports", "Quarterly earnings calls and exports.");
}

pub mod catalog {
    pub const TITLE: &str = "Dimension Catalog";
    pub fn subtitle() -> String {
        format!(
            "Standard Classification of Comparison Dimensions (SCCD), version {}.",
            super::SCCD_VERSION
        )
    }

    pub const ACTIVE_DIMENSIONS: &str = "Active dimensions";
    pub const SECTORS: &str = "Sectors";
    pub const INDUSTRIES: &str = "Industries";

    pub const SEARCH_HINT: &str = "Search name, definition, industry or ID";
    pub const ALL_SECTORS: &str = "All sectors";
    pub const STATUS_ALL: &str = "All dimensions";
    pub const STATUS_APPLIES: &str = "Applies to me";
    pub const STATUS_EXCLUDED: &str = "Does not apply";
    pub const STATUS_DISCOVERED: &str = "Discovered";
    pub const CLEAR_FILTERS: &str = "Clear filters";
    pub fn showing(shown: usize, total: usize) -> String {
        format!(
            "Showing {} of {} dimensions",
            crate::format::thousands(shown as u64),
            crate::format::thousands(total as u64)
        )
    }
    pub const NO_MATCHES: &str = "No dimensions match these filters.";

    pub const COL_ID: &str = "ID";
    pub const COL_DIMENSION: &str = "Dimension";
    pub const COL_INDUSTRY: &str = "Industry";
    pub const COL_SECTOR: &str = "Sector";
    pub const COL_WEIGHT: &str = "Weight";
    pub const COL_APPLIES: &str = "Applies";
    pub const APPLIES_HINT: &str = "Applies to me";
    pub const DISCOVERED_TAG: &str = "Discovered";

    pub const LOAD_FAILED: &str = "The catalog could not be loaded.";
    pub const SAVE_FAILED: &str = "The change could not be saved.";
}

pub mod settings {
    pub const TITLE: &str = "Settings";
    pub const SUBTITLE: &str = "Display and storage preferences.";
    pub const APPEARANCE: &str = "Appearance";
    pub const THEME: &str = "Theme";
    pub const THEME_SYSTEM: &str = "System";
    pub const THEME_LIGHT: &str = "Light";
    pub const THEME_DARK: &str = "Dark";
    pub const STORAGE: &str = "Storage";
    pub const STORAGE_LOCATION: &str = "Profile location";
    pub const STORAGE_OVERRIDE: &str = "Set by the STANDING_HOME environment variable.";
    pub const STORAGE_PORTABLE: &str =
        "Portable. The profile is kept beside the application and travels with it.";
    pub const STORAGE_SYSTEM: &str =
        "Standard. The profile is kept in this computer\u{2019}s application-data folder.";
    pub const SAVE_FAILED: &str = "Settings could not be saved.";
}

pub mod startup {
    pub const TITLE: &str = "The profile could not be opened.";
    pub const BODY: &str = "Standing was unable to open its database. No data has been modified. \
                            The details below may assist with diagnosis.";
}
