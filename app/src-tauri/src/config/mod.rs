//! Configuration management module
//!
//! Handles loading, saving, and validating application configuration.

mod commands;
pub mod defaults;

use defaults::{DEFAULT_CLIPBOARD_LIMIT, DEFAULT_COLOR, DEFAULT_LANGUAGE, DEFAULT_SIZE};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use commands::{
    open_config_folder, open_launcher_config, open_snippets_config, read_config, reload_config,
    save_config,
};

/// Theme configuration
///
/// Defines color and size theme settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Color theme: 'midnight', 'dark', or 'light'
    #[serde(default = "default_color")]
    pub color: String,

    /// Size theme: 'minimal', 'normal', or 'wide'
    #[serde(default = "default_size")]
    pub size: String,

    /// Custom accent color (hex format, e.g., '#3584e4')
    #[serde(
        default,
        rename = "accentColor",
        skip_serializing_if = "Option::is_none"
    )]
    pub accent_color: Option<String>,
}

fn default_color() -> String {
    DEFAULT_COLOR.to_string()
}

fn default_size() -> String {
    DEFAULT_SIZE.to_string()
}

fn default_language() -> String {
    DEFAULT_LANGUAGE.to_string()
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            color: default_color(),
            size: default_size(),
            accent_color: None,
        }
    }
}

impl ThemeConfig {
    /// Validate theme values
    pub fn validate(&mut self) {
        // Validate color
        if !matches!(self.color.as_str(), "midnight" | "dark" | "light") {
            eprintln!(
                "Invalid color theme '{}', falling back to '{}'",
                self.color, DEFAULT_COLOR
            );
            self.color = DEFAULT_COLOR.to_string();
        }

        // Validate size
        if !matches!(self.size.as_str(), "minimal" | "normal" | "wide") {
            eprintln!(
                "Invalid size theme '{}', falling back to '{}'",
                self.size, DEFAULT_SIZE
            );
            self.size = DEFAULT_SIZE.to_string();
        }
    }
}

/// Application configuration
///
/// This struct represents the user's configuration for the uti application.
/// The configuration is stored in `~/.config/uti/config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// Theme configuration
    #[serde(default)]
    pub theme: ThemeConfig,

    /// Maximum number of clipboard items to store
    #[serde(default = "default_clipboard_limit")]
    pub clipboard_history_limit: usize,

    /// UI language: 'en' or 'ja'
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_clipboard_limit() -> usize {
    DEFAULT_CLIPBOARD_LIMIT
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            clipboard_history_limit: default_clipboard_limit(),
            language: default_language(),
        }
    }
}

impl AppConfig {
    /// Get the path to the config file
    ///
    /// Returns `~/.config/uti/config.json`
    pub fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("uti");
        path.push("config.json");
        path
    }

    /// Get the path to the config directory
    ///
    /// Returns `~/.config/uti/`
    pub fn get_config_dir() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("uti");
        path
    }

    /// Validate configuration values
    pub fn validate(&mut self) {
        // Validate theme
        self.theme.validate();

        // Validate clipboard_history_limit
        if self.clipboard_history_limit == 0 {
            eprintln!(
                "clipboard_history_limit cannot be 0, using default ({})",
                DEFAULT_CLIPBOARD_LIMIT
            );
            self.clipboard_history_limit = DEFAULT_CLIPBOARD_LIMIT;
        }

        // Validate language
        if !matches!(self.language.as_str(), "en" | "ja") {
            eprintln!(
                "Invalid language '{}', falling back to '{}'",
                self.language, DEFAULT_LANGUAGE
            );
            self.language = DEFAULT_LANGUAGE.to_string();
        }
    }

    /// Load configuration from file
    ///
    /// If the file doesn't exist or can't be read, returns default config.
    pub fn load() -> Self {
        let path = Self::get_config_path();

        match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<Self>(&contents) {
                Ok(mut config) => {
                    println!("Loaded config from: {:?}", path);
                    config.validate();
                    config
                }
                Err(e) => {
                    eprintln!("Failed to parse config file: {}", e);
                    Self::default()
                }
            },
            Err(_) => {
                println!("Config file not found, using defaults");
                Self::default()
            }
        }
    }

    /// Save configuration to file
    ///
    /// Creates the config directory if it doesn't exist.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_config_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        // Write config file
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&path, contents)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        println!("Config saved to: {:?}", path);
        Ok(())
    }
}
