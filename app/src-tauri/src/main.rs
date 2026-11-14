//! Tauri Application Backend
//!
//! Wires the window, Tauri commands and D-Bus (see `dbus`) together.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod clipboard;
mod config;
mod dbus;
mod launcher;
mod settings;
mod snippets;
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
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use updater::{check_for_updates_full, open_update_dialog, perform_update_gui};
use utils::{get_version, open_url};
use window::{hide_for_paste, set_pinned, show_window, toggle_window, type_text, PinState};

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

            // Single D-Bus connection: owns the app name, serves
            // org.freedesktop.Application and carries all outgoing calls.
            let bus = tauri::async_runtime::block_on(dbus::AppBus::connect(app.handle().clone()));
            if let Some(conn) = bus.connection().cloned() {
                let window_clone = window.clone();
                tauri::async_runtime::spawn(dbus::listen_triggered(conn, window_clone));
            }
            app.manage(bus);

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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
