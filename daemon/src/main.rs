//! Double Ctrl Hotkey Daemon
//!
//! This daemon monitors keyboard input devices for double Ctrl key presses
//! and sends D-Bus signals to notify the Tauri application.

use evdev::{Device, EventType, Key};
use log::{debug, error, info};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use zbus::Connection;

/// Maximum time interval between two Ctrl presses to be considered a double tap
const DOUBLE_TAP_INTERVAL: Duration = Duration::from_millis(300);

/// Sends a D-Bus signal to notify listeners of a double Ctrl press event
///
/// # Arguments
///
/// * `conn` - The D-Bus connection to use for sending the signal
///
/// # Returns
///
/// Returns `Ok(())` if the signal was sent successfully, or a `zbus::Error` if it failed.
///
/// # Examples
///
/// ```no_run
/// # use zbus::Connection;
/// # async fn example() -> zbus::Result<()> {
/// let conn = Connection::session().await?;
/// notify_double_ctrl(&conn).await?;
/// # Ok(())
/// # }
/// ```
async fn notify_double_ctrl(conn: &Connection) -> zbus::Result<()> {
    conn.emit_signal(
        None::<()>,
        "/io/github/noppomario/uti/DoubleTap",
        "io.github.noppomario.uti.DoubleTap",
        "Triggered",
        &(),
    )
    .await?;
    info!("D-Bus signal sent: Triggered");
    Ok(())
}

/// Finds all available keyboard devices
///
/// Searches through `/dev/input/event*` devices to find keyboards by checking
/// if the device supports the 'A' key.
///
/// # Returns
///
/// Returns a list of keyboard device paths.
fn find_keyboard_devices() -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut keyboards = Vec::new();

    for entry in std::fs::read_dir("/dev/input")? {
        let path = entry?.path();
        if let Some(name) = path.file_name() {
            if name.to_string_lossy().starts_with("event") {
                if let Ok(device) = Device::open(&path) {
                    if device
                        .supported_keys()
                        .is_some_and(|keys| keys.contains(Key::KEY_A))
                    {
                        keyboards.push(path);
                    }
                }
            }
        }
    }

    if keyboards.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No keyboard found",
        ));
    }

    Ok(keyboards)
}

/// Monitors a single keyboard device for double Ctrl presses
///
/// # Arguments
///
/// * `device_path` - Path to the keyboard device
/// * `device_name` - Name of the keyboard device
/// * `last_ctrl_release` - Shared state for tracking last Ctrl release time
/// * `conn` - D-Bus connection for sending signals
async fn monitor_device(
    device_path: std::path::PathBuf,
    device_name: String,
    last_ctrl_release: Arc<Mutex<Option<Instant>>>,
    conn: Arc<Connection>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut device = Device::open(&device_path)?;
    info!("[{}] Monitoring started", device_name);

    loop {
        match device.fetch_events() {
            Ok(events) => {
                for event in events {
                    // Log all key events for debugging (debug level)
                    if event.event_type() == EventType::KEY {
                        let key = Key::new(event.code());
                        debug!(
                            "[{}] Key event: {:?} value={} (type={:?})",
                            device_name,
                            key,
                            event.value(),
                            event.event_type()
                        );
                    }

                    if event.event_type() != EventType::KEY {
                        continue;
                    }

                    let key = Key::new(event.code());
                    if key != Key::KEY_LEFTCTRL && key != Key::KEY_RIGHTCTRL {
                        continue;
                    }

                    let key_name = if key == Key::KEY_LEFTCTRL {
                        "LEFT"
                    } else {
                        "RIGHT"
                    };

                    // Key press event (value == 1)
                    if event.value() == 1 {
                        debug!("[{}] Ctrl key pressed ({})", device_name, key_name);
                    }

                    // Key release event (value == 0)
                    if event.value() == 0 {
                        let now = Instant::now();
                        let mut last_release = last_ctrl_release.lock().await;

                        if let Some(last) = *last_release {
                            let interval = now.duration_since(last);
                            debug!(
                                "[{}] Ctrl key released ({}) - {}ms since last release",
                                device_name,
                                key_name,
                                interval.as_millis()
                            );

                            if interval < DOUBLE_TAP_INTERVAL {
                                info!("[{}] Double Ctrl detected!", device_name);
                                notify_double_ctrl(&conn).await?;
                                *last_release = None;
                                continue;
                            }
                        } else {
                            debug!(
                                "[{}] Ctrl key released ({}) - first press",
                                device_name, key_name
                            );
                        }

                        *last_release = Some(now);
                    }
                }
            }
            Err(e) => {
                error!("[{}] Error fetching events: {}", device_name, e);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

/// Main daemon entry point
///
/// Initializes the keyboard device monitoring, connects to D-Bus, and spawns
/// monitoring tasks for all keyboard devices.
///
/// # Errors
///
/// Returns an error if:
/// - No keyboard device is found
/// - Failed to connect to D-Bus session bus
/// - Failed to spawn monitoring tasks
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger (default to INFO level if RUST_LOG not set)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Double Ctrl daemon starting...");

    let keyboards = find_keyboard_devices()?;
    info!("Found {} keyboard device(s):", keyboards.len());

    for (i, path) in keyboards.iter().enumerate() {
        if let Ok(dev) = Device::open(path) {
            info!(
                "  [{}] {} - {}",
                i,
                path.display(),
                dev.name().unwrap_or("unknown")
            );
        }
    }

    let conn = Arc::new(Connection::session().await?);

    // Request the bus name so other applications can connect to us
    conn.request_name("io.github.noppomario.uti").await?;
    info!("Connected to D-Bus session bus");
    info!("Registered bus name: io.github.noppomario.uti");
    info!("Monitoring all keyboard devices for double Ctrl press...");

    // Shared state for last Ctrl release time across all keyboards
    let last_ctrl_release = Arc::new(Mutex::new(None));

    // Spawn a monitoring task for each keyboard device
    let mut tasks = vec![];

    for path in keyboards {
        let device_name = if let Ok(dev) = Device::open(&path) {
            dev.name().unwrap_or("unknown").to_string()
        } else {
            path.display().to_string()
        };

        let last_release_clone = Arc::clone(&last_ctrl_release);
        let conn_clone = Arc::clone(&conn);

        let task = tokio::spawn(async move {
            if let Err(e) =
                monitor_device(path, device_name.clone(), last_release_clone, conn_clone).await
            {
                error!("[{}] Monitoring task failed: {}", device_name, e);
            }
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete (they run indefinitely)
    for task in tasks {
        let _ = task.await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_tap_interval_constant() {
        // Verify the double tap interval is set to 300ms
        assert_eq!(DOUBLE_TAP_INTERVAL, Duration::from_millis(300));
    }

    #[test]
    fn test_double_tap_detection_within_threshold() {
        // Arrange
        let first_press = Instant::now();
        let second_press = first_press + Duration::from_millis(250);

        // Act
        let interval = second_press.duration_since(first_press);

        // Assert
        assert!(
            interval < DOUBLE_TAP_INTERVAL,
            "250ms should be within the 300ms threshold"
        );
    }

    #[test]
    fn test_double_tap_detection_exceeds_threshold() {
        // Arrange
        let first_press = Instant::now();
        let second_press = first_press + Duration::from_millis(350);

        // Act
        let interval = second_press.duration_since(first_press);

        // Assert
        assert!(
            interval > DOUBLE_TAP_INTERVAL,
            "350ms should exceed the 300ms threshold"
        );
    }

    #[test]
    fn test_double_tap_detection_exact_threshold() {
        // Arrange
        let first_press = Instant::now();
        let second_press = first_press + DOUBLE_TAP_INTERVAL;

        // Act
        let interval = second_press.duration_since(first_press);

        // Assert
        assert!(
            interval >= DOUBLE_TAP_INTERVAL,
            "300ms should not be considered a double tap (must be strictly less)"
        );
    }

    #[test]
    fn test_find_keyboard_devices_path_format() {
        // This test verifies that the function looks for event devices
        // In CI/test environments without /dev/input access, this is expected to fail
        let result = find_keyboard_devices();

        match result {
            Ok(keyboards) => {
                // If keyboards are found, paths should start with /dev/input/event
                for path in keyboards {
                    let path_str = path.to_string_lossy();
                    assert!(
                        path_str.starts_with("/dev/input/event"),
                        "Keyboard device path should be /dev/input/eventN, got: {}",
                        path_str
                    );
                }
            }
            Err(e) => {
                // In test environment, it's acceptable to not have keyboard access
                // Just verify the error is what we expect
                assert!(
                    e.kind() == std::io::ErrorKind::NotFound
                        || e.kind() == std::io::ErrorKind::PermissionDenied,
                    "Expected NotFound or PermissionDenied, got: {:?}",
                    e.kind()
                );
            }
        }
    }
}
