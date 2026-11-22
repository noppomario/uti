//! D-Bus calls over the GApplication's own connection
//!
//! The extension and the daemon only accept calls from the owner of
//! `io.github.noppomario.uti`, which is this connection.

use gtk::{gio, glib, prelude::*};

/// Bus name owned by the uti app (also its GApplication id).
pub const APP_BUS_NAME: &str = "io.github.noppomario.uti";

const DAEMON_BUS_NAME: &str = "io.github.noppomario.uti.Daemon";
const DAEMON_OBJECT_PATH: &str = "/io/github/noppomario/uti/Daemon";
const DAEMON_INTERFACE: &str = "io.github.noppomario.uti.Daemon1";

const SHELL_BUS_NAME: &str = "io.github.noppomario.uti.Shell";
const SHELL_OBJECT_PATH: &str = "/io/github/noppomario/uti/Shell";
const SHELL_INTERFACE: &str = "io.github.noppomario.uti.Shell1";

const CALL_TIMEOUT_MS: i32 = 2000;

/// Subscribes to `Daemon1.Triggered` (double Ctrl).
pub fn on_triggered(
    conn: &gio::DBusConnection,
    callback: impl Fn() + 'static,
) -> gio::SignalSubscription {
    conn.subscribe_to_signal(
        Some(DAEMON_BUS_NAME),
        Some(DAEMON_INTERFACE),
        Some("Triggered"),
        Some(DAEMON_OBJECT_PATH),
        None,
        gio::DBusSignalFlags::NONE,
        move |_| callback(),
    )
}

/// Subscribes to `Shell1.ClipboardChanged(s)`.
pub fn on_clipboard_changed(
    conn: &gio::DBusConnection,
    callback: impl Fn(String) + 'static,
) -> gio::SignalSubscription {
    conn.subscribe_to_signal(
        Some(SHELL_BUS_NAME),
        Some(SHELL_INTERFACE),
        Some("ClipboardChanged"),
        Some(SHELL_OBJECT_PATH),
        None,
        gio::DBusSignalFlags::NONE,
        move |signal| {
            if let Some((text,)) = signal.parameters.get::<(String,)>() {
                callback(text);
            }
        },
    )
}

/// Subscribes to `Shell1.ShellPressed()` (desktop background or panel clicked).
pub fn on_shell_pressed(
    conn: &gio::DBusConnection,
    callback: impl Fn() + 'static,
) -> gio::SignalSubscription {
    conn.subscribe_to_signal(
        Some(SHELL_BUS_NAME),
        Some(SHELL_INTERFACE),
        Some("ShellPressed"),
        Some(SHELL_OBJECT_PATH),
        None,
        gio::DBusSignalFlags::NONE,
        move |_| callback(),
    )
}

async fn call(
    conn: &gio::DBusConnection,
    service: (&str, &str, &str),
    method: &str,
    params: glib::Variant,
) -> Result<(), glib::Error> {
    let (name, path, iface) = service;
    conn.call_future(
        Some(name),
        path,
        iface,
        method,
        Some(&params),
        None,
        gio::DBusCallFlags::NONE,
        CALL_TIMEOUT_MS,
    )
    .await
    .map(|_| ())
}

const SHELL: (&str, &str, &str) = (SHELL_BUS_NAME, SHELL_OBJECT_PATH, SHELL_INTERFACE);

/// Asks the extension to move the main window to the pointer on its next frame.
pub async fn position_at_pointer(conn: &gio::DBusConnection) {
    if let Err(e) = call(conn, SHELL, "PositionAtPointer", (false,).to_variant()).await {
        eprintln!("Shell1.PositionAtPointer failed: {e}");
    }
}

/// Keeps the main window above other windows (or stops doing so).
pub async fn set_above(conn: &gio::DBusConnection, above: bool) {
    if let Err(e) = call(conn, SHELL, "SetAbove", (above,).to_variant()).await {
        eprintln!("Shell1.SetAbove failed: {e}");
    }
}
