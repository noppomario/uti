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

/// Subscribes to `Daemon1.DoubleClicked` (left double click anywhere).
pub fn on_double_clicked(
    conn: &gio::DBusConnection,
    callback: impl Fn() + 'static,
) -> gio::SignalSubscription {
    conn.subscribe_to_signal(
        Some(DAEMON_BUS_NAME),
        Some(DAEMON_INTERFACE),
        Some("DoubleClicked"),
        Some(DAEMON_OBJECT_PATH),
        None,
        gio::DBusSignalFlags::NONE,
        move |_| callback(),
    )
}

/// Subscribes to `Daemon1.LeftRightClicked` (right press while left is held).
pub fn on_left_right_clicked(
    conn: &gio::DBusConnection,
    callback: impl Fn() + 'static,
) -> gio::SignalSubscription {
    conn.subscribe_to_signal(
        Some(DAEMON_BUS_NAME),
        Some(DAEMON_INTERFACE),
        Some("LeftRightClicked"),
        Some(DAEMON_OBJECT_PATH),
        None,
        gio::DBusSignalFlags::NONE,
        move |_| callback(),
    )
}

/// Calls `on_appeared` whenever the daemon or the extension (re)appears on
/// the bus, including right away if they are already running.
pub fn watch_services(
    conn: &gio::DBusConnection,
    on_appeared: impl Fn() + Clone + 'static,
) -> [gio::WatcherId; 2] {
    [DAEMON_BUS_NAME, SHELL_BUS_NAME].map(|name| {
        let f = on_appeared.clone();
        gio::bus_watch_name_on_connection(
            conn,
            name,
            gio::BusNameWatcherFlags::NONE,
            move |_, _, _| f(),
            |_, _| {},
        )
    })
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
) -> Result<glib::Variant, glib::Error> {
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
}

const SHELL: (&str, &str, &str) = (SHELL_BUS_NAME, SHELL_OBJECT_PATH, SHELL_INTERFACE);

/// Asks the extension to move the main window to the pointer on its next frame.
pub async fn position_at_pointer(conn: &gio::DBusConnection) {
    if let Err(e) = call(conn, SHELL, "PositionAtPointer", (false,).to_variant()).await {
        eprintln!("Shell1.PositionAtPointer failed: {e}");
    }
}

const DAEMON: (&str, &str, &str) = (DAEMON_BUS_NAME, DAEMON_OBJECT_PATH, DAEMON_INTERFACE);

/// Turns the left-held + right click trigger on or off: the daemon's mouse
/// grab and the extension's blocking of background long presses.
pub async fn set_left_right_click(conn: &gio::DBusConnection, enabled: bool) {
    if let Err(e) = call(conn, DAEMON, "SetMouseGrab", (enabled,).to_variant()).await {
        eprintln!("Daemon1.SetMouseGrab failed: {e}");
    }
    if let Err(e) = call(conn, SHELL, "SetBlockBackgroundPress", (enabled,).to_variant()).await {
        eprintln!("Shell1.SetBlockBackgroundPress failed: {e}");
    }
}

/// Whether the pointer is over the desktop; `false` if the extension is missing.
pub async fn pointer_on_desktop(conn: &gio::DBusConnection) -> bool {
    match call(conn, SHELL, "PointerOnDesktop", ().to_variant()).await {
        Ok(reply) => reply.get::<(bool,)>().is_some_and(|(b,)| b),
        Err(e) => {
            eprintln!("Shell1.PointerOnDesktop failed: {e}");
            false
        },
    }
}

/// Keeps the main window above other windows (or stops doing so).
pub async fn set_above(conn: &gio::DBusConnection, above: bool) {
    if let Err(e) = call(conn, SHELL, "SetAbove", (above,).to_variant()).await {
        eprintln!("Shell1.SetAbove failed: {e}");
    }
}
