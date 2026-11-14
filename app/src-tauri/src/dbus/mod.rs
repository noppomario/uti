//! D-Bus integration (Phase 1 contract)
//!
//! The app owns `io.github.noppomario.uti`, serves `org.freedesktop.Application`,
//! listens to the daemon's `Triggered` signal and calls the GNOME extension
//! (`Shell1`) and the daemon (`Daemon1`). All of this goes through a single
//! connection, because the daemon and the extension only accept calls from the
//! owner of `io.github.noppomario.uti`.

mod actions;

pub use actions::Action;

use std::collections::HashMap;
use tauri::AppHandle;
use zbus::{fdo, interface, proxy, Connection, OwnedVariant};

/// Well-known bus name owned by the app.
pub use uti_core::dbus::APP_BUS_NAME as APP_NAME;
/// Object path where `org.freedesktop.Application` is served.
pub use uti_core::dbus::APP_OBJECT_PATH as APP_PATH;

/// Daemon `Daemon1` interface (double Ctrl detector, uinput paste fallback).
#[proxy(
    interface = "io.github.noppomario.uti.Daemon1",
    default_service = "io.github.noppomario.uti.Daemon",
    default_path = "/io/github/noppomario/uti/Daemon"
)]
pub trait Daemon1 {
    /// Emitted when double Ctrl press is detected.
    #[zbus(signal)]
    fn triggered(&self) -> zbus::Result<()>;

    /// Pastes the clipboard into the focused window via uinput.
    fn paste(&self, plain: bool, submit: bool) -> zbus::Result<()>;
}

/// GNOME extension `Shell1` interface.
#[proxy(
    interface = "io.github.noppomario.uti.Shell1",
    default_service = "io.github.noppomario.uti.Shell",
    default_path = "/io/github/noppomario/uti/Shell"
)]
pub trait Shell1 {
    /// Moves the main window to the pointer once it is presented.
    fn position_at_pointer(&self, activate: bool) -> zbus::Result<()>;

    /// Keeps the main window above other windows.
    fn set_above(&self, above: bool) -> zbus::Result<()>;

    /// Pastes the clipboard into the focused window via a virtual keyboard.
    fn paste(&self, plain: bool, submit: bool) -> zbus::Result<()>;
}

/// The app's single D-Bus connection, stored in Tauri state.
///
/// `None` when the session bus is unavailable; all calls then become no-ops.
pub struct AppBus {
    conn: Option<Connection>,
}

impl AppBus {
    /// Connects to the session bus, serves `org.freedesktop.Application` and
    /// requests [`APP_NAME`].
    ///
    /// Failing to acquire the name is logged but not fatal, so the app still
    /// works (without passing the extension's caller check) while an old daemon
    /// holds it.
    pub async fn connect(app: AppHandle) -> Self {
        let conn = zbus::connection::Builder::session()
            .serve_at(APP_PATH, ApplicationService { app })
            .build()
            .await;
        let conn = match conn {
            Ok(c) => c,
            Err(e) => {
                eprintln!("D-Bus session connection failed: {}", e);
                return Self { conn: None };
            }
        };
        match conn.request_name(APP_NAME).await {
            Ok(()) => println!("Acquired D-Bus name {}", APP_NAME),
            Err(e) => eprintln!("Failed to acquire D-Bus name {}: {}", APP_NAME, e),
        }
        Self { conn: Some(conn) }
    }

    /// Returns the shared connection, if connected.
    pub fn connection(&self) -> Option<&Connection> {
        self.conn.as_ref()
    }

    async fn shell(&self) -> Option<Shell1Proxy<'_>> {
        Shell1Proxy::new(self.conn.as_ref()?).await.ok()
    }

    /// Asks the extension to position the main window at the pointer.
    ///
    /// Errors (e.g. extension not installed) are ignored.
    pub async fn position_at_pointer(&self) {
        if let Some(shell) = self.shell().await {
            if let Err(e) = shell.position_at_pointer(false).await {
                eprintln!("Shell1.PositionAtPointer failed: {}", e);
            }
        }
    }

    /// Asks the extension to keep the main window above others.
    pub async fn set_above(&self, above: bool) {
        if let Some(shell) = self.shell().await {
            if let Err(e) = shell.set_above(above).await {
                eprintln!("Shell1.SetAbove failed: {}", e);
            }
        }
    }

    /// Pastes via the extension, falling back to the daemon.
    pub async fn paste(&self, plain: bool, submit: bool) {
        let Some(conn) = self.conn.as_ref() else {
            return;
        };
        let shell_err = match self.shell().await {
            Some(shell) => match shell.paste(plain, submit).await {
                Ok(()) => return,
                Err(e) => e.to_string(),
            },
            None => "proxy unavailable".to_string(),
        };
        eprintln!(
            "Shell1.Paste failed ({}), falling back to daemon",
            shell_err
        );
        let result = match Daemon1Proxy::new(conn).await {
            Ok(daemon) => daemon.paste(plain, submit).await,
            Err(e) => Err(e),
        };
        if let Err(e) = result {
            eprintln!("Daemon1.Paste failed: {}", e);
        }
    }
}

/// `org.freedesktop.Application` implementation.
struct ApplicationService {
    app: AppHandle,
}

#[interface(name = "org.freedesktop.Application")]
impl ApplicationService {
    /// No-op: the app has no default activation behavior.
    fn activate(&self, _platform_data: HashMap<String, OwnedVariant>) {}

    /// No-op: the app does not open files.
    fn open(&self, _uris: Vec<String>, _platform_data: HashMap<String, OwnedVariant>) {}

    /// Runs a named action (see [`Action`]).
    ///
    /// The action runs in a separate task so that a caller blocking on this
    /// reply (e.g. the extension) can still serve the `Shell1` calls it makes.
    fn activate_action(
        &self,
        action_name: &str,
        _parameter: Vec<OwnedVariant>,
        _platform_data: HashMap<String, OwnedVariant>,
    ) -> fdo::Result<()> {
        let action = Action::parse(action_name)
            .ok_or_else(|| fdo::Error::InvalidArgs(format!("Unknown action: {}", action_name)))?;
        println!("ActivateAction: {:?}", action);
        let app = self.app.clone();
        tauri::async_runtime::spawn(async move { actions::run(&app, action).await });
        Ok(())
    }
}

/// Listens for the daemon's `Triggered` signal and forwards it to the frontend
/// as `double-ctrl-pressed`.
///
/// Retries with exponential backoff (1 s -> 30 s) when the proxy or signal
/// stream cannot be created, and re-subscribes when the stream ends.
pub async fn listen_triggered(conn: Connection, window: tauri::WebviewWindow) {
    use futures_util::stream::StreamExt;
    use std::time::Duration;
    use tauri::Emitter;

    let mut retry_delay = Duration::from_secs(1);
    let max_delay = Duration::from_secs(30);

    loop {
        let stream = match Daemon1Proxy::new(&conn).await {
            Ok(proxy) => proxy.receive_triggered().await,
            Err(e) => Err(e),
        };
        match stream {
            Ok(mut stream) => {
                println!("Listening for Daemon1.Triggered...");
                retry_delay = Duration::from_secs(1);
                while stream.next().await.is_some() {
                    println!("Daemon1.Triggered received");
                    let _ = window.emit("double-ctrl-pressed", ());
                }
                eprintln!(
                    "Triggered stream ended, resubscribing in {:?}...",
                    retry_delay
                );
            }
            Err(e) => eprintln!(
                "Subscribing to Triggered failed: {}, retrying in {:?}...",
                e, retry_delay
            ),
        }
        tokio::time::sleep(retry_delay).await;
        retry_delay = (retry_delay * 2).min(max_delay);
    }
}
