use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RowDisplay {
    Compact,
    #[default]
    Normal,
    Comfortable,
}

impl RowDisplay {
    pub fn as_str(&self) -> &'static str {
        match self {
            RowDisplay::Compact => "Compact",
            RowDisplay::Normal => "Normal",
            RowDisplay::Comfortable => "Comfortable",
        }
    }

    pub fn cycle_next(&self) -> Self {
        match self {
            RowDisplay::Compact => RowDisplay::Normal,
            RowDisplay::Normal => RowDisplay::Comfortable,
            RowDisplay::Comfortable => RowDisplay::Compact,
        }
    }

    pub fn cycle_prev(&self) -> Self {
        match self {
            RowDisplay::Compact => RowDisplay::Comfortable,
            RowDisplay::Normal => RowDisplay::Compact,
            RowDisplay::Comfortable => RowDisplay::Normal,
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub api_key: Option<String>,
    pub api_url: String,
    pub row_display: RowDisplay,
    pub refresh_interval_secs: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        let api_key = Self::load_api_key();
        let api_url = "https://www.moltbook.com/api/v1".to_string();

        // Load settings from config file
        let (row_display, refresh_interval_secs) = Self::load_settings();

        Ok(Self {
            api_key,
            api_url,
            row_display,
            refresh_interval_secs,
        })
    }

    fn load_api_key() -> Option<String> {
        // First, check environment variable
        if let Ok(key) = std::env::var("MOLTBOOK_API_KEY") {
            if !key.is_empty() {
                return Some(key);
            }
        }

        // Then, check config file
        if let Some(config_path) = Self::config_file_path() {
            if config_path.exists() {
                if let Ok(contents) = fs::read_to_string(&config_path) {
                    for line in contents.lines() {
                        let line = line.trim();
                        if line.starts_with("api_key") {
                            if let Some(value) = line.split('=').nth(1) {
                                let key = value.trim().trim_matches('"').trim_matches('\'');
                                if !key.is_empty() {
                                    return Some(key.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn load_settings() -> (RowDisplay, u64) {
        let mut row_display = RowDisplay::default();
        let mut refresh_interval_secs = 10u64;

        if let Some(config_path) = Self::config_file_path() {
            if config_path.exists() {
                if let Ok(contents) = fs::read_to_string(&config_path) {
                    for line in contents.lines() {
                        let line = line.trim();
                        if line.starts_with("row_display") {
                            if let Some(value) = line.split('=').nth(1) {
                                let value = value.trim().trim_matches('"').trim_matches('\'');
                                row_display = match value {
                                    "compact" => RowDisplay::Compact,
                                    "comfortable" => RowDisplay::Comfortable,
                                    _ => RowDisplay::Normal,
                                };
                            }
                        } else if line.starts_with("refresh_interval_secs") {
                            if let Some(value) = line.split('=').nth(1) {
                                if let Ok(secs) = value.trim().parse::<u64>() {
                                    refresh_interval_secs = secs;
                                }
                            }
                        }
                    }
                }
            }
        }

        (row_display, refresh_interval_secs)
    }

    fn config_file_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".moltbook-tui").join("config.toml"))
    }

    pub fn save(api_key: &str) -> Result<Self> {
        // Load existing settings to preserve them
        let (row_display, refresh_interval_secs) = Self::load_settings();

        let config_path = Self::config_file_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        // Write config file with all settings
        let row_display_str = match row_display {
            RowDisplay::Compact => "compact",
            RowDisplay::Normal => "normal",
            RowDisplay::Comfortable => "comfortable",
        };
        let content = format!(
            "api_key = \"{}\"\nrow_display = \"{}\"\nrefresh_interval_secs = {}",
            api_key, row_display_str, refresh_interval_secs
        );
        fs::write(&config_path, &content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        let api_url = "https://www.moltbook.com/api/v1".to_string();

        Ok(Self {
            api_key: Some(api_key.to_string()),
            api_url,
            row_display,
            refresh_interval_secs,
        })
    }

    pub fn save_settings(row_display: RowDisplay, refresh_interval_secs: u64) -> Result<()> {
        let config_path = Self::config_file_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        // Load existing api_key if present
        let api_key = Self::load_api_key();

        // Write config file with all settings
        let row_display_str = match row_display {
            RowDisplay::Compact => "compact",
            RowDisplay::Normal => "normal",
            RowDisplay::Comfortable => "comfortable",
        };

        let content = if let Some(key) = api_key {
            format!(
                "api_key = \"{}\"\nrow_display = \"{}\"\nrefresh_interval_secs = {}",
                key, row_display_str, refresh_interval_secs
            )
        } else {
            format!(
                "row_display = \"{}\"\nrefresh_interval_secs = {}",
                row_display_str, refresh_interval_secs
            )
        };

        fs::write(&config_path, &content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }
}
