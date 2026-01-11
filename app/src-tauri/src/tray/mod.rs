//! System tray management module
//!
//! Handles tray icon, menu construction, and menu event handling.

mod handlers;

use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager,
};

pub use handlers::handle_menu_event;

/// Build and initialize the system tray
///
/// Creates the tray icon with menu items and sets up event handlers.
///
/// # Arguments
///
/// * `app` - The Tauri application instance
///
/// # Returns
///
/// Ok if successful, Err with error message if failed
pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let show_hide_i = MenuItem::with_id(app, "show_hide", "Show/Hide", true, None::<&str>)?;

    // Check if autostart is enabled
    use tauri_plugin_autostart::ManagerExt;
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

    // Settings menu item
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;

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
            &settings_i,
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
        .on_menu_event(handle_menu_event)
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

    Ok(())
}
