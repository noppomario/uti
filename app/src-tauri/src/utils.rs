//! Utility commands
//!
//! Generic utility functions used across modules.

/// Gets the current application version
///
/// Returns the version from Cargo.toml.
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// const version = await invoke('get_version');
/// console.log(version); // e.g., "0.1.0"
/// ```
#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Opens a URL in the default browser
///
/// # Arguments
///
/// * `url` - The URL to open
///
/// # Returns
///
/// Ok if successful, Err with error message if failed
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('open_url', { url: 'https://example.com' });
/// ```
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
}
