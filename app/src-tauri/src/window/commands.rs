//! Window Tauri commands
//!
//! Provides window control commands for the frontend.

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{State, WebviewWindow};

use super::PinState;
use crate::dbus::AppBus;

/// Returns true when running under GNOME (the extension handles positioning).
fn is_gnome() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|v| v.to_uppercase().contains("GNOME"))
        .unwrap_or(false)
}

/// Shows and focuses the main window.
///
/// On GNOME, `Shell1.PositionAtPointer` is called first: the extension moves
/// the window on its first frame, so it must know before the window is shown.
/// Elsewhere the window is centered.
pub async fn show(window: &WebviewWindow, bus: &AppBus) {
    if is_gnome() {
        bus.position_at_pointer().await;
    } else if let Err(e) = window.center() {
        eprintln!("Failed to center window: {}", e);
    }
    let _ = window.show();
    let _ = window.set_focus();
    println!("Window shown");
}

/// Toggles the main window, ignoring the request while pinned.
pub async fn toggle(window: &WebviewWindow, pinned: &AtomicBool, bus: &AppBus) {
    if pinned.load(Ordering::SeqCst) {
        println!("Window is pinned, toggle ignored");
        return;
    }
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        println!("Window hidden");
    } else {
        show(window, bus).await;
    }
}

/// Toggles the window visibility state
#[tauri::command]
pub async fn toggle_window(
    window: WebviewWindow,
    pin_state: State<'_, PinState>,
    bus: State<'_, AppBus>,
) -> Result<(), String> {
    toggle(&window, &pin_state.is_pinned, &bus).await;
    Ok(())
}

/// Force hide window for paste operation (ignores PIN state)
#[tauri::command]
pub fn hide_for_paste(window: WebviewWindow) {
    let _ = window.hide();
    println!("Window hidden for paste (PIN state ignored)");
}

/// Show window (for re-showing after paste when pinned)
#[tauri::command]
pub fn show_window(window: WebviewWindow) {
    let _ = window.show();
    let _ = window.set_focus();
    println!("Window shown");
}

/// Set window pinned state (always-on-top with auto-hide disabled)
#[tauri::command]
pub async fn set_pinned(
    window: WebviewWindow,
    pin_state: State<'_, PinState>,
    bus: State<'_, AppBus>,
    pinned: bool,
) -> Result<(), String> {
    pin_state.is_pinned.store(pinned, Ordering::SeqCst);

    // GNOME/Wayland: only the compositor can keep a window above
    bus.set_above(pinned).await;

    // Also call Tauri API (works on non-GNOME environments)
    window
        .set_always_on_top(pinned)
        .map_err(|e| e.to_string())?;
    println!("Window pinned: {}", pinned);
    Ok(())
}

/// Pastes the clipboard into the focused window (extension, then daemon fallback)
#[tauri::command]
pub async fn type_text(bus: State<'_, AppBus>) -> Result<(), String> {
    bus.paste(true, true).await;
    Ok(())
}
