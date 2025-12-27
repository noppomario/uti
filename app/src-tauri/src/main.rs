//! Tauri Application Backend
//!
//! This module handles D-Bus signal reception and window visibility toggling
//! for the uti desktop utility.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Emitter, Manager, WebviewWindow};
use zbus::Connection;

/// Toggles the window visibility state
///
/// If the window is currently visible, it will be hidden. If hidden, it will be shown
/// and focused.
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
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        println!("Window hidden");
    } else {
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
    let conn = Connection::session().await?;
    println!("Connected to D-Bus session bus");

    // zbus v4 SignalStream API
    use futures_util::stream::StreamExt;
    use zbus::proxy;

    /// D-Bus proxy interface for receiving DoubleTap signals
    #[proxy(
        interface = "io.github.noppomario.uti.DoubleTap",
        default_path = "/io/github/noppomario/uti/DoubleTap"
    )]
    trait DoubleTap {
        /// Signal emitted when double Ctrl press is detected
        #[zbus(signal)]
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
/// Sets up the Tauri application with the toggle_window command handler
/// and spawns an async task to listen for D-Bus signals.
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![toggle_window])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

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
