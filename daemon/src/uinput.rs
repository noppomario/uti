//! Virtual keyboard module for simulating key presses
//!
//! This module provides a virtual keyboard implementation using evdev uinput
//! to simulate keyboard input events.

use evdev::uinput::VirtualDeviceBuilder;
use evdev::{AttributeSet, InputEvent, Key};
use log::{debug, error, info};
use std::io;
use std::thread::sleep;
use std::time::Duration;

/// Delay between key events to ensure proper handling
const KEY_EVENT_DELAY: Duration = Duration::from_millis(10);

/// Virtual keyboard device for simulating key presses
///
/// Uses evdev uinput to create a virtual input device that can
/// emit keyboard events.
pub struct VirtualKeyboard {
    device: evdev::uinput::VirtualDevice,
}

impl VirtualKeyboard {
    /// Creates a new virtual keyboard device
    ///
    /// # Returns
    ///
    /// Returns `Ok(VirtualKeyboard)` if the device was created successfully,
    /// or an `io::Error` if creation failed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No permission to access /dev/uinput
    /// - Failed to create the virtual device
    pub fn new() -> io::Result<Self> {
        // Define which keys the virtual device supports
        let mut keys = AttributeSet::<Key>::new();
        keys.insert(Key::KEY_LEFTCTRL);
        keys.insert(Key::KEY_LEFTSHIFT);
        keys.insert(Key::KEY_V);
        keys.insert(Key::KEY_ENTER);

        let device = VirtualDeviceBuilder::new()?
            .name("uti Virtual Keyboard")
            .with_keys(&keys)?
            .build()?;

        info!("Virtual keyboard device created");
        Ok(Self { device })
    }

    /// Simulates Ctrl+Shift+V (paste) followed by Enter
    ///
    /// Uses Ctrl+Shift+V to work in both terminals (where Ctrl+V doesn't work)
    /// and GUI applications (where it pastes as plain text).
    /// After pasting, sends Enter to submit the text (useful for terminals).
    ///
    /// Emits the key events in the following sequence:
    /// 1. Press Ctrl, Shift, V (paste)
    /// 2. Release V, Shift, Ctrl
    /// 3. Press Enter (submit)
    /// 4. Release Enter
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the keystroke was simulated successfully,
    /// or an `io::Error` if it failed.
    pub fn paste(&mut self) -> io::Result<()> {
        debug!("Simulating Ctrl+Shift+V + Enter");

        // Key press values: 1 = press, 0 = release
        const PRESS: i32 = 1;
        const RELEASE: i32 = 0;

        // Press Ctrl
        self.emit_key(Key::KEY_LEFTCTRL, PRESS)?;
        sleep(KEY_EVENT_DELAY);

        // Press Shift
        self.emit_key(Key::KEY_LEFTSHIFT, PRESS)?;
        sleep(KEY_EVENT_DELAY);

        // Press V
        self.emit_key(Key::KEY_V, PRESS)?;
        sleep(KEY_EVENT_DELAY);

        // Release V
        self.emit_key(Key::KEY_V, RELEASE)?;
        sleep(KEY_EVENT_DELAY);

        // Release Shift
        self.emit_key(Key::KEY_LEFTSHIFT, RELEASE)?;
        sleep(KEY_EVENT_DELAY);

        // Release Ctrl
        self.emit_key(Key::KEY_LEFTCTRL, RELEASE)?;
        sleep(KEY_EVENT_DELAY);

        // Press Enter
        self.emit_key(Key::KEY_ENTER, PRESS)?;
        sleep(KEY_EVENT_DELAY);

        // Release Enter
        self.emit_key(Key::KEY_ENTER, RELEASE)?;

        info!("Ctrl+Shift+V + Enter simulated successfully");
        Ok(())
    }

    /// Emits a single key event
    ///
    /// # Arguments
    ///
    /// * `key` - The key to emit
    /// * `value` - 1 for press, 0 for release
    fn emit_key(&mut self, key: Key, value: i32) -> io::Result<()> {
        let events = [
            // Key event
            InputEvent::new(evdev::EventType::KEY, key.code(), value),
            // Sync event to flush the key event
            InputEvent::new(evdev::EventType::SYNCHRONIZATION, 0, 0),
        ];

        if let Err(e) = self.device.emit(&events) {
            error!("Failed to emit key event: {}", e);
            return Err(e);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_delay_constant() {
        assert_eq!(KEY_EVENT_DELAY, Duration::from_millis(10));
    }
}
