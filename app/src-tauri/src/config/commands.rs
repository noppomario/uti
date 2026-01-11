//! Tauri commands for configuration management

use super::AppConfig;

/// Reads the application configuration
///
/// Returns the user's configuration from `~/.config/uti/config.json`.
/// If the file doesn't exist, returns default configuration.
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// const config = await invoke('read_config');
/// console.log(config.theme.color); // 'midnight' | 'dark' | 'light'
/// console.log(config.theme.size);  // 'minimal' | 'normal' | 'wide'
/// console.log(config.language);    // 'en' | 'ja'
/// ```
#[tauri::command]
pub fn read_config() -> AppConfig {
    AppConfig::load()
}

/// Saves the application configuration
///
/// Writes the configuration to `~/.config/uti/config.json`.
///
/// # Arguments
///
/// * `config` - The configuration to save
///
/// # Returns
///
/// Ok if successful, Err with error message if failed
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('save_config', { config: { theme: { color: 'dark' } } });
/// ```
#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
    config.save()
}

/// Opens the config folder in the default file manager
///
/// Opens `~/.config/uti/` using the system's default file manager.
///
/// # Returns
///
/// Ok if successful, Err with error message if failed
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('open_config_folder');
/// ```
#[tauri::command]
pub fn open_config_folder() -> Result<(), String> {
    let folder = AppConfig::get_config_dir();

    // Ensure the directory exists
    if !folder.exists() {
        std::fs::create_dir_all(&folder)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    open::that(&folder).map_err(|e| format!("Failed to open folder: {}", e))
}

/// Reloads the configuration from file and returns it
///
/// This is used to refresh the configuration after manual file edits.
/// Same as `read_config` but semantically indicates a reload operation.
///
/// # Returns
///
/// The reloaded configuration
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// const config = await invoke('reload_config');
/// ```
#[tauri::command]
pub fn reload_config() -> AppConfig {
    println!("Reloading configuration...");
    AppConfig::load()
}
