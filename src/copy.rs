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

    pub const REPORTS: (&str, &str) = ("Reports", "Quarterly earnings calls and exports.");
}

pub mod engine {
    pub const FAILED: &str = "The index could not be computed.";
    pub const MARKET_FAILED: &str = "Market hours could not be applied.";
}

pub mod notices {
    use crate::engine::market::Candle;
    use crate::format;

    /// The reason recorded for every Enough Threshold adjustment.
    pub const THRESHOLD_REASON: &str = "Threshold adjusted to reflect current market conditions.";

    pub fn threshold_adjusted(prior: f64, new: f64) -> String {
        format!(
            "Enough Threshold revised from {} to {}. {THRESHOLD_REASON}",
            format::index(prior),
            format::index(new)
        )
    }

    /// The daily summary posted when a market day closes.
    pub fn market_close(day: jiff::civil::Date, candle: &Candle, previous: Option<f64>) -> String {
        let movement = match previous {
            Some(prev) if (candle.close - prev).abs() >= 0.005 => {
                let change = candle.close - prev;
                let pct = if prev.abs() > f64::EPSILON {
                    100.0 * change.abs() / prev.abs()
                } else {
                    0.0
                };
                format!(
                    ", {} {} ({pct:.2}%)",
                    if change > 0.0 { "up" } else { "down" },
                    format::index(change.abs())
                )
            }
            Some(_) => ", unchanged".to_owned(),
            None => String::new(),
        };
        format!(
            "Market close, {}. The Personal Standing Index closed at {}{movement}. \
             Session range {} to {}.",
            format::date_long(day),
            format::index(candle.close),
            format::index(candle.low),
            format::index(candle.high),
        )
    }
}

pub mod dashboard {
    use crate::engine::positions::Volatility;
    use crate::format;

    pub const PSI: &str = "Personal Standing Index";
    pub fn change(change: f64, pct: f64) -> String {
        let arrow = if change >= 0.005 {
            "\u{25B2}"
        } else if change <= -0.005 {
            "\u{25BC}"
        } else {
            "\u{25C6}"
        };
        format!(
            "{arrow} {} ({})",
            format::signed(change),
            format::signed_percent(pct)
        )
    }
    pub const SINCE_CLOSE: &str = "Since previous close";
    pub const SINCE_OPEN: &str = "Since today\u{2019}s open";
    pub const NO_CLOSE: &str = "No previous close on record.";
    pub const COMPOSITE: &str = "Composite standing";
    pub const MOMENTUM: &str = "Momentum";
    pub const THRESHOLD: &str = "Enough Threshold";
    pub const DISTANCE: &str = "Shortfall";

    pub const SPOTLIGHT: &str = "Spotlight Monitor";
    pub const SPOTLIGHT_LABEL: &str = "People currently thinking about you";
    pub fn margin(m: u64) -> String {
        format!("(\u{00B1}{})", format::thousands(m))
    }
    pub fn last_nonzero(at: Option<jiff::Zoned>) -> String {
        match at {
            Some(z) => format!(
                "Last non-zero reading: {}",
                z.strftime("%-d %B %Y, %H:%M:%S")
            ),
            None => "Last non-zero reading: None recorded".to_owned(),
        }
    }
    pub fn reading_taken(at: &jiff::Zoned) -> String {
        format!("Reading taken {}.", at.strftime("%H:%M:%S"))
    }

    pub const HISTORY: &str = "Index history";
    pub const RANGE_MONTH: &str = "1M";
    pub const RANGE_QUARTER: &str = "3M";
    pub const RANGE_YEAR: &str = "1Y";
    pub const RANGE_ALL: &str = "All";
    pub const NO_HISTORY: &str = "No market history on record.";
    pub const THRESHOLD_LINE: &str = "Enough Threshold";
    pub const OPEN: &str = "Open";
    pub const HIGH: &str = "High";
    pub const LOW: &str = "Low";
    pub const CLOSE: &str = "Close";

    pub const POSITIONS: &str = "Positions Held";
    pub const NO_POSITIONS: &str = "No positions currently held.";
    pub fn volatility(v: Volatility) -> &'static str {
        match v {
            Volatility::Low => "Low volatility",
            Volatility::Moderate => "Moderate volatility",
            Volatility::High => "High volatility",
            Volatility::Extreme => "Extreme volatility",
        }
    }
    pub fn more_positions(n: usize) -> String {
        format!("And {} more.", format::thousands(n as u64))
    }

    pub const PEERS: &str = "Comparables";
    pub const COL_CHANGE: &str = "Change";
    pub const NOTICES: &str = "Notices";
    pub const NO_NOTICES: &str = "No notices.";
    pub const LOAD_FAILED: &str = "The dashboard could not be loaded.";
}

pub mod filing {
    use crate::format;
    use crate::model::event::EventKind;

    pub const TITLE: &str = "Daily Filing";
    pub const SUBTITLE: &str = "Status events, recorded as they occur.";
    pub const FILE_AN_EVENT: &str = "File an event";
    pub const GROUP_OWN: &str = "Concerning you";
    pub const GROUP_OBSERVED: &str = "Observed of a Comparable. Applied to their ticker.";
    pub const OBSERVED_NEEDS_COMPARABLES: &str =
        "Observations require at least one Comparable in the peer set.";
    pub const COMPARABLE: &str = "Comparable";
    pub const CHOOSE_COMPARABLE: &str = "Select a Comparable";
    pub const COMPARABLE_REQUIRED: &str = "Select the Comparable observed.";
    pub const NOTE: &str = "Note";
    pub const NOTE_HINT: &str = "e.g. at the barbecue";
    pub const SUBMIT: &str = "File";

    pub fn label(kind: EventKind) -> &'static str {
        match kind {
            EventKind::ComplimentReceived => "Compliment received",
            EventKind::InvitationReceived => "Invitation received",
            EventKind::NotInvited => "Not invited (learned of after the fact)",
            EventKind::TextUnanswered => "Text left unanswered",
            EventKind::Achievement => "Achievement",
            EventKind::PublicRecognition => "Public recognition",
            EventKind::SlightMinor => "Slight (minor)",
            EventKind::SlightMajor => "Slight (major)",
            EventKind::ComparableAchievement => "Observed Comparable achievement",
            EventKind::ComparableThriving => "Comparable seen to be thriving",
        }
    }

    pub fn description(kind: EventKind) -> &'static str {
        match kind {
            EventKind::ComplimentReceived => "A favorable remark about you, made to you.",
            EventKind::InvitationReceived => "An invitation to a gathering, event or occasion.",
            EventKind::NotInvited => "An occasion you were not invited to, learned of afterward.",
            EventKind::TextUnanswered => "A message you sent that has not received a reply.",
            EventKind::Achievement => "A milestone reached or an objective completed.",
            EventKind::PublicRecognition => "Acknowledgment of you made in front of others.",
            EventKind::SlightMinor => "A small discourtesy or omission.",
            EventKind::SlightMajor => "A significant discourtesy or omission.",
            EventKind::ComparableAchievement => {
                "An achievement by a Comparable, as observed. Raises their ticker."
            }
            EventKind::ComparableThriving => {
                "Evidence that a Comparable is doing well. Raises their ticker."
            }
        }
    }

    pub fn filed(kind: EventKind) -> String {
        format!(
            "Filed: {} ({}).",
            label(kind),
            format::signed(kind.weight())
        )
    }

    pub const MOMENTUM: &str = "Momentum";
    pub const FILED_TODAY: &str = "Filed today";
    pub const MOMENTUM_NOTE: &str =
        "Each event's contribution halves every seven days. See the Methodology.";
    pub const RECENT: &str = "Filings, last 30 days";
    pub const NO_FILINGS: &str = "No filings in the last 30 days.";
    pub const COL_FILED: &str = "Filed";
    pub const COL_EVENT: &str = "Event";
    pub const COL_WEIGHT: &str = "Weight";
    pub const COL_CURRENT: &str = "Now";

    pub const WITHDRAW: &str = "Withdraw";
    pub const WITHDRAW_TITLE: &str = "Withdraw this filing?";
    pub fn withdraw_body(kind: EventKind) -> String {
        format!(
            "{} will be removed from the record and the index recomputed without it.",
            label(kind)
        )
    }
    pub fn withdrawn(kind: EventKind) -> String {
        format!("Filing withdrawn: {}.", label(kind))
    }

    pub const LOAD_FAILED: &str = "Filings could not be loaded.";
    pub const WRITE_FAILED: &str = "The filing could not be recorded.";
}

pub mod threshold {
    use crate::db::threshold::Reason;

    pub const TITLE: &str = "Enough Threshold";
    pub const SUBTITLE: &str = "The index level at which you will feel secure.";
    pub const CURRENT: &str = "Current threshold";
    pub fn set_on(date: &str) -> String {
        format!("In effect since {date}.")
    }
    pub const NOT_SET: &str = "No threshold set.";
    pub const NOT_SET_BODY: &str =
        "Enter the Personal Standing Index value at which you will feel secure.";
    pub const PSI: &str = "Personal Standing Index";
    pub const SHORTFALL: &str = "Shortfall";
    pub const REVISE: &str = "Revise threshold";
    pub const SET: &str = "Set threshold";
    pub const FIELD_HELP: &str = "An index value from 1 to 10,000.";
    pub const SAVE: &str = "Save";
    pub const SAVED: &str = "Threshold saved.";
    pub const UNCHANGED: &str = "That is the current threshold.";

    pub const HISTORY: &str = "Threshold History";
    pub const NO_HISTORY: &str = "No changes recorded.";
    pub const COL_DATE: &str = "Date";
    pub const COL_PRIOR: &str = "Prior";
    pub const COL_NEW: &str = "New";
    pub const COL_REASON: &str = "Reason";
    pub fn reason(r: Reason) -> &'static str {
        match r {
            Reason::Initial => "Initial threshold.",
            Reason::User => "Set by user.",
            Reason::Adjustment => super::notices::THRESHOLD_REASON,
        }
    }
    pub const LOAD_FAILED: &str = "The threshold could not be loaded.";
    pub const WRITE_FAILED: &str = "The threshold could not be saved.";
}

pub mod form {
    use crate::model::FieldError;

    pub const OPTIONAL: &str = "Optional";
    pub const CANCEL: &str = "Cancel";

    pub fn error(e: FieldError) -> &'static str {
        match e {
            FieldError::Required => "This field is required.",
            FieldError::TooLong => "Enter 120 characters or fewer.",
            FieldError::NotANumber => "Enter a number.",
            FieldError::ThresholdRange => "Enter a value from 1 to 10,000.",
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
    pub const COL_INDEX: &str = "Index";
    pub const COL_SPREAD: &str = "Your spread";
    pub const SPREAD_HINT: &str =
        "Your standing relative to this Comparable, in index points. Negative when they lead.";
    pub const COL_COVERAGE: &str = "Coverage";
    pub const COL_LAST_RATED: &str = "Last rated";
    pub const COL_STATUS: &str = "Status";
    pub const STATUS_ACTIVE: &str = "In peer set";
    pub const STATUS_INACTIVE: &str = "Removed";
    pub const NOT_RATED: &str = "Not rated";
    pub const NO_VALUE: &str = "\u{2014}";

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

#[cfg(test)]
mod tests {
    use crate::engine::market::Candle;

    #[test]
    fn market_close_summaries() {
        let day = jiff::civil::date(2026, 10, 16);
        let candle = Candle {
            day,
            open: 98.0,
            high: 98.3,
            low: 96.1,
            close: 97.42,
        };
        assert_eq!(
            super::notices::market_close(day, &candle, Some(98.65)),
            "Market close, 16 October 2026. The Personal Standing Index closed at 97.42, \
             down 1.23 (1.25%). Session range 96.10 to 98.30."
        );
        assert_eq!(
            super::notices::market_close(day, &candle, Some(97.42)),
            "Market close, 16 October 2026. The Personal Standing Index closed at 97.42, \
             unchanged. Session range 96.10 to 98.30."
        );
        assert_eq!(
            super::notices::market_close(day, &candle, None),
            "Market close, 16 October 2026. The Personal Standing Index closed at 97.42. \
             Session range 96.10 to 98.30."
        );
    }
}
