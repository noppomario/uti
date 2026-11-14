//! Launcher Tauri commands
//!
//! Provides launcher and recent files commands for the frontend.

use super::recent_files;
use super::{DesktopApp, LauncherConfig, RecentFile};

/// Gets recent files from recently-used.xbel
#[tauri::command]
pub fn get_recent_files(app_name: Option<String>, xbel_path: Option<String>) -> Vec<RecentFile> {
    recent_files::get_recent_files_from_xbel(app_name.as_deref(), xbel_path.as_deref())
}

/// Gets recent files from VSCode state.vscdb SQLite database
#[tauri::command]
pub fn get_vscode_recent_files(storage_path: String) -> Vec<RecentFile> {
    recent_files::get_recent_files_from_vscode(&storage_path)
}

/// Executes a command with optional arguments
#[tauri::command]
pub fn execute_command(command: String, args: Vec<String>) -> Result<(), String> {
    std::process::Command::new(&command)
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to execute {}: {}", command, e))?;
    Ok(())
}

/// Gets the launcher configuration
#[tauri::command]
pub fn get_launcher_config() -> LauncherConfig {
    super::load_launcher_config()
}

/// Search for desktop applications matching the query
#[tauri::command]
pub fn search_desktop_files(query: String) -> Vec<DesktopApp> {
    super::desktop::search_desktop_files(&query)
}
