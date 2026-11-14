//! D-Bus names shared by the app, the daemon and the GNOME extension
//!
//! zbus attribute macros need string literals, so `#[interface]` / `#[proxy]`
//! declarations repeat these values (the daemon test checks its interface).

/// Bus name owned by the uti app (also its GApplication id).
pub const APP_BUS_NAME: &str = "io.github.noppomario.uti";
/// Object path of the app's `org.freedesktop.Application`.
pub const APP_OBJECT_PATH: &str = "/io/github/noppomario/uti";

/// Bus name owned by uti-daemon.
pub const DAEMON_BUS_NAME: &str = "io.github.noppomario.uti.Daemon";
/// Object path of `Daemon1`.
pub const DAEMON_OBJECT_PATH: &str = "/io/github/noppomario/uti/Daemon";
/// Daemon interface: signal `Triggered`, method `Paste(bb)`.
pub const DAEMON_INTERFACE: &str = "io.github.noppomario.uti.Daemon1";

/// Bus name owned by the "uti for GNOME" extension.
pub const SHELL_BUS_NAME: &str = "io.github.noppomario.uti.Shell";
/// Object path of `Shell1`.
pub const SHELL_OBJECT_PATH: &str = "/io/github/noppomario/uti/Shell";
/// Extension interface: `PositionAtPointer`, `SetAbove`, `Paste`, `ClipboardChanged`.
pub const SHELL_INTERFACE: &str = "io.github.noppomario.uti.Shell1";

/// Legacy object path still emitted by the daemon for one release.
pub const LEGACY_DOUBLETAP_PATH: &str = "/io/github/noppomario/uti/DoubleTap";
/// Legacy interface still emitted by the daemon for one release.
pub const LEGACY_DOUBLETAP_INTERFACE: &str = "io.github.noppomario.uti.DoubleTap";
