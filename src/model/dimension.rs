//! Comparison dimensions, as classified by the SCCD.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionState {
    /// In the catalog from the start.
    Active,
    /// Held back until revealed by Dimension Discovery.
    Hidden,
    /// Revealed by Dimension Discovery.
    Discovered,
}

impl DimensionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Hidden => "hidden",
            Self::Discovered => "discovered",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "active" => Some(Self::Active),
            "hidden" => Some(Self::Hidden),
            "discovered" => Some(Self::Discovered),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dimension {
    /// Stable identifier, e.g. `PHY-017`. Never reused or renumbered.
    pub id: String,
    pub sector: String,
    pub industry: String,
    pub name: String,
    pub definition: String,
    /// The user's "Applies to me" toggle.
    pub applies: bool,
    pub state: DimensionState,
    pub weight: f64,
    pub discovered_at: Option<String>,
}

impl Dimension {
    /// Whether the dimension counts toward the index: applicable and not hidden.
    pub fn is_active(&self) -> bool {
        self.applies && self.state != DimensionState::Hidden
    }
}
