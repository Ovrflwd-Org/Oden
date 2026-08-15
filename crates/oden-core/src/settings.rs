use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::errors::SettingsError;

const DEFAULT_THEME: &str = "Gruvbox Dark";

/// User-configurable application settings, persisted to disk as JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub theme: String,
    pub check_for_updates: bool,
    pub last_seen_version: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: DEFAULT_THEME.to_string(),
            check_for_updates: true,
            last_seen_version: String::new(),
        }
    }
}

fn get_settings_path() -> Result<PathBuf, SettingsError> {
    let project_dirs =
        ProjectDirs::from("com", "outoforder", "oden").ok_or(SettingsError::MissingProjectDirs)?;
    let config_dir = project_dirs.config_dir();
    fs::create_dir_all(config_dir).map_err(SettingsError::CreateDir)?;
    Ok(config_dir.join("settings.json"))
}

impl AppSettings {
    pub fn load() -> Result<Self, SettingsError> {
        let path = get_settings_path()?;
        if !path.exists() {
            tracing::debug!(path = %path.display(), "no settings file yet, using defaults");
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path).map_err(SettingsError::Read)?;
        serde_json::from_str(&contents).map_err(SettingsError::Parse)
    }

    pub fn save(&self) -> Result<(), SettingsError> {
        let path = get_settings_path()?;
        let contents = serde_json::to_string_pretty(self).map_err(SettingsError::Serialize)?;
        fs::write(&path, contents).map_err(SettingsError::Write)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_stable() {
        let settings = AppSettings::default();
        assert_eq!(settings.theme, DEFAULT_THEME);
        assert!(settings.check_for_updates);
    }

    #[test]
    fn round_trips_through_json() {
        let settings = AppSettings {
            theme: "Ayu".to_string(),
            check_for_updates: false,
            last_seen_version: "0.7.1".to_string(),
        };
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, parsed);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let parsed: AppSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(parsed, AppSettings::default());
    }
}
