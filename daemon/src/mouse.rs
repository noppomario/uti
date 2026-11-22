//! Left-held + right click on grabbed mice
//!
//! While grabbed, a mouse's events reach the compositor only through a uinput
//! mirror, so the right press of the gesture can be dropped and the app under
//! the pointer opens no context menu.

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, Device, KeyCode, RelativeAxisCode};
use std::io;

/// Name of the uinput mirrors; the device monitor skips them.
pub const VIRTUAL_MOUSE_NAME: &str = "uti Virtual Mouse";

/// What to do with one button event of a grabbed mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verdict {
    /// Re-emit the event through the mirror.
    pub forward: bool,
    /// The event completed a left-held + right click.
    pub chord: bool,
}

const FORWARD: Verdict = Verdict {
    forward: true,
    chord: false,
};

/// Finds right presses made while left is held, and drops them with their
/// release.
#[derive(Debug, Default)]
pub struct ChordFilter {
    left_held: bool,
    swallowing_right: bool,
}

impl ChordFilter {
    /// Creates a filter with no button held.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds one key/button event (`value`: 1 press, 0 release).
    pub fn feed(&mut self, code: KeyCode, value: i32) -> Verdict {
        match (code, value) {
            (KeyCode::BTN_LEFT, 1) => self.left_held = true,
            (KeyCode::BTN_LEFT, 0) => self.left_held = false,
            (KeyCode::BTN_RIGHT, 1) if self.left_held => {
                self.swallowing_right = true;
                return Verdict {
                    forward: false,
                    chord: true,
                };
            },
            (KeyCode::BTN_RIGHT, 0) if self.swallowing_right => {
                self.swallowing_right = false;
                return Verdict {
                    forward: false,
                    chord: false,
                };
            },
            _ => {},
        }
        FORWARD
    }
}

/// Mice with real left and right buttons and relative motion. Touchpads
/// (absolute axes) are left alone: their clicks and taps come from libinput.
pub fn is_grabbable(has_buttons: bool, has_rel_xy: bool, has_abs: bool) -> bool {
    has_buttons && has_rel_xy && !has_abs
}

/// Whether `device` qualifies for [`is_grabbable`].
pub fn device_is_grabbable(device: &Device) -> bool {
    let keys = device.supported_keys();
    let rel = device.supported_relative_axes();
    is_grabbable(
        keys.is_some_and(|k| k.contains(KeyCode::BTN_LEFT) && k.contains(KeyCode::BTN_RIGHT)),
        rel.is_some_and(|r| {
            r.contains(RelativeAxisCode::REL_X) && r.contains(RelativeAxisCode::REL_Y)
        }),
        device.supported_absolute_axes().is_some_and(|a| a.iter().next().is_some()),
    )
}

/// Creates a uinput device with the buttons, axes and ids of `device`, so
/// libinput treats it like the original.
///
/// # Errors
///
/// Returns an error if `/dev/uinput` is not accessible or creation fails.
pub fn mirror(device: &Device) -> io::Result<VirtualDevice> {
    let empty_keys = AttributeSet::<KeyCode>::new();
    let empty_rel = AttributeSet::<RelativeAxisCode>::new();
    VirtualDevice::builder()?
        .name(VIRTUAL_MOUSE_NAME)
        .input_id(device.input_id())
        .with_keys(device.supported_keys().unwrap_or(&empty_keys))?
        .with_relative_axes(device.supported_relative_axes().unwrap_or(&empty_rel))?
        .with_properties(device.properties())?
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEFT: KeyCode = KeyCode::BTN_LEFT;
    const RIGHT: KeyCode = KeyCode::BTN_RIGHT;

    fn feed(events: &[(KeyCode, i32)]) -> Vec<Verdict> {
        let mut filter = ChordFilter::new();
        events.iter().map(|(c, v)| filter.feed(*c, *v)).collect()
    }

    const DROP: Verdict = Verdict {
        forward: false,
        chord: false,
    };
    const CHORD: Verdict = Verdict {
        forward: false,
        chord: true,
    };

    #[test]
    fn right_while_left_held_is_a_chord_and_dropped() {
        let v = feed(&[(LEFT, 1), (RIGHT, 1), (RIGHT, 0), (LEFT, 0)]);
        assert_eq!(v, vec![FORWARD, CHORD, DROP, FORWARD]);
    }

    #[test]
    fn plain_right_click_passes_through() {
        assert_eq!(feed(&[(RIGHT, 1), (RIGHT, 0)]), vec![FORWARD, FORWARD]);
    }

    #[test]
    fn right_release_after_left_release_is_still_dropped() {
        let v = feed(&[(LEFT, 1), (RIGHT, 1), (LEFT, 0), (RIGHT, 0)]);
        assert_eq!(v, vec![FORWARD, CHORD, FORWARD, DROP]);
    }

    #[test]
    fn right_held_before_left_is_not_a_chord() {
        let v = feed(&[(RIGHT, 1), (LEFT, 1), (LEFT, 0), (RIGHT, 0)]);
        assert_eq!(v, vec![FORWARD; 4]);
    }

    #[test]
    fn other_buttons_pass_through() {
        let v = feed(&[(LEFT, 1), (KeyCode::BTN_MIDDLE, 1), (KeyCode::BTN_MIDDLE, 0)]);
        assert_eq!(v, vec![FORWARD; 3]);
    }

    #[test]
    fn only_mice_without_absolute_axes_are_grabbed() {
        assert!(is_grabbable(true, true, false));
        assert!(!is_grabbable(true, true, true), "touchpad");
        assert!(!is_grabbable(false, true, false), "no buttons");
        assert!(!is_grabbable(true, false, false), "no motion");
    }
}
