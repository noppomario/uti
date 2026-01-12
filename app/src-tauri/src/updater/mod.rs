//! Self-update functionality for uti
//!
//! This module provides functions to check for updates and install them
//! via GitHub Releases and RPM packages.

mod core;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

// Re-export public items from core for external use (CLI, etc.)
pub use core::{check_for_updates, perform_update, UpdateCheckResultFull};

/// Window size for update dialog
pub mod dialog_size {
    /// Update dialog size (width, height) - larger for progress display
    pub const SIZE: (f64, f64) = (500.0, 500.0);
}

/// Checks for updates with full details for GUI
///
/// Returns UpdateCheckResultFull that includes release URL for linking.
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// const result = await invoke('check_for_updates_full');
/// if (result.update_available) {
///   console.log(`Update ${result.latest_version} available at ${result.release_url}`);
/// }
/// ```
#[tauri::command]
pub async fn check_for_updates_full() -> Result<UpdateCheckResultFull, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    core::check_for_updates_full(current_version)
        .await
        .map_err(|e| format!("{}", e))
}

/// Performs update with progress events for GUI
///
/// Downloads and installs updates while emitting progress events.
/// The frontend should listen for 'update-progress' events.
///
/// # Arguments
///
/// * `app` - The Tauri AppHandle for emitting events
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// import { listen } from '@tauri-apps/api/event';
///
/// // Listen for progress
/// await listen('update-progress', (event) => {
///   console.log(event.payload);
/// });
///
/// // Start update
/// await invoke('perform_update_gui');
/// ```
#[tauri::command]
pub async fn perform_update_gui(app: AppHandle) -> Result<(), String> {
    let current_version = env!("CARGO_PKG_VERSION");

    // Get update info first
    let result = core::check_for_updates_full(current_version)
        .await
        .map_err(|e| format!("{}", e))?;

    if !result.update_available {
        return Err("No update available".to_string());
    }

    // Perform update with progress events
    core::perform_update_with_progress(&app, &result)
        .await
        .map_err(|e| format!("{}", e))
}

/// Opens the update dialog window
///
/// Creates a new update dialog window that handles the full update flow.
///
/// # Arguments
///
/// * `app` - The Tauri AppHandle
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('open_update_dialog');
/// ```
#[tauri::command]
pub fn open_update_dialog(app: AppHandle) {
    // Close existing dialog window if any
    if let Some(existing) = app.get_webview_window("dialog") {
        let _ = existing.close();
    }

    let (width, height) = dialog_size::SIZE;

    // Create new dialog window (no URL params - dialog handles state internally)
    match WebviewWindowBuilder::new(&app, "dialog", WebviewUrl::App("update-dialog.html".into()))
        .title("uti")
        .inner_size(width, height)
        .min_inner_size(400.0, 300.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .center()
        .visible(true)
        .focused(true)
        .build()
    {
        Ok(_) => println!("Update dialog created ({}x{})", width, height),
        Err(e) => eprintln!("Failed to create update dialog: {:?}", e),
    }
}
