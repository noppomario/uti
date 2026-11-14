//! Virtual keyboard for simulating paste
//!
//! Fallback paste path used when the GNOME extension is unavailable. Emits
//! evdev keycodes, so the combo assumes a layout where `KEY_V` is "v".

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, EventType, InputEvent, KeyCode};
use log::{debug, info};
use std::io;
use std::thread::sleep;
use std::time::Duration;

/// Name of the virtual device; the device monitor skips it.
pub const VIRTUAL_KEYBOARD_NAME: &str = "uti Virtual Keyboard";

/// Delay between key events to ensure proper handling
const KEY_EVENT_DELAY: Duration = Duration::from_millis(10);

const PRESS: i32 = 1;
const RELEASE: i32 = 0;

/// Virtual keyboard device for simulating key presses
pub struct VirtualKeyboard {
    device: VirtualDevice,
}

/// Returns the key combo for a paste request.
///
/// `plain` uses Ctrl+Shift+V (terminals, paste without formatting) instead of
/// Ctrl+V; `submit` appends Enter.
pub fn paste_sequence(plain: bool, submit: bool) -> Vec<Vec<KeyCode>> {
    let mut combo = vec![KeyCode::KEY_LEFTCTRL];
    if plain {
        combo.push(KeyCode::KEY_LEFTSHIFT);
    }
    combo.push(KeyCode::KEY_V);

    let mut seq = vec![combo];
    if submit {
        seq.push(vec![KeyCode::KEY_ENTER]);
    }
    seq
}

impl VirtualKeyboard {
    /// Creates the virtual keyboard device
    ///
    /// # Errors
    ///
    /// Returns an error if `/dev/uinput` is not accessible or device creation fails.
    pub fn new() -> io::Result<Self> {
        let mut keys = AttributeSet::<KeyCode>::new();
        for key in [
            KeyCode::KEY_LEFTCTRL,
            KeyCode::KEY_LEFTSHIFT,
            KeyCode::KEY_V,
            KeyCode::KEY_ENTER,
        ] {
            keys.insert(key);
        }

        let device = VirtualDevice::builder()?
            .name(VIRTUAL_KEYBOARD_NAME)
            .with_keys(&keys)?
            .build()?;

        info!("Virtual keyboard device created");
        Ok(Self { device })
    }

    /// Simulates a paste combo, optionally followed by Enter
    ///
    /// # Errors
    ///
    /// Returns an error if emitting an event fails.
    pub fn paste(&mut self, plain: bool, submit: bool) -> io::Result<()> {
        debug!("Simulating paste (plain={plain}, submit={submit})");
        for combo in paste_sequence(plain, submit) {
            for key in &combo {
                self.emit_key(*key, PRESS)?;
            }
            for key in combo.iter().rev() {
                self.emit_key(*key, RELEASE)?;
            }
        }
        Ok(())
    }

    fn emit_key(&mut self, key: KeyCode, value: i32) -> io::Result<()> {
        self.device
            .emit(&[InputEvent::new(EventType::KEY.0, key.code(), value)])?;
        sleep(KEY_EVENT_DELAY);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_submit_is_ctrl_shift_v_then_enter() {
        assert_eq!(
            paste_sequence(true, true),
            vec![
                vec![
                    KeyCode::KEY_LEFTCTRL,
                    KeyCode::KEY_LEFTSHIFT,
                    KeyCode::KEY_V
                ],
                vec![KeyCode::KEY_ENTER],
            ]
        );
    }

    #[test]
    fn default_is_ctrl_v_without_enter() {
        assert_eq!(
            paste_sequence(false, false),
            vec![vec![KeyCode::KEY_LEFTCTRL, KeyCode::KEY_V]]
        );
    }
}
