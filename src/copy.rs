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
    pub const DAILY_FILING: (&str, &str) =
        ("Daily Filing", "Status events for the current market day.");
    pub const THRESHOLD: (&str, &str) = (
        "Enough Threshold",
        "The index level at which you will feel secure.",
    );
    pub const REPORTS: (&str, &str) = ("Reports", "Quarterly earnings calls and exports.");
}

pub mod form {
    use crate::model::FieldError;

    pub const OPTIONAL: &str = "Optional";
    pub const CANCEL: &str = "Cancel";

    pub fn error(e: FieldError) -> &'static str {
        match e {
            FieldError::Required => "This field is required.",
            FieldError::TooLong => "Enter 120 characters or fewer.",
        }
    }
}

pub mod comparables {
    pub const TITLE: &str = "Comparables";
    pub const SUBTITLE: &str = "The people against whom your standing is measured.";
    pub const ADD: &str = "Add a Comparable";
    pub const SHOW_INACTIVE: &str = "Include former Comparables";

    pub const COL_TICKER: &str = "Ticker";
    pub const COL_NAME: &str = "Name";
    pub const COL_RELATIONSHIP: &str = "Relationship";
    pub const COL_COVERAGE: &str = "Coverage";
    pub const COL_LAST_RATED: &str = "Last rated";
    pub const COL_STATUS: &str = "Status";
    pub const STATUS_ACTIVE: &str = "In peer set";
    pub const STATUS_INACTIVE: &str = "Removed";
    pub const NOT_RATED: &str = "Not rated";

    pub fn coverage(rated: usize, total: usize) -> String {
        format!(
            "{} of {}",
            crate::format::thousands(rated as u64),
            crate::format::thousands(total as u64)
        )
    }

    pub const EMPTY_TITLE: &str = "No Comparables on record.";
    pub const EMPTY_BODY: &str =
        "Add the people you compare yourself to. The index requires at least one.";

    pub fn added(name: &str, ticker: &str) -> String {
        format!("{name} added to the peer set as {ticker}.")
    }
    pub fn amended(name: &str) -> String {
        format!("Record for {name} amended.")
    }
    pub fn removed(name: &str) -> String {
        format!("{name} removed from the peer set. History is retained.")
    }
    pub fn restored(name: &str) -> String {
        format!("{name} restored to the peer set.")
    }
    pub fn deleted(name: &str) -> String {
        format!("{name} and all associated records deleted.")
    }
    pub const LOAD_FAILED: &str = "Comparables could not be loaded.";
    pub const WRITE_FAILED: &str = "The change could not be saved.";

    pub const DELETE_TITLE: &str = "Delete this Comparable?";
    pub fn delete_body(name: &str) -> String {
        format!(
            "{name}, with every rating, event and index value recorded for them, will be \
             permanently deleted. To keep the history, remove them from the peer set instead."
        )
    }
    pub const DELETE_CONFIRM: &str = "Delete permanently";
}

pub mod comparable_form {
    pub const NEW_TITLE: &str = "Add a Comparable";
    pub const NEW_SUBTITLE: &str =
        "A person against whom your standing will be measured. A ticker is issued on entry.";
    pub const EDIT_TITLE: &str = "Comparable";
    pub fn edit_subtitle(added: &str) -> String {
        format!("Added {added}.")
    }

    pub const NAME: &str = "Name";
    pub const NAME_HELP: &str = "Real name or nickname. This record does not leave your computer.";
    pub const RELATIONSHIP: &str = "Relationship";
    pub const RELATIONSHIP_HINT: &str = "e.g. college roommate, brother-in-law";

    pub const SUBMIT_NEW: &str = "Add to peer set";
    pub const SUBMIT_EDIT: &str = "Save changes";
    pub const REMOVE: &str = "Remove from peer set";
    pub const RESTORE: &str = "Restore to peer set";
    pub const DELETE: &str = "Delete permanently";
    pub const INCOMPLETE: &str = "The record is incomplete. See the fields marked above.";
}

pub mod ratings {
    pub const TITLE: &str = "Ratings";
    pub const SUBTITLE: &str = "Assessments of yourself and your Comparables, by dimension.";
    pub const SEARCH_HINT: &str = "Jump to a dimension";
    pub const SELF_LABEL: &str = "You";
    pub const NOT_RATED: &str = "Not rated";
    pub const PROVISIONAL: &str = "(provisional estimate)";
    pub fn search_results(n: usize) -> String {
        match n {
            0 => "No applicable dimensions match.".to_owned(),
            1 => "1 dimension".to_owned(),
            n => format!("{} dimensions", crate::format::thousands(n as u64)),
        }
    }
    pub fn rated(rated: usize, total: usize) -> String {
        format!("{rated}/{total}")
    }
    pub const NO_COMPARABLES: &str =
        "No Comparables on record. Add them to rate them alongside yourself.";
    pub const NO_DIMENSIONS: &str =
        "No dimensions currently apply. Applicability is set in the Dimension Catalog.";
    pub const LOAD_FAILED: &str = "Ratings could not be loaded.";
    pub const WRITE_FAILED: &str = "The rating could not be recorded.";
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
    pub const SUBTITLE: &str = "Profile, display and storage preferences.";
    pub const PROFILE: &str = "Profile";
    pub const DISPLAY_NAME: &str = "Display name";
    pub const FIELD: &str = "Field";
    pub const FIELD_HELP: &str = "Your line of work. Used to identify relevant peers.";
    pub const SAVE_PROFILE: &str = "Save profile";
    pub const PROFILE_SAVED: &str = "Profile saved.";
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
