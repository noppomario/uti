//! Tauri Application Backend
//!
//! This module handles D-Bus signal reception and window visibility toggling
//! for the uti desktop utility.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod config;
mod launcher;
mod settings;
mod snippets;
mod tray;
mod updater;

use clap::{Parser, Subcommand};
use clipboard::{ClipboardItem, ClipboardStore};
use config::{
    open_config_folder, open_launcher_config, open_snippets_config, read_config, reload_config,
    save_config, AppConfig,
};
use launcher::{LauncherConfig, RecentFile};
use settings::{
    apply_window_size, check_for_updates, check_for_updates_with_dialog, get_autostart_status,
    get_version, open_github, set_autostart, set_window_mode,
};
use snippets::{load_snippets, save_snippets, SnippetItem, SnippetsStore};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State, WebviewWindow};
use tauri_plugin_autostart::MacosLauncher;
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

/// Application state for window pin functionality
///
/// Tracks whether the window is pinned (always-on-top with auto-hide disabled).
struct PinState {
    is_pinned: Arc<AtomicBool>,
}

impl PinState {
    fn new() -> Self {
        Self {
            is_pinned: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// Gets the clipboard history
///
/// Returns a list of clipboard items sorted by timestamp (newest first).
#[tauri::command]
fn get_clipboard_history(store: State<Mutex<ClipboardStore>>) -> Vec<ClipboardItem> {
    let store = store.lock().unwrap();
    store.items.clone()
}

/// Adds a new item to the clipboard history
///
/// If the item already exists, its timestamp will be updated.
/// Enforces the maximum item limit via LRU eviction.
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
#[tauri::command]
async fn paste_item(text: String) -> Result<(), String> {
    println!("Would paste: {}", text);
    Ok(())
}

/// Gets recent files from recently-used.xbel
#[tauri::command]
fn get_recent_files(app_name: Option<String>, xbel_path: Option<String>) -> Vec<RecentFile> {
    launcher::recent_files::get_recent_files_from_xbel(app_name.as_deref(), xbel_path.as_deref())
}

/// Gets recent files from VSCode state.vscdb SQLite database
#[tauri::command]
fn get_vscode_recent_files(storage_path: String) -> Vec<RecentFile> {
    launcher::recent_files::get_recent_files_from_vscode(&storage_path)
}

/// Executes a command with optional arguments
#[tauri::command]
fn execute_command(command: String, args: Vec<String>) -> Result<(), String> {
    std::process::Command::new(&command)
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to execute {}: {}", command, e))?;
    Ok(())
}

/// Gets the launcher configuration
#[tauri::command]
fn get_launcher_config() -> LauncherConfig {
    launcher::load_launcher_config()
}

/// Gets all snippets
#[tauri::command]
fn get_snippets(store: State<Mutex<SnippetsStore>>) -> Vec<SnippetItem> {
    let store = store.lock().unwrap();
    store.items.clone()
}

/// Adds a new snippet (used when pinning from clipboard)
#[tauri::command]
fn add_snippet(
    value: String,
    label: Option<String>,
    store: State<Mutex<SnippetsStore>>,
) -> SnippetItem {
    let mut store = store.lock().unwrap();
    let item = SnippetItem::new(value, label);
    store.items.push(item.clone());

    if let Err(e) = save_snippets(&store) {
        eprintln!("Failed to save snippets: {}", e);
    }
    item
}

/// Removes a clipboard item by index (used when pinning to snippets)
#[tauri::command]
fn remove_clipboard_item(index: usize, store: State<Mutex<ClipboardStore>>) {
    let mut store = store.lock().unwrap();
    if index < store.items.len() {
        store.items.remove(index);
        let path = ClipboardStore::get_storage_path();
        if let Err(e) = store.save(&path) {
            eprintln!("Failed to save clipboard store: {}", e);
        }
    }
}

/// Emits a TypeText D-Bus signal to trigger auto-paste via daemon
#[tauri::command]
async fn type_text() {
    match Connection::session().await {
        Ok(conn) => {
            if let Err(e) = conn
                .emit_signal(
                    None::<()>,
                    "/io/github/noppomario/uti/DoubleTap",
                    "io.github.noppomario.uti.DoubleTap",
                    "TypeText",
                    &(),
                )
                .await
            {
                eprintln!("Failed to emit TypeText signal: {}", e);
            } else {
                println!("TypeText signal emitted");
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to D-Bus: {}", e);
        }
    }
}

/// Toggles the window visibility state
#[tauri::command]
fn toggle_window(window: WebviewWindow, pin_state: State<'_, PinState>) {
    // Ignore toggle when window is pinned to prevent unexpected behavior
    if pin_state.is_pinned.load(Ordering::SeqCst) {
        println!("Window is pinned, toggle_window ignored");
        return;
    }

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

/// Force hide window for paste operation (ignores PIN state)
#[tauri::command]
fn hide_for_paste(window: WebviewWindow) {
    let _ = window.hide();
    println!("Window hidden for paste (PIN state ignored)");
}

/// Show window (for re-showing after paste when pinned)
#[tauri::command]
fn show_window(window: WebviewWindow) {
    let _ = window.show();
    let _ = window.set_focus();
    println!("Window shown");
}

/// Set window pinned state (always-on-top with auto-hide disabled)
#[tauri::command]
async fn set_pinned(
    window: WebviewWindow,
    pin_state: State<'_, PinState>,
    pinned: bool,
) -> Result<(), String> {
    pin_state.is_pinned.store(pinned, Ordering::SeqCst);

    // Emit D-Bus signal for GNOME extension to handle always-on-top
    if let Ok(conn) = zbus::Connection::session().await {
        let _ = conn
            .emit_signal(
                None::<()>,
                "/io/github/noppomario/uti/DoubleTap",
                "io.github.noppomario.uti.DoubleTap",
                "SetAlwaysOnTop",
                &(pinned,),
            )
            .await;
        println!("D-Bus SetAlwaysOnTop signal emitted: {}", pinned);
    }

    // Also call Tauri API (works on non-GNOME environments)
    window
        .set_always_on_top(pinned)
        .map_err(|e| e.to_string())?;
    println!("Window pinned: {}", pinned);
    Ok(())
}

/// Search for desktop applications matching the query
#[tauri::command]
fn search_desktop_files(query: String) -> Vec<launcher::DesktopApp> {
    launcher::search_desktop_files(&query)
}

/// Listens for D-Bus signals from the daemon and forwards them to the frontend
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
    let app_config = AppConfig::load();

    // Load clipboard store from file, respecting config limit
    let path = ClipboardStore::get_storage_path();
    let mut store = ClipboardStore::load(&path);

    // Apply config limit (in case it changed since last save)
    if store.max_items != app_config.clipboard_history_limit {
        store.max_items = app_config.clipboard_history_limit;
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
        .manage(Mutex::new(store))
        .manage(Mutex::new(load_snippets()))
        .manage(PinState::new())
        .invoke_handler(tauri::generate_handler![
            // Window commands
            toggle_window,
            hide_for_paste,
            show_window,
            set_pinned,
            set_window_mode,
            type_text,
            // Clipboard commands
            get_clipboard_history,
            add_clipboard_item,
            remove_clipboard_item,
            paste_item,
            // Config commands
            read_config,
            save_config,
            open_config_folder,
            open_launcher_config,
            open_snippets_config,
            reload_config,
            // Launcher commands
            get_recent_files,
            get_vscode_recent_files,
            execute_command,
            get_launcher_config,
            search_desktop_files,
            // Snippets commands
            get_snippets,
            add_snippet,
            // Settings commands
            get_version,
            get_autostart_status,
            set_autostart,
            check_for_updates,
            check_for_updates_with_dialog,
            open_github,
        ])
        .setup(move |app| {
            let window = app.get_webview_window("main").unwrap();

            // Apply window size based on theme configuration
            let config = AppConfig::load();
            apply_window_size(&window, &config.theme.size);

            // Hide window if started with --minimized flag
            if start_minimized {
                window.hide().ok();
            }

            // Setup tray icon
            if let Err(e) = tray::setup_tray(app) {
                eprintln!("Failed to setup tray: {}", e);
            }

            // Auto-hide window when it loses focus (unless pinned)
            let window_for_blur = window.clone();
            let is_pinned = app.state::<PinState>().is_pinned.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(focused) = event {
                    if !focused && !is_pinned.load(Ordering::SeqCst) {
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
