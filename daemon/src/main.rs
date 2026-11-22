//! uti input daemon
//!
//! Watches all keyboards and mice (including ones plugged in later) for a
//! double Ctrl tap and a left double click, announced on D-Bus as
//! `io.github.noppomario.uti.Daemon1.Triggered` and `DoubleClicked`. While
//! the app enables it (`SetMouseGrab`), mice are grabbed and mirrored through
//! uinput so a left-held + right click (`LeftRightClicked`) opens no menu.
//! Also offers `Daemon1.Paste` as a uinput fallback for the GNOME extension's
//! paste; only the owner of the app's bus name may call it.

mod click;
mod detector;
mod mouse;
mod uinput;

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use click::DoubleClickDetector;
use detector::{DoubleTapDetector, KeyEvent};
use evdev::uinput::VirtualDevice;
use evdev::{Device, EventStream, EventType, InputEvent, KeyCode, SynchronizationCode};
use futures_util::StreamExt;
use inotify::{Inotify, WatchMask};
use log::{debug, error, info, warn};
use mouse::{ChordFilter, VIRTUAL_MOUSE_NAME};
use tokio::sync::{mpsc, watch};
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

/// Events the device readers forward to the main loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Input {
    Key(KeyEvent),
    LeftPress,
    /// Right press while left is held, on a grabbed mouse.
    LeftRight,
}

/// What a device is monitored for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeviceKind {
    keyboard: bool,
    pointer: bool,
}

impl DeviceKind {
    /// Real keyboards (`KEY_A`) and pointers (`BTN_LEFT`), excluding uti's
    /// own virtual devices.
    fn of(name: Option<&str>, has_key_a: bool, has_btn_left: bool) -> Self {
        let own = matches!(name, Some(VIRTUAL_KEYBOARD_NAME | VIRTUAL_MOUSE_NAME));
        Self {
            keyboard: has_key_a && !own,
            pointer: has_btn_left && !own,
        }
    }

    fn is_monitored(self) -> bool {
        self.keyboard || self.pointer
    }

    /// Maps a raw key/button event of this device to an [`Input`].
    fn input(self, code: KeyCode, value: i32) -> Option<Input> {
        if code == KeyCode::BTN_LEFT {
            return (self.pointer && value == 1).then_some(Input::LeftPress);
        }
        if self.keyboard {
            classify(code, value).map(Input::Key)
        } else {
            None
        }
    }
}

fn is_event_node(path: &Path) -> bool {
    path.file_name().is_some_and(|n| n.to_string_lossy().starts_with("event"))
}

/// Tracks monitored device nodes and spawns one reader task per device.
#[derive(Clone)]
struct Devices {
    active: Arc<Mutex<HashSet<PathBuf>>>,
    tx: mpsc::UnboundedSender<Input>,
    /// Whether grabbable mice should be grabbed (set by the app).
    grab: watch::Receiver<bool>,
}

/// A grabbed mouse: its events are filtered and re-emitted through `mirror`.
struct Grab {
    mirror: VirtualDevice,
    filter: ChordFilter,
    batch: Vec<InputEvent>,
}

impl Grab {
    /// Creates the mirror, then grabs the device; `None` (not grabbed) on failure.
    fn start(stream: &mut EventStream, name: &str) -> Option<Self> {
        let mirror = match mouse::mirror(stream.device()) {
            Ok(m) => m,
            Err(e) => {
                warn!("[{name}] cannot create mirror, not grabbing: {e}");
                return None;
            },
        };
        if let Err(e) = stream.device_mut().grab() {
            warn!("[{name}] cannot grab: {e}");
            return None;
        }
        info!("[{name}] grabbed");
        Some(Self {
            mirror,
            filter: ChordFilter::new(),
            batch: Vec::new(),
        })
    }

    /// Filters and re-emits `ev`; returns whether it completed the gesture.
    fn handle(&mut self, ev: InputEvent) -> io::Result<bool> {
        match ev.event_type() {
            EventType::SYNCHRONIZATION => {
                if ev.code() == SynchronizationCode::SYN_REPORT.0 && !self.batch.is_empty() {
                    self.mirror.emit(&self.batch)?;
                    self.batch.clear();
                }
                Ok(false)
            },
            EventType::KEY => {
                let verdict = self.filter.feed(KeyCode::new(ev.code()), ev.value());
                if verdict.forward {
                    self.batch.push(ev);
                }
                Ok(verdict.chord)
            },
            // MSC_SCAN and the like: not declared on the mirror.
            EventType::MISC => Ok(false),
            _ => {
                self.batch.push(ev);
                Ok(false)
            },
        }
    }
}

impl Devices {
    /// Starts monitoring `path` if it is a keyboard or pointer not monitored yet.
    fn try_add(&self, path: PathBuf) {
        if !is_event_node(&path) || self.active.lock().unwrap().contains(&path) {
            return;
        }
        let Ok(device) = Device::open(&path) else {
            return; // Not readable (yet); udev may fix permissions (IN_ATTRIB).
        };
        let keys = device.supported_keys();
        let name = device.name().unwrap_or("unknown").to_owned();
        let kind = DeviceKind::of(
            Some(&name),
            keys.is_some_and(|k| k.contains(KeyCode::KEY_A)),
            keys.is_some_and(|k| k.contains(KeyCode::BTN_LEFT)),
        );
        if !kind.is_monitored() {
            return;
        }
        let grabbable = kind.pointer && mouse::device_is_grabbable(&device);
        let stream = match device.into_event_stream() {
            Ok(s) => s,
            Err(e) => return warn!("[{name}] cannot stream events: {e}"),
        };
        self.active.lock().unwrap().insert(path.clone());
        info!("[{name}] monitoring {}", path.display());

        let this = self.clone();
        tokio::spawn(async move {
            let mut stream = stream;
            let mut grab_rx = this.grab.clone();
            let mut grab = if grabbable && *grab_rx.borrow_and_update() {
                Grab::start(&mut stream, &name)
            } else {
                None
            };
            loop {
                tokio::select! {
                    ev = stream.next_event() => {
                        let ev = match ev {
                            Ok(ev) => ev,
                            Err(e) => {
                                info!("[{name}] stopped: {e}");
                                break;
                            },
                        };
                        if let Some(g) = grab.as_mut() {
                            match g.handle(ev) {
                                Ok(true) => {
                                    if this.tx.send(Input::LeftRight).is_err() {
                                        break;
                                    }
                                },
                                Ok(false) => {},
                                Err(e) => {
                                    // Never keep a grab whose events go nowhere.
                                    error!("[{name}] mirror failed, releasing grab: {e}");
                                    let _ = stream.device_mut().ungrab();
                                    grab = None;
                                },
                            }
                        }
                        if ev.event_type() == EventType::KEY {
                            if let Some(input) = kind.input(KeyCode::new(ev.code()), ev.value()) {
                                if this.tx.send(input).is_err() {
                                    break;
                                }
                            }
                        }
                    },
                    Ok(()) = grab_rx.changed(), if grabbable => {
                        let wanted = *grab_rx.borrow_and_update();
                        if wanted && grab.is_none() {
                            grab = Grab::start(&mut stream, &name);
                        } else if !wanted && grab.is_some() {
                            // Dropping the mirror releases anything it holds.
                            grab = None;
                            match stream.device_mut().ungrab() {
                                Ok(()) => info!("[{name}] released"),
                                Err(e) => warn!("[{name}] cannot release grab: {e}"),
                            }
                        }
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

/// Rejects callers that do not own the app's bus name.
async fn check_caller(method: &str, header: &Header<'_>, conn: &Connection) -> fdo::Result<()> {
    let sender = header.sender().map(|s| s.to_string());
    let owner = fdo::DBusProxy::new(conn)
        .await?
        .get_name_owner(zbus::names::BusName::try_from(APP_BUS_NAME)?)
        .await
        .ok()
        .map(|o| o.to_string());
    if sender.is_none() || sender != owner {
        warn!("{method} denied for {sender:?} (owner {owner:?})");
        return Err(fdo::Error::AccessDenied(format!("caller does not own {APP_BUS_NAME}")));
    }
    Ok(())
}

/// `io.github.noppomario.uti.Daemon1`
struct Daemon1 {
    keyboard: Arc<Mutex<Option<VirtualKeyboard>>>,
    grab: Arc<watch::Sender<bool>>,
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
        check_caller("Paste", &header, conn).await?;

        let keyboard = Arc::clone(&self.keyboard);
        tokio::task::spawn_blocking(move || match keyboard.lock().unwrap().as_mut() {
            Some(kb) => kb.paste(plain, submit).map_err(|e| e.to_string()),
            None => Err("virtual keyboard unavailable".into()),
        })
        .await
        .map_err(|e| fdo::Error::Failed(e.to_string()))?
        .map_err(fdo::Error::Failed)
    }

    /// Grabs mice (or releases them) for left-held + right click detection.
    /// Released again when the app leaves the bus.
    async fn set_mouse_grab(
        &self,
        enabled: bool,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] conn: &Connection,
    ) -> fdo::Result<()> {
        check_caller("SetMouseGrab", &header, conn).await?;
        info!("mouse grab {}", if enabled { "enabled" } else { "disabled" });
        self.grab.send_replace(enabled);
        Ok(())
    }

    /// Emitted on a double Ctrl tap.
    #[zbus(signal)]
    async fn triggered(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// Emitted on a left double click, wherever the pointer is.
    #[zbus(signal)]
    async fn double_clicked(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// Emitted on a right press while left is held, on a grabbed mouse.
    #[zbus(signal)]
    async fn left_right_clicked(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
}

/// Signals of [`Daemon1`].
#[derive(Debug, Clone, Copy)]
enum Signal {
    Triggered,
    DoubleClicked,
    LeftRightClicked,
}

async fn announce(conn: &Connection, signal: Signal) {
    let emitter = match SignalEmitter::new(conn, OBJECT_PATH) {
        Ok(emitter) => emitter,
        Err(e) => return error!("invalid object path: {e}"),
    };
    let result = match signal {
        Signal::Triggered => Daemon1::triggered(&emitter).await,
        Signal::DoubleClicked => Daemon1::double_clicked(&emitter).await,
        Signal::LeftRightClicked => Daemon1::left_right_clicked(&emitter).await,
    };
    if let Err(e) = result {
        error!("failed to emit {signal:?}: {e}");
    }
}

/// Turns the mouse grab off whenever the app's bus name loses its owner.
async fn release_grab_when_app_leaves(conn: Connection, grab: Arc<watch::Sender<bool>>) {
    let result: zbus::Result<()> = async {
        let proxy = fdo::DBusProxy::new(&conn).await?;
        let mut changes = proxy.receive_name_owner_changed_with_args(&[(0, APP_BUS_NAME)]).await?;
        while let Some(change) = changes.next().await {
            if change.args()?.new_owner().is_none() && grab.send_replace(false) {
                info!("app left the bus, mouse grab disabled");
            }
        }
        Ok(())
    }
    .await;
    if let Err(e) = result {
        error!("cannot watch {APP_BUS_NAME}: {e}");
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

    let (grab_tx, grab_rx) = watch::channel(false);
    let grab_tx = Arc::new(grab_tx);
    let conn = zbus::connection::Builder::session()
        .name(BUS_NAME)
        .serve_at(
            OBJECT_PATH,
            Daemon1 {
                keyboard: Arc::new(Mutex::new(keyboard)),
                grab: Arc::clone(&grab_tx),
            },
        )
        .build()
        .await?;
    info!("D-Bus ready as {BUS_NAME}");
    tokio::spawn(release_grab_when_app_leaves(conn.clone(), grab_tx));

    let (tx, mut rx) = mpsc::unbounded_channel();
    let devices = Devices {
        active: Arc::default(),
        tx,
        grab: grab_rx,
    };
    devices.scan();
    let hotplug = devices.clone();
    tokio::spawn(async move {
        if let Err(e) = watch_hotplug(hotplug).await {
            error!("hotplug watch failed, new devices will be ignored: {e}");
        }
    });

    let mut ctrl = DoubleTapDetector::new();
    let mut click = DoubleClickDetector::new();
    while let Some(input) = rx.recv().await {
        let now = Instant::now();
        match input {
            Input::Key(event) if ctrl.feed(event, now) => {
                info!("double Ctrl detected");
                announce(&conn, Signal::Triggered).await;
            },
            Input::LeftPress if click.press(now) => {
                debug!("double click detected");
                announce(&conn, Signal::DoubleClicked).await;
            },
            Input::LeftRight => {
                debug!("left-held + right click detected");
                announce(&conn, Signal::LeftRightClicked).await;
            },
            _ => {},
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
    fn own_virtual_devices_are_not_monitored() {
        assert!(!DeviceKind::of(Some(VIRTUAL_KEYBOARD_NAME), true, false).is_monitored());
        assert!(!DeviceKind::of(Some(VIRTUAL_MOUSE_NAME), false, true).is_monitored());
        assert!(DeviceKind::of(Some("Topre REALFORCE 112JP"), true, false).keyboard);
    }

    #[test]
    fn mice_are_monitored_for_left_presses_only() {
        let mouse = DeviceKind::of(Some("Logitech MX Anywhere 3"), false, true);
        assert!(mouse.is_monitored() && !mouse.keyboard);
        assert_eq!(mouse.input(KeyCode::BTN_LEFT, 1), Some(Input::LeftPress));
        assert_eq!(mouse.input(KeyCode::BTN_LEFT, 0), None);
        assert_eq!(mouse.input(KeyCode::BTN_RIGHT, 1), None);
        assert_eq!(mouse.input(KeyCode::KEY_LEFTCTRL, 1), None);
    }

    #[test]
    fn combined_receiver_reports_keys_and_clicks() {
        let combo = DeviceKind::of(Some("Logitech USB Receiver"), true, true);
        assert_eq!(combo.input(KeyCode::KEY_LEFTCTRL, 1), Some(Input::Key(KeyEvent::CtrlDown)));
        assert_eq!(combo.input(KeyCode::BTN_LEFT, 1), Some(Input::LeftPress));
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
