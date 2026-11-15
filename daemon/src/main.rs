//! uti keyboard daemon
//!
//! Watches all keyboards (including ones plugged in later) for a double Ctrl
//! tap and announces it on D-Bus as `io.github.noppomario.uti.Daemon1.Triggered`.
//! Also offers `Daemon1.Paste` as a uinput fallback for the GNOME extension's
//! paste; only the owner of the app's bus name may call it.

mod detector;
mod uinput;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use detector::{DoubleTapDetector, KeyEvent};
use evdev::{Device, EventType, KeyCode};
use futures_util::StreamExt;
use inotify::{Inotify, WatchMask};
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use uinput::{VirtualKeyboard, VIRTUAL_KEYBOARD_NAME};
use zbus::message::Header;
use zbus::object_server::SignalEmitter;
use zbus::{fdo, Connection};

// D-Bus names; the app and the GNOME extension use the same literals.
const APP_BUS_NAME: &str = "io.github.noppomario.uti";
const BUS_NAME: &str = "io.github.noppomario.uti.Daemon";
const OBJECT_PATH: &str = "/io/github/noppomario/uti/Daemon";
#[cfg(test)]
const DAEMON_INTERFACE: &str = "io.github.noppomario.uti.Daemon1";
/// Legacy signal still emitted for clients older than v0.3.0.
const LEGACY_PATH: &str = "/io/github/noppomario/uti/DoubleTap";
const LEGACY_INTERFACE: &str = "io.github.noppomario.uti.DoubleTap";
const INPUT_DIR: &str = "/dev/input";

/// Maps a raw key event to a detector event; `None` for irrelevant events.
///
/// Auto-repeat (`value == 2`) and releases of non-Ctrl keys are ignored.
fn classify(code: KeyCode, value: i32) -> Option<KeyEvent> {
    let is_ctrl = code == KeyCode::KEY_LEFTCTRL || code == KeyCode::KEY_RIGHTCTRL;
    match (is_ctrl, value) {
        (true, 1) => Some(KeyEvent::CtrlDown),
        (true, 0) => Some(KeyEvent::CtrlUp),
        (false, 1) => Some(KeyEvent::OtherDown),
        _ => None,
    }
}

/// Returns true for real keyboards, excluding uti's own virtual keyboard.
fn is_keyboard(name: Option<&str>, has_key_a: bool) -> bool {
    has_key_a && name != Some(VIRTUAL_KEYBOARD_NAME)
}

fn is_event_node(path: &Path) -> bool {
    path.file_name().is_some_and(|n| n.to_string_lossy().starts_with("event"))
}

/// Tracks monitored device nodes and spawns one reader task per keyboard.
#[derive(Clone)]
struct Devices {
    active: Arc<Mutex<HashSet<PathBuf>>>,
    tx: mpsc::UnboundedSender<KeyEvent>,
}

impl Devices {
    /// Starts monitoring `path` if it is a keyboard not monitored yet.
    fn try_add(&self, path: PathBuf) {
        if !is_event_node(&path) || self.active.lock().unwrap().contains(&path) {
            return;
        }
        let Ok(device) = Device::open(&path) else {
            return; // Not readable (yet); udev may fix permissions (IN_ATTRIB).
        };
        let has_key_a = device.supported_keys().is_some_and(|k| k.contains(KeyCode::KEY_A));
        let name = device.name().unwrap_or("unknown").to_owned();
        if !is_keyboard(Some(&name), has_key_a) {
            return;
        }
        let stream = match device.into_event_stream() {
            Ok(s) => s,
            Err(e) => return warn!("[{name}] cannot stream events: {e}"),
        };
        self.active.lock().unwrap().insert(path.clone());
        info!("[{name}] monitoring {}", path.display());

        let this = self.clone();
        tokio::spawn(async move {
            let mut stream = stream;
            loop {
                match stream.next_event().await {
                    Ok(ev) if ev.event_type() == EventType::KEY => {
                        if let Some(k) = classify(KeyCode::new(ev.code()), ev.value()) {
                            if this.tx.send(k).is_err() {
                                break;
                            }
                        }
                    },
                    Ok(_) => {},
                    Err(e) => {
                        info!("[{name}] stopped: {e}");
                        break;
                    },
                }
            }
            this.active.lock().unwrap().remove(&path);
        });
    }

    fn scan(&self) {
        match std::fs::read_dir(INPUT_DIR) {
            Ok(entries) => entries.flatten().for_each(|e| self.try_add(e.path())),
            Err(e) => error!("cannot read {INPUT_DIR}: {e}"),
        }
    }
}

/// Watches `/dev/input` for new device nodes.
async fn watch_hotplug(devices: Devices) -> std::io::Result<()> {
    let inotify = Inotify::init()?;
    inotify.watches().add(INPUT_DIR, WatchMask::CREATE | WatchMask::ATTRIB)?;
    let mut events = inotify.into_event_stream([0u8; 4096])?;
    while let Some(event) = events.next().await {
        if let Some(name) = event?.name {
            debug!("hotplug: {}", name.to_string_lossy());
            devices.try_add(Path::new(INPUT_DIR).join(name));
        }
    }
    Ok(())
}

/// `io.github.noppomario.uti.Daemon1`
struct Daemon1 {
    keyboard: Arc<Mutex<Option<VirtualKeyboard>>>,
}

#[zbus::interface(name = "io.github.noppomario.uti.Daemon1")]
impl Daemon1 {
    /// Simulates a paste in the focused window (uinput fallback).
    async fn paste(
        &self,
        plain: bool,
        submit: bool,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] conn: &Connection,
    ) -> fdo::Result<()> {
        let sender = header.sender().map(|s| s.to_string());
        let owner = fdo::DBusProxy::new(conn)
            .await?
            .get_name_owner(zbus::names::BusName::try_from(APP_BUS_NAME)?)
            .await
            .ok()
            .map(|o| o.to_string());
        if sender.is_none() || sender != owner {
            warn!("Paste denied for {sender:?} (owner {owner:?})");
            return Err(fdo::Error::AccessDenied(format!("caller does not own {APP_BUS_NAME}")));
        }

        let keyboard = Arc::clone(&self.keyboard);
        tokio::task::spawn_blocking(move || match keyboard.lock().unwrap().as_mut() {
            Some(kb) => kb.paste(plain, submit).map_err(|e| e.to_string()),
            None => Err("virtual keyboard unavailable".into()),
        })
        .await
        .map_err(|e| fdo::Error::Failed(e.to_string()))?
        .map_err(fdo::Error::Failed)
    }

    /// Emitted on a double Ctrl tap.
    #[zbus(signal)]
    async fn triggered(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
}

async fn announce(conn: &Connection) {
    match SignalEmitter::new(conn, OBJECT_PATH) {
        Ok(emitter) => {
            if let Err(e) = Daemon1::triggered(&emitter).await {
                error!("failed to emit Triggered: {e}");
            }
        },
        Err(e) => error!("invalid object path: {e}"),
    }
    let legacy = conn
        .emit_signal(None::<()>, LEGACY_PATH, LEGACY_INTERFACE, "Triggered", &())
        .await;
    if let Err(e) = legacy {
        error!("failed to emit legacy Triggered: {e}");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("uti-daemon starting");

    let keyboard = match VirtualKeyboard::new() {
        Ok(kb) => Some(kb),
        Err(e) => {
            warn!("virtual keyboard unavailable, Paste disabled: {e}");
            None
        },
    };

    let conn = zbus::connection::Builder::session()
        .name(BUS_NAME)
        .serve_at(
            OBJECT_PATH,
            Daemon1 {
                keyboard: Arc::new(Mutex::new(keyboard)),
            },
        )
        .build()
        .await?;
    info!("D-Bus ready as {BUS_NAME}");

    let (tx, mut rx) = mpsc::unbounded_channel();
    let devices = Devices {
        active: Arc::default(),
        tx,
    };
    devices.scan();
    let hotplug = devices.clone();
    tokio::spawn(async move {
        if let Err(e) = watch_hotplug(hotplug).await {
            error!("hotplug watch failed, new keyboards will be ignored: {e}");
        }
    });

    let mut detector = DoubleTapDetector::new();
    while let Some(event) = rx.recv().await {
        if detector.feed(event, Instant::now()) {
            info!("double Ctrl detected");
            announce(&conn).await;
        }
    }
    Err("event channel closed".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_ctrl_press_and_release() {
        assert_eq!(classify(KeyCode::KEY_LEFTCTRL, 1), Some(KeyEvent::CtrlDown));
        assert_eq!(classify(KeyCode::KEY_RIGHTCTRL, 0), Some(KeyEvent::CtrlUp));
    }

    #[test]
    fn classify_ignores_repeat_and_other_releases() {
        assert_eq!(classify(KeyCode::KEY_LEFTCTRL, 2), None);
        assert_eq!(classify(KeyCode::KEY_C, 0), None);
        assert_eq!(classify(KeyCode::KEY_C, 1), Some(KeyEvent::OtherDown));
    }

    #[test]
    fn own_virtual_keyboard_is_not_monitored() {
        assert!(!is_keyboard(Some(VIRTUAL_KEYBOARD_NAME), true));
        assert!(is_keyboard(Some("Topre REALFORCE 112JP"), true));
        assert!(!is_keyboard(Some("Logitech MX Anywhere 3"), false));
    }

    #[test]
    fn interface_macro_name_matches_shared_constant() {
        // `#[zbus::interface(name = ...)]` needs a literal.
        assert_eq!(<Daemon1 as zbus::object_server::Interface>::name().as_str(), DAEMON_INTERFACE);
    }

    #[test]
    fn only_event_nodes_are_considered() {
        assert!(is_event_node(Path::new("/dev/input/event13")));
        assert!(!is_event_node(Path::new("/dev/input/mice")));
        assert!(!is_event_node(Path::new("/dev/input/by-id")));
    }
}
