//! Tray menu event handlers

use crate::settings::window_size;
use crate::updater;
use tauri::{menu::MenuEvent, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;

/// Payload for update dialog window
#[derive(Clone)]
struct UpdateDialogPayload {
    title: String,
    message: String,
    kind: String, // "info" | "error"
}

/// URL-encode a string for use in query parameters
fn urlencoding(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
}

/// Handle tray menu events
///
/// Called when a menu item is clicked in the tray menu.
pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        "show_hide" => handle_show_hide(app),
        "settings" => handle_settings(app),
        "autostart" => handle_autostart(app),
        "check_update" => handle_check_update(app),
        "github" => handle_github(),
        "quit" => handle_quit(app),
        _ => {}
    }
}

/// Toggle main window visibility
fn handle_show_hide(app: &AppHandle) {
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

/// Open settings window
fn handle_settings(app: &AppHandle) {
    // Check if settings window already exists
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.set_focus();
        return;
    }

    // Create settings window with custom title bar (matches main window style)
    let (width, height) = window_size::SETTINGS;
    let (min_width, min_height) = window_size::SETTINGS;
    match WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("uti - Settings")
        .inner_size(width, height)
        .min_inner_size(min_width, min_height)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .center()
        .visible(true)
        .focused(true)
        .build()
    {
        Ok(_) => println!("Settings window created"),
        Err(e) => eprintln!("Failed to create settings window: {:?}", e),
    }
}

/// Toggle autostart setting
fn handle_autostart(app: &AppHandle) {
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

/// Check for updates and show dialog
fn handle_check_update(app: &AppHandle) {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let payload = match updater::check_for_updates(&current_version).await {
            Ok(result) => {
                if result.update_available {
                    UpdateDialogPayload {
                        title: "Update Available".to_string(),
                        message: format!(
                            "Update available: {} -> {}\n\nRun 'uti update' in terminal to install.",
                            result.current_version, result.latest_version
                        ),
                        kind: "info".to_string(),
                    }
                } else {
                    UpdateDialogPayload {
                        title: "No Update".to_string(),
                        message: "You are running the latest version.".to_string(),
                        kind: "info".to_string(),
                    }
                }
            }
            Err(e) => UpdateDialogPayload {
                title: "Update Check Failed".to_string(),
                message: format!("{}", e),
                kind: "error".to_string(),
            },
        };

        // Close existing dialog window if any
        if let Some(existing) = app_handle.get_webview_window("dialog") {
            let _ = existing.close();
        }

        // Create new dialog window with URL parameters
        let url = format!(
            "update-dialog.html?title={}&message={}&kind={}",
            urlencoding(&payload.title),
            urlencoding(&payload.message),
            urlencoding(&payload.kind)
        );

        match WebviewWindowBuilder::new(&app_handle, "dialog", WebviewUrl::App(url.into()))
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
            Ok(dialog_window) => {
                println!("Dialog window created: {:?}", dialog_window.label());
            }
            Err(e) => {
                eprintln!("Failed to create dialog window: {:?}", e);
            }
        }
    });
}

/// Open GitHub repository
fn handle_github() {
    let _ = open::that("https://github.com/noppomario/uti");
}

/// Quit the application
fn handle_quit(app: &AppHandle) {
    app.exit(0);
}
