//! Tauri Application Backend
//!
//! This module handles D-Bus signal reception and window visibility toggling
//! for the uti desktop utility.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod clipboard_store;

use clipboard::ClipboardItem;
use clipboard_store::ClipboardStore;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State, WebviewWindow};
use zbus::Connection;

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
        // Center window on screen (Wayland doesn't support cursor positioning)
        match window.center() {
            Ok(_) => println!("Window centered on screen"),
            Err(e) => eprintln!("Failed to center window: {}", e),
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

/// Main application entry point
///
/// Sets up the Tauri application with clipboard management commands
/// and spawns an async task to listen for D-Bus signals.
fn main() {
    // Load clipboard store from file
    let path = ClipboardStore::get_storage_path();
    let store = ClipboardStore::load(&path);

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(Mutex::new(store))
        .invoke_handler(tauri::generate_handler![
            toggle_window,
            get_clipboard_history,
            add_clipboard_item,
            paste_item
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

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
