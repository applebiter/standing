//! User settings, persisted as JSON in the platform config directory.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeChoice {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: ThemeChoice,
}

impl Settings {
    /// Loads settings, falling back to defaults if the file is missing or unreadable.
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

/// Why the files are where they are. Shown on the Settings screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    /// `STANDING_HOME` was set.
    Override,
    /// Beside the .AppImage file.
    Portable,
    /// The platform's standard application-data directories.
    System,
}

/// Where Standing keeps its files.
#[derive(Debug, Clone)]
pub struct Paths {
    pub database: PathBuf,
    pub settings: PathBuf,
    pub location: Location,
}

/// Name of the folder created beside a portable copy of the application.
pub const PORTABLE_DIR_NAME: &str = "Standing Data";

impl Paths {
    /// Environment variable naming a directory to use for all files instead
    /// of the platform defaults, e.g. for a separate demonstration profile.
    pub const HOME_VAR: &str = "STANDING_HOME";

    /// Resolves, in order: the `STANDING_HOME` override; a writable
    /// data folder beside the AppImage; the platform's standard directories.
    pub fn discover() -> anyhow::Result<Self> {
        if let Some(home) = std::env::var_os(Self::HOME_VAR).filter(|v| !v.is_empty()) {
            return Ok(Self::in_dir(PathBuf::from(home), Location::Override));
        }
        let appimage = std::env::var_os("APPIMAGE").map(PathBuf::from);
        if let Some(base) = portable_base(appimage.as_deref()) {
            let dir = base.join(PORTABLE_DIR_NAME);
            if is_writable_dir(&dir) {
                return Ok(Self::in_dir(dir, Location::Portable));
            }
        }
        let dirs = directories::ProjectDirs::from("", "", "Standing")
            .ok_or_else(|| anyhow::anyhow!("no home directory is available"))?;
        Ok(Self {
            database: dirs.data_dir().join("standing.sqlite3"),
            settings: dirs.config_dir().join("settings.json"),
            location: Location::System,
        })
    }

    fn in_dir(dir: PathBuf, location: Location) -> Self {
        Self {
            database: dir.join("standing.sqlite3"),
            settings: dir.join("settings.json"),
            location,
        }
    }

    /// The folder holding the database.
    pub fn data_dir(&self) -> &Path {
        self.database.parent().unwrap_or(Path::new("."))
    }
}

/// The folder holding the .AppImage file, when running as one. The AppImage
/// runtime sets $APPIMAGE; current_exe() would point inside the read-only
/// mounted image. `None` otherwise, e.g. for development builds.
fn portable_base(appimage: Option<&Path>) -> Option<PathBuf> {
    appimage?.parent().map(Path::to_owned)
}

/// Creates `dir` if needed and confirms a file can be written in it.
fn is_writable_dir(dir: &Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".write-test");
    let ok = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_file_fills_defaults() {
        let s: Settings = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(s, Settings::default());
        let s: Settings = serde_json::from_str(r#"{"theme":"dark"}"#).unwrap();
        assert_eq!(s.theme, ThemeChoice::Dark);
    }

    #[test]
    fn portable_only_as_appimage() {
        assert_eq!(
            portable_base(Some(Path::new("/media/stick/Standing-x86_64.AppImage"))),
            Some(PathBuf::from("/media/stick"))
        );
        assert_eq!(portable_base(None), None);
    }

    #[test]
    fn writable_dir_is_created_and_probe_removed() {
        let dir = std::env::temp_dir().join(format!("standing-portable-{}", std::process::id()));
        assert!(is_writable_dir(&dir));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
