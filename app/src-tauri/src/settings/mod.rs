//! Settings window and window size management
//!
//! Handles window dimensions for different themes and modes,
//! and provides commands for autostart functionality.

use tauri::{AppHandle, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;

/// Window size constants for each theme
pub mod window_size {
    /// Minimal theme: compact width
    pub const MINIMAL: (f64, f64) = (250.0, 600.0);
    /// Normal theme: balanced size
    pub const NORMAL: (f64, f64) = (500.0, 700.0);
    /// Wide theme: expanded width
    pub const WIDE: (f64, f64) = (750.0, 800.0);
    /// Prompt mode: horizontal layout for text input
    pub const PROMPT: (f64, f64) = (600.0, 250.0);
    /// Settings window size (width, height)
    pub const SETTINGS: (f64, f64) = (640.0, 480.0);

    /// Get window size by theme name
    pub fn by_theme(theme: &str) -> (f64, f64) {
        match theme {
            "normal" => NORMAL,
            "wide" => WIDE,
            _ => MINIMAL,
        }
    }
}

/// Apply window size based on size theme
///
/// Sets the window dimensions based on the configured size theme.
///
/// # Arguments
///
/// * `window` - The Tauri window instance
/// * `size` - The size theme name: "minimal", "normal", or "wide"
pub fn apply_window_size(window: &WebviewWindow, size: &str) {
    let (width, height) = window_size::by_theme(size);

    if let Err(e) = window.set_size(tauri::LogicalSize::new(width, height)) {
        eprintln!("Failed to set window size: {}", e);
    } else {
        println!("Window size set to {}x{} ({})", width, height, size);
    }
}

/// Sets window mode for different tab layouts
///
/// Switches between "prompt" mode (horizontal layout) and "default" mode
/// (uses configured theme size).
///
/// # Arguments
///
/// * `window` - The Tauri window instance
/// * `mode` - The window mode: "prompt" or "default"
#[tauri::command]
pub fn set_window_mode(window: WebviewWindow, mode: String) {
    use crate::config::AppConfig;

    let (width, height) = match mode.as_str() {
        "prompt" => window_size::PROMPT,
        _ => {
            // Use configured theme size for default mode
            let config = AppConfig::load();
            window_size::by_theme(&config.theme.size)
        }
    };

    if let Err(e) = window.set_size(tauri::LogicalSize::new(width, height)) {
        eprintln!("Failed to set window mode: {}", e);
    } else {
        println!("Window mode set to {} ({}x{})", mode, width, height);
    }
}

/// Gets the current autostart status
///
/// Returns true if autostart is enabled, false otherwise.
///
/// # Arguments
///
/// * `app` - The Tauri AppHandle
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// const enabled = await invoke('get_autostart_status');
/// console.log(enabled); // true or false
/// ```
#[tauri::command]
pub fn get_autostart_status(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

/// Sets the autostart status
///
/// Enables or disables autostart on login.
///
/// # Arguments
///
/// * `app` - The Tauri AppHandle
/// * `enabled` - Whether to enable autostart
///
/// # Returns
///
/// Ok if successful, Err with error message if failed
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('set_autostart', { enabled: true });
/// ```
#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let autostart_manager = app.autolaunch();

    if enabled {
        autostart_manager
            .enable()
            .map_err(|e| format!("Failed to enable autostart: {}", e))?;
        println!("Auto-start enabled");
    } else {
        autostart_manager
            .disable()
            .map_err(|e| format!("Failed to disable autostart: {}", e))?;
        println!("Auto-start disabled");
    }

    Ok(())
}
