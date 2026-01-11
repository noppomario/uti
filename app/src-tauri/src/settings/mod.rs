//! Settings window and window size management
//!
//! Handles window dimensions for different themes and modes,
//! and provides commands for settings UI functionality.

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
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

/// Result of update check from settings
#[derive(Debug, Serialize)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
}

/// Checks for updates from settings UI
///
/// Returns update check result that the frontend can display.
///
/// # Returns
///
/// UpdateCheckResult with version info
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// const result = await invoke('check_for_updates');
/// if (result.update_available) {
///   console.log(`Update available: ${result.latest_version}`);
/// }
/// ```
#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    use crate::updater;

    let current_version = env!("CARGO_PKG_VERSION");

    let result = updater::check_for_updates(current_version)
        .await
        .map_err(|e| format!("{}", e))?;

    Ok(UpdateCheckResult {
        current_version: result.current_version,
        latest_version: result.latest_version,
        update_available: result.update_available,
    })
}

/// URL-encode a string for use in query parameters
fn urlencoding(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
}

/// Checks for updates and shows a dialog window with the result
///
/// This provides the same UX as the tray menu update check.
///
/// # Arguments
///
/// * `app` - The Tauri AppHandle
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('check_for_updates_with_dialog');
/// ```
#[tauri::command]
pub async fn check_for_updates_with_dialog(app: AppHandle) {
    use crate::updater;

    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let (title, message, kind) = match updater::check_for_updates(&current_version).await {
        Ok(result) => {
            if result.update_available {
                (
                    "Update Available".to_string(),
                    format!(
                        "Update available: {} -> {}\n\nRun 'uti update' in terminal to install.",
                        result.current_version, result.latest_version
                    ),
                    "info".to_string(),
                )
            } else {
                (
                    "No Update".to_string(),
                    "You are running the latest version.".to_string(),
                    "info".to_string(),
                )
            }
        }
        Err(e) => (
            "Update Check Failed".to_string(),
            format!("{}", e),
            "error".to_string(),
        ),
    };

    // Close existing dialog window if any
    if let Some(existing) = app.get_webview_window("dialog") {
        let _ = existing.close();
    }

    // Create new dialog window with URL parameters
    let url = format!(
        "update-dialog.html?title={}&message={}&kind={}",
        urlencoding(&title),
        urlencoding(&message),
        urlencoding(&kind)
    );

    match WebviewWindowBuilder::new(&app, "dialog", WebviewUrl::App(url.into()))
        .title("uti")
        .inner_size(420.0, 200.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .center()
        .visible(true)
        .focused(true)
        .build()
    {
        Ok(_) => println!("Update dialog created"),
        Err(e) => eprintln!("Failed to create update dialog: {:?}", e),
    }
}

/// Opens the GitHub repository in the default browser
///
/// # Returns
///
/// Ok if successful, Err with error message if failed
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('open_github');
/// ```
#[tauri::command]
pub fn open_github() -> Result<(), String> {
    open::that("https://github.com/noppomario/uti")
        .map_err(|e| format!("Failed to open URL: {}", e))
}
