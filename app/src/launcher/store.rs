//! Launcher configuration storage
//!
//! Handles loading and saving launcher configuration from JSON file.

use super::{LauncherConfig, LauncherNode};
use std::path::PathBuf;

/// Get the path to the launcher configuration file
///
/// Returns `~/.config/uti/launcher.json`
pub fn get_launcher_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("uti")
        .join("launcher.json")
}

/// The launcher tree from `launcher.json` (empty when missing or invalid).
pub fn load_launcher_tree() -> Vec<LauncherNode> {
    load_launcher_config().into_tree()
}

/// Load launcher configuration from file
///
/// If the file doesn't exist, returns empty configuration.
/// If the file is invalid JSON, logs error and returns empty configuration.
fn load_launcher_config() -> LauncherConfig {
    let path = get_launcher_config_path();

    match std::fs::read_to_string(&path) {
        Ok(contents) => match serde_json::from_str::<LauncherConfig>(&contents) {
            Ok(config) => {
                println!("Loaded launcher config from: {:?}", path);
                config
            },
            Err(e) => {
                eprintln!("Failed to parse launcher config: {}", e);
                empty_config()
            },
        },
        Err(_) => {
            println!("Launcher config not found at {:?}; showing the default tree.", path);
            empty_config()
        },
    }
}

fn empty_config() -> LauncherConfig {
    LauncherConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_launcher_config_path() {
        let path = get_launcher_config_path();
        assert!(path.ends_with("launcher.json"));
        assert!(path.to_string_lossy().contains("uti"));
    }

    #[test]
    fn missing_config_shows_the_default_tree() {
        assert_eq!(empty_config().into_tree().len(), 3);
    }
}
