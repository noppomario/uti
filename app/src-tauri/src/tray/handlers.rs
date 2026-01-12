//! Tray menu event handlers

use crate::settings::window_size;
use crate::updater::open_update_dialog;
use tauri::{menu::MenuEvent, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;

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
    open_update_dialog(app.clone());
}

/// Open GitHub repository
fn handle_github() {
    let _ = open::that("https://github.com/noppomario/uti");
}

/// Quit the application
fn handle_quit(app: &AppHandle) {
    app.exit(0);
}
