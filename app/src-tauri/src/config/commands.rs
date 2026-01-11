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

/// Opens the launcher configuration file in the default editor
///
/// Opens `~/.config/uti/launcher.json` using the system's default editor.
/// Creates the file with an empty config if it doesn't exist.
///
/// # Returns
///
/// Ok if successful, Err with error message if failed
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('open_launcher_config');
/// ```
#[tauri::command]
pub fn open_launcher_config() -> Result<(), String> {
    use crate::launcher::get_launcher_config_path;

    let path = get_launcher_config_path();

    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
    }

    // Create the file with default content if it doesn't exist
    if !path.exists() {
        let default_content = r#"{
  "commands": []
}
"#;
        std::fs::write(&path, default_content)
            .map_err(|e| format!("Failed to create launcher config: {}", e))?;
    }

    open::that(&path).map_err(|e| format!("Failed to open file: {}", e))
}

/// Opens the snippets configuration file in the default editor
///
/// Opens `~/.config/uti/snippets.json` using the system's default editor.
/// Creates the file with an empty config if it doesn't exist.
///
/// # Returns
///
/// Ok if successful, Err with error message if failed
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('open_snippets_config');
/// ```
#[tauri::command]
pub fn open_snippets_config() -> Result<(), String> {
    use crate::snippets::SnippetsStore;

    let path = SnippetsStore::get_storage_path();

    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
    }

    // Create the file with default content if it doesn't exist
    if !path.exists() {
        let default_content = r#"{
  "items": []
}
"#;
        std::fs::write(&path, default_content)
            .map_err(|e| format!("Failed to create snippets config: {}", e))?;
    }

    open::that(&path).map_err(|e| format!("Failed to open file: {}", e))
}
