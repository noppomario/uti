//! Settings window and window size management
//!
//! Handles window dimensions for different themes and modes.

use tauri::WebviewWindow;

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
    /// Settings window size
    pub const SETTINGS: (f64, f64) = (500.0, 600.0);

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
