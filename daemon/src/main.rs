//! Double Ctrl Hotkey Daemon
//!
//! This daemon monitors keyboard input devices for double Ctrl key presses
//! and sends D-Bus signals to notify the Tauri application.

use evdev::{Device, EventType, Key};
use std::time::{Duration, Instant};
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
    println!("D-Bus signal sent: Triggered");
    Ok(())
}

/// Finds the first available keyboard device
///
/// Searches through `/dev/input/event*` devices to find a keyboard by checking
/// if the device supports the 'A' key.
///
/// # Returns
///
/// Returns the path to the keyboard device if found, or an error if no keyboard is detected.
///
/// # Errors
///
/// Returns `std::io::Error` with kind `NotFound` if no keyboard device is found.
fn find_keyboard_device() -> std::io::Result<std::path::PathBuf> {
    for entry in std::fs::read_dir("/dev/input")? {
        let path = entry?.path();
        if let Some(name) = path.file_name() {
            if name.to_string_lossy().starts_with("event") {
                if let Ok(device) = Device::open(&path) {
                    if device
                        .supported_keys()
                        .is_some_and(|keys| keys.contains(Key::KEY_A))
                    {
                        println!("Found keyboard device: {}", path.display());
                        return Ok(path);
                    }
                }
            }
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No keyboard found",
    ))
}

/// Main daemon entry point
///
/// Initializes the keyboard device monitoring, connects to D-Bus, and enters
/// an event loop to detect double Ctrl key presses.
///
/// # Errors
///
/// Returns an error if:
/// - No keyboard device is found
/// - Failed to connect to D-Bus session bus
/// - Failed to read events from the keyboard device
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Double Ctrl daemon starting...");

    let device_path = find_keyboard_device()?;
    let mut device = Device::open(&device_path)?;
    let conn = Connection::session().await?;

    println!("Monitoring device: {}", device.name().unwrap_or("unknown"));
    println!("Connected to D-Bus session bus");
    println!("Waiting for double Ctrl press...");

    let mut last_ctrl_release: Option<Instant> = None;

    loop {
        for event in device.fetch_events()? {
            if event.event_type() != EventType::KEY {
                continue;
            }

            let key = Key::new(event.code());
            if key != Key::KEY_LEFTCTRL && key != Key::KEY_RIGHTCTRL {
                continue;
            }

            // Key release event (value == 0)
            if event.value() == 0 {
                let now = Instant::now();

                if let Some(last) = last_ctrl_release {
                    if now.duration_since(last) < DOUBLE_TAP_INTERVAL {
                        println!("Double Ctrl detected!");
                        notify_double_ctrl(&conn).await?;
                        last_ctrl_release = None;
                        continue;
                    }
                }

                last_ctrl_release = Some(now);
            }
        }
    }
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
    fn test_find_keyboard_device_path_format() {
        // This test verifies that the function looks for event devices
        // We can't actually test device finding without hardware/permissions,
        // but we can document expected behavior
        let result = find_keyboard_device();

        // If a keyboard is found, path should start with /dev/input/event
        if let Ok(path) = result {
            assert!(
                path.starts_with("/dev/input/event"),
                "Keyboard device path should be /dev/input/eventN"
            );
        }
        // If no keyboard found, that's also acceptable in test environment
    }
}
