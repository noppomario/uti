//! Launcher configuration storage
//!
//! Handles loading and saving launcher configuration from JSON file.

use super::LauncherConfig;
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

/// Load launcher configuration from file
///
/// If the file doesn't exist, returns empty configuration.
/// If the file is invalid JSON, logs error and returns empty configuration.
pub fn load_launcher_config() -> LauncherConfig {
    let path = get_launcher_config_path();

    match std::fs::read_to_string(&path) {
        Ok(contents) => match serde_json::from_str::<LauncherConfig>(&contents) {
            Ok(config) => {
                println!("Loaded launcher config from: {:?}", path);
                config
            }
            Err(e) => {
                eprintln!("Failed to parse launcher config: {}", e);
                empty_config()
            }
        },
        Err(_) => {
            println!(
                "Launcher config not found at {:?}. Create it to add launcher commands.",
                path
            );
            empty_config()
        }
    }
}

/// Create empty launcher configuration
fn empty_config() -> LauncherConfig {
    LauncherConfig { commands: vec![] }
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
    fn test_empty_config_has_no_commands() {
        let config = empty_config();
        assert!(config.commands.is_empty());
    }

    #[test]
    fn test_load_launcher_config_returns_valid_config() {
        // This test uses the actual filesystem
        // If the user doesn't have a config file, it should return empty config
        let config = load_launcher_config();
        // Just verify the function doesn't panic and returns a valid LauncherConfig
        let _ = config.commands;
    }
}
