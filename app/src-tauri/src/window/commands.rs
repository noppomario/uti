//! Window Tauri commands
//!
//! Provides window control commands for the frontend.

use std::sync::atomic::Ordering;
use tauri::{State, WebviewWindow};
use zbus::Connection;

use super::PinState;

/// Toggles the window visibility state
#[tauri::command]
pub fn toggle_window(window: WebviewWindow, pin_state: State<'_, PinState>) {
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
    pinned: bool,
) -> Result<(), String> {
    pin_state.is_pinned.store(pinned, Ordering::SeqCst);

    // Emit D-Bus signal for GNOME extension to handle always-on-top
    if let Ok(conn) = Connection::session().await {
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

/// Emits a TypeText D-Bus signal to trigger auto-paste via daemon
#[tauri::command]
pub async fn type_text() {
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
