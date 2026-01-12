//! Tauri Application Backend
//!
//! This module handles D-Bus signal reception and window visibility toggling
//! for the uti desktop utility.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod clipboard;
mod config;
mod launcher;
mod settings;
mod snippets;
mod tray;
mod updater;
mod utils;
mod window;

use clap::Parser;
use clipboard::{
    add_clipboard_item, get_clipboard_history, paste_item, remove_clipboard_item, ClipboardStore,
};
use config::{
    open_config_folder, open_launcher_config, open_snippets_config, read_config, reload_config,
    save_config, AppConfig,
};
use launcher::{
    execute_command, get_launcher_config, get_recent_files, get_vscode_recent_files,
    search_desktop_files,
};
use settings::{apply_window_size, get_autostart_status, set_autostart, set_window_mode};
use snippets::{add_snippet, get_snippets, load_snippets};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::MacosLauncher;
use updater::{check_for_updates_full, open_update_dialog, perform_update_gui};
use utils::{get_version, open_url};
use window::{hide_for_paste, set_pinned, show_window, toggle_window, type_text, PinState};
use zbus::Connection;

/// Listens for D-Bus signals from the daemon and forwards them to the frontend.
///
/// Uses exponential backoff retry (1s -> 2s -> 4s -> ... -> max 30s) for:
/// - D-Bus session connection
/// - Proxy creation (daemon may not be running yet)
/// - Signal stream acquisition
///
/// Automatically reconnects when the stream ends (e.g., daemon restart).
async fn listen_dbus(window: tauri::WebviewWindow) {
    use futures_util::stream::StreamExt;
    use std::time::Duration;
    use zbus::proxy;

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

    let mut retry_delay = Duration::from_secs(1);
    let max_delay = Duration::from_secs(30);

    loop {
        // Connect to D-Bus session bus
        let conn = match Connection::session().await {
            Ok(c) => {
                println!("Connected to D-Bus session bus");
                c
            }
            Err(e) => {
                eprintln!(
                    "D-Bus connection failed: {}, retrying in {:?}...",
                    e, retry_delay
                );
                tokio::time::sleep(retry_delay).await;
                retry_delay = (retry_delay * 2).min(max_delay);
                continue;
            }
        };

        // Create proxy (may fail if daemon not running)
        let proxy = match DoubleTapProxy::new(&conn, "io.github.noppomario.uti").await {
            Ok(p) => p,
            Err(e) => {
                eprintln!(
                    "D-Bus proxy creation failed: {}, retrying in {:?}...",
                    e, retry_delay
                );
                tokio::time::sleep(retry_delay).await;
                retry_delay = (retry_delay * 2).min(max_delay);
                continue;
            }
        };

        // Get signal stream - success, reset retry delay
        let mut stream = match proxy.receive_triggered().await {
            Ok(s) => {
                println!("Listening for D-Bus signals...");
                retry_delay = Duration::from_secs(1);
                s
            }
            Err(e) => {
                eprintln!(
                    "Failed to receive signals: {}, retrying in {:?}...",
                    e, retry_delay
                );
                tokio::time::sleep(retry_delay).await;
                retry_delay = (retry_delay * 2).min(max_delay);
                continue;
            }
        };

        // Process signals until stream ends
        while let Some(_signal) = stream.next().await {
            println!("D-Bus signal received!");
            let _ = window.emit("double-ctrl-pressed", ());
        }

        // Stream ended (connection lost), retry with backoff
        eprintln!(
            "D-Bus signal stream ended, reconnecting in {:?}...",
            retry_delay
        );
        tokio::time::sleep(retry_delay).await;
        retry_delay = (retry_delay * 2).min(max_delay);
    }
}

/// Main application entry point
fn main() {
    let args = cli::Cli::parse();

    // Handle CLI subcommands
    if cli::run_command(&args) {
        return;
    }

    // No subcommand: run GUI
    run_gui(args.minimized);
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
            check_for_updates_full,
            perform_update_gui,
            open_update_dialog,
            open_url,
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
                listen_dbus(window_clone).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
