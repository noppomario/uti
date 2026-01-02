//! Tauri Application Backend
//!
//! This module handles D-Bus signal reception and window visibility toggling
//! for the uti desktop utility.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod launcher;
mod updater;

use clap::{Parser, Subcommand};
use clipboard::{ClipboardItem, ClipboardStore};
use launcher::{LauncherConfig, RecentFile};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WebviewWindow,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use zbus::Connection;

/// uti - Double Ctrl hotkey desktop tool
#[derive(Parser)]
#[command(name = "uti")]
#[command(about = "Desktop utility for toggling window visibility with double Ctrl press")]
#[command(version, long_version = env!("CARGO_PKG_VERSION"), disable_version_flag = true)]
struct Cli {
    /// Print version
    #[arg(short = 'v', short_alias = 'V', long = "version", action = clap::ArgAction::Version)]
    version: (),

    /// Start minimized (used by autostart)
    #[arg(long)]
    minimized: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check for updates and install if available
    Update {
        /// Check only, don't install
        #[arg(long)]
        check: bool,
    },
}

/// Application configuration
///
/// This struct represents the user's configuration for the uti application.
/// The configuration is stored in `~/.config/uti/config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// Theme mode: 'light' or 'dark'
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Maximum number of clipboard items to store
    #[serde(default = "default_clipboard_limit")]
    pub clipboard_history_limit: usize,

    /// Whether to show tooltip on hover
    #[serde(default = "default_show_tooltip")]
    pub show_tooltip: bool,

    /// Delay before showing tooltip (milliseconds)
    #[serde(default = "default_tooltip_delay")]
    pub tooltip_delay: u32,
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_clipboard_limit() -> usize {
    50
}

fn default_show_tooltip() -> bool {
    true
}

fn default_tooltip_delay() -> u32 {
    500
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            clipboard_history_limit: default_clipboard_limit(),
            show_tooltip: default_show_tooltip(),
            tooltip_delay: default_tooltip_delay(),
        }
    }
}

impl AppConfig {
    /// Get the path to the config file
    ///
    /// Returns `~/.config/uti/config.json`
    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("uti");
        path.push("config.json");
        path
    }

    /// Validate configuration values
    ///
    /// Ensures theme is one of: 'dark' or 'light'
    fn validate(&mut self) {
        // Validate theme
        if !matches!(self.theme.as_str(), "dark" | "light") {
            eprintln!("Invalid theme '{}', falling back to 'dark'", self.theme);
            self.theme = "dark".to_string();
        }

        // Validate clipboard_history_limit
        if self.clipboard_history_limit == 0 {
            eprintln!("clipboard_history_limit cannot be 0, using default (50)");
            self.clipboard_history_limit = 50;
        }
    }

    /// Load configuration from file
    ///
    /// If the file doesn't exist or can't be read, returns default config.
    fn load() -> Self {
        let path = Self::get_config_path();

        match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<Self>(&contents) {
                Ok(mut config) => {
                    println!("Loaded config from: {:?}", path);
                    config.validate();
                    config
                }
                Err(e) => {
                    eprintln!("Failed to parse config file: {}", e);
                    Self::default()
                }
            },
            Err(_) => {
                println!("Config file not found, using defaults");
                Self::default()
            }
        }
    }
}

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
/// console.log(config.theme); // 'dark' | 'light'
/// ```
#[tauri::command]
fn read_config() -> AppConfig {
    AppConfig::load()
}

/// Gets the clipboard history
///
/// Returns a list of clipboard items sorted by timestamp (newest first).
///
/// # Arguments
///
/// * `store` - The shared clipboard store state
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// const history = await invoke('get_clipboard_history');
/// ```
#[tauri::command]
fn get_clipboard_history(store: State<Mutex<ClipboardStore>>) -> Vec<ClipboardItem> {
    let store = store.lock().unwrap();
    store.items.clone()
}

/// Adds a new item to the clipboard history
///
/// If the item already exists, its timestamp will be updated.
/// Enforces the maximum item limit via LRU eviction.
///
/// # Arguments
///
/// * `text` - The clipboard text content
/// * `store` - The shared clipboard store state
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('add_clipboard_item', { text: 'Hello' });
/// ```
#[tauri::command]
fn add_clipboard_item(text: String, store: State<Mutex<ClipboardStore>>) {
    let mut store = store.lock().unwrap();
    store.add(text);

    // Save to file
    let path = ClipboardStore::get_storage_path();
    if let Err(e) = store.save(&path) {
        eprintln!("Failed to save clipboard store: {}", e);
    }
}

/// Sets the system clipboard to the specified text
///
/// # Arguments
///
/// * `text` - The text to set to the clipboard
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('paste_item', { text: 'Hello' });
/// ```
#[tauri::command]
async fn paste_item(text: String) -> Result<(), String> {
    // Note: This requires app handle, will be properly implemented in setup
    // For now, just return Ok to pass compilation
    println!("Would paste: {}", text);
    Ok(())
}

/// Gets recent files from recently-used.xbel
///
/// # Arguments
///
/// * `app_name` - Application name to filter by (optional, e.g., "org.gnome.Nautilus")
///   - Required when using system XBEL (xbel_path is None)
///   - Optional when using per-app XBEL (xbel_path is Some)
/// * `xbel_path` - Optional custom path to XBEL file (supports ~ expansion)
///
/// # Returns
///
/// Vector of recent files, sorted by timestamp (newest first), limited to 10 items.
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// // Use system recently-used.xbel with app filter
/// const files = await invoke('get_recent_files', { appName: 'org.gnome.Nautilus' });
/// // Use custom XBEL path with app filter
/// const files = await invoke('get_recent_files', {
///   appName: 'gnome-text-editor',
///   xbelPath: '~/.local/share/org.gnome.TextEditor/recently-used.xbel'
/// });
/// // Use custom XBEL path without filter (all entries)
/// const files = await invoke('get_recent_files', {
///   xbelPath: '~/.local/share/org.gnome.TextEditor/recently-used.xbel'
/// });
/// ```
#[tauri::command]
fn get_recent_files(app_name: Option<String>, xbel_path: Option<String>) -> Vec<RecentFile> {
    launcher::recent_files::get_recent_files_from_xbel(app_name.as_deref(), xbel_path.as_deref())
}

/// Gets recent files from VSCode state.vscdb SQLite database
///
/// # Arguments
///
/// * `storage_path` - Path to state.vscdb (supports ~ expansion)
///
/// # Returns
///
/// Vector of recent files, limited to 10 items.
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// const files = await invoke('get_vscode_recent_files', {
///   storagePath: '~/.config/Code/User/globalStorage/state.vscdb'
/// });
/// ```
#[tauri::command]
fn get_vscode_recent_files(storage_path: String) -> Vec<RecentFile> {
    launcher::recent_files::get_recent_files_from_vscode(&storage_path)
}

/// Executes a command with optional arguments
///
/// # Arguments
///
/// * `command` - The command to execute (e.g., "nautilus", "code")
/// * `args` - Arguments to pass to the command (e.g., ["/path/to/file"])
///
/// # Returns
///
/// Returns Ok(()) if the command was spawned successfully, Err with message otherwise.
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// await invoke('execute_command', { command: 'nautilus', args: ['/home/user'] });
/// ```
#[tauri::command]
fn execute_command(command: String, args: Vec<String>) -> Result<(), String> {
    std::process::Command::new(&command)
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to execute {}: {}", command, e))?;
    Ok(())
}

/// Gets the launcher configuration
///
/// Returns the launcher configuration from `~/.config/uti/launcher.json`.
/// If the file doesn't exist, returns default configuration.
///
/// # Examples
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// const config = await invoke('get_launcher_config');
/// console.log(config.commands);
/// ```
#[tauri::command]
fn get_launcher_config() -> LauncherConfig {
    launcher::load_launcher_config()
}

/// Toggles the window visibility state
///
/// If the window is currently visible, it will be hidden. If hidden, it will be shown
/// at the cursor position and focused.
///
/// # Arguments
///
/// * `window` - The Tauri window instance to toggle
///
/// # Examples
///
/// This function is invoked from the frontend:
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
/// invoke('toggle_window');
/// ```
#[tauri::command]
fn toggle_window(window: WebviewWindow) {
    let is_visible = window.is_visible().unwrap_or(false);
    println!("Current window state: visible={}", is_visible);

    if is_visible {
        let _ = window.hide();
        println!("Window hidden");
    } else {
        // On GNOME, the extension handles positioning at cursor location.
        // On other environments, center the window as fallback.
        let is_gnome = std::env::var("XDG_CURRENT_DESKTOP")
            .map(|v| v.to_uppercase().contains("GNOME"))
            .unwrap_or(false);

        if is_gnome {
            // Wait for GNOME extension to position the window before showing.
            // The extension receives the D-Bus signal and calls move_frame().
            std::thread::sleep(std::time::Duration::from_millis(50));
        } else {
            match window.center() {
                Ok(_) => println!("Window centered on screen"),
                Err(e) => eprintln!("Failed to center window: {}", e),
            }
        }

        let _ = window.show();
        let _ = window.set_focus();
        println!("Window shown");
    }
}

/// Listens for D-Bus signals from the daemon and forwards them to the frontend
///
/// This function connects to the D-Bus session bus, subscribes to the DoubleTap
/// signal, and emits a frontend event whenever the signal is received.
///
/// # Arguments
///
/// * `window` - The Tauri window instance for emitting frontend events
///
/// # Returns
///
/// Returns `Ok(())` if the listener was set up successfully and runs indefinitely.
/// Returns an error if D-Bus connection or signal subscription fails.
///
/// # Errors
///
/// This function will return an error if:
/// - Failed to connect to D-Bus session bus
/// - Failed to create the D-Bus proxy
/// - Failed to subscribe to the signal stream
async fn listen_dbus(window: WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    use futures_util::stream::StreamExt;
    use zbus::proxy;

    let conn = Connection::session().await?;
    println!("Connected to D-Bus session bus");

    /// D-Bus proxy interface for receiving DoubleTap signals
    #[proxy(
        interface = "io.github.noppomario.uti.DoubleTap",
        default_path = "/io/github/noppomario/uti/DoubleTap"
    )]
    trait DoubleTap {
        /// Signal emitted when double Ctrl press is detected
        #[zbus(signal, name = "Triggered")]
        fn triggered(&self) -> zbus::Result<()>;
    }

    let proxy = DoubleTapProxy::new(&conn, "io.github.noppomario.uti").await?;
    let mut stream = proxy.receive_triggered().await?;
    println!("Listening for D-Bus signals...");

    while let Some(_signal) = stream.next().await {
        println!("D-Bus signal received!");
        let _ = window.emit("double-ctrl-pressed", ());
    }

    Ok(())
}

/// Handle CLI update command
async fn handle_update_command(check_only: bool) {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);
    println!("Checking for updates...");

    match updater::check_for_updates(current_version).await {
        Ok(result) => {
            if result.update_available {
                println!(
                    "Update available: {} -> {}",
                    result.current_version, result.latest_version
                );

                if check_only {
                    println!("Run 'uti update' to install the update.");
                    return;
                }

                if result.uti_rpm_url.is_none() && result.daemon_rpm_url.is_none() {
                    println!("No RPM packages found in the release.");
                    return;
                }

                println!("Installing update...");
                match updater::perform_update(&result).await {
                    Ok(()) => {
                        println!("Update installed successfully!");
                        println!();
                        // Red bold warning box
                        println!("\x1b[1;31m+--------------------------------------------------------------+\x1b[0m");
                        println!("\x1b[1;31m|  WARNING: YOU MUST LOG OUT AND LOG BACK IN TO APPLY CHANGES  |\x1b[0m");
                        println!("\x1b[1;31m+--------------------------------------------------------------+\x1b[0m");
                        println!();
                    }
                    Err(e) => {
                        eprintln!("Update failed: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("You are running the latest version.");
            }
        }
        Err(e) => {
            eprintln!("Failed to check for updates: {}", e);
            std::process::exit(1);
        }
    }
}

/// Main application entry point
///
/// Sets up the Tauri application with clipboard management commands
/// and spawns an async task to listen for D-Bus signals.
fn main() {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(command) = cli.command {
        match command {
            Commands::Update { check } => {
                // Run update check in a tokio runtime
                let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                rt.block_on(handle_update_command(check));
                return;
            }
        }
    }

    // No subcommand: run GUI
    run_gui(cli.minimized);
}

/// Run the Tauri GUI application
fn run_gui(start_minimized: bool) {
    // Load config to get clipboard history limit
    let config = AppConfig::load();

    // Load clipboard store from file, respecting config limit
    let path = ClipboardStore::get_storage_path();
    let mut store = ClipboardStore::load(&path);

    // Apply config limit (in case it changed since last save)
    if store.max_items != config.clipboard_history_limit {
        store.max_items = config.clipboard_history_limit;
        // Save updated limit to file
        if let Err(e) = store.save(&path) {
            eprintln!("Failed to save updated max_items: {}", e);
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(store))
        .invoke_handler(tauri::generate_handler![
            toggle_window,
            get_clipboard_history,
            add_clipboard_item,
            paste_item,
            read_config,
            get_recent_files,
            get_vscode_recent_files,
            execute_command,
            get_launcher_config
        ])
        .setup(move |app| {
            let window = app.get_webview_window("main").unwrap();

            // Hide window if started with --minimized flag
            if start_minimized {
                window.hide().ok();
            }

            // === Tray icon setup ===
            let show_hide_i = MenuItem::with_id(app, "show_hide", "Show/Hide", true, None::<&str>)?;

            // Check if autostart is enabled
            let autostart_manager = app.autolaunch();
            let autostart_enabled = autostart_manager.is_enabled().unwrap_or(false);
            let autostart_i = CheckMenuItem::with_id(
                app,
                "autostart",
                "Auto-start",
                true,
                autostart_enabled,
                None::<&str>,
            )?;

            // App title with version (disabled, just for info)
            let version = app.package_info().version.to_string();
            let title_i = MenuItem::with_id(
                app,
                "title",
                format!("uti v{}", version),
                false,
                None::<&str>,
            )?;

            let update_i = MenuItem::with_id(
                app,
                "check_update",
                "Check for Updates...",
                true,
                None::<&str>,
            )?;
            let github_i = MenuItem::with_id(app, "github", "GitHub ↗", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &show_hide_i,
                    &PredefinedMenuItem::separator(app)?,
                    &autostart_i,
                    &update_i,
                    &github_i,
                    &PredefinedMenuItem::separator(app)?,
                    &title_i,
                    &quit_i,
                ],
            )?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show_hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.center();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                    "autostart" => {
                        let autostart_manager = app.autolaunch();
                        let is_enabled = autostart_manager.is_enabled().unwrap_or(false);
                        if is_enabled {
                            if let Err(e) = autostart_manager.disable() {
                                eprintln!("Failed to disable autostart: {}", e);
                            } else {
                                println!("Auto-start disabled");
                            }
                        } else if let Err(e) = autostart_manager.enable() {
                            eprintln!("Failed to enable autostart: {}", e);
                        } else {
                            println!("Auto-start enabled");
                        }
                    }
                    "check_update" => {
                        let current_version = env!("CARGO_PKG_VERSION").to_string();
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            match updater::check_for_updates(&current_version).await {
                                Ok(result) => {
                                    if result.update_available {
                                        app_handle
                                            .dialog()
                                            .message(format!(
                                                "Update available: {} → {}\n\nRun 'uti update' in terminal to install.",
                                                result.current_version, result.latest_version
                                            ))
                                            .kind(MessageDialogKind::Info)
                                            .title("Update Available")
                                            .show(|_| {});
                                    } else {
                                        app_handle
                                            .dialog()
                                            .message("You are running the latest version.")
                                            .kind(MessageDialogKind::Info)
                                            .title("No Update")
                                            .show(|_| {});
                                    }
                                }
                                Err(e) => {
                                    app_handle
                                        .dialog()
                                        .message(format!("{}", e))
                                        .kind(MessageDialogKind::Error)
                                        .title("Update Check Failed")
                                        .show(|_| {});
                                }
                            }
                        });
                    }
                    "github" => {
                        let _ = open::that("https://github.com/noppomario/uti");
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.center();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Auto-hide window when it loses focus
            let window_for_blur = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(focused) = event {
                    if !focused {
                        let _ = window_for_blur.hide();
                        println!("Window lost focus, hiding");
                    }
                }
            });

            let window_clone = window.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = listen_dbus(window_clone).await {
                    eprintln!("D-Bus error: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
