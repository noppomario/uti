//! Double Ctrl tap detection
//!
//! Pure state machine fed with key events and their timestamps, so the
//! timing rules can be tested without devices.

use std::time::{Duration, Instant};

/// Maximum time between the releases of the first and second tap.
pub const DOUBLE_TAP_INTERVAL: Duration = Duration::from_millis(300);

/// A Ctrl press held at least this long is not a tap.
pub const MAX_TAP_HOLD: Duration = Duration::from_millis(250);

/// Key events relevant to the detector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    /// Left or right Ctrl pressed (not auto-repeat).
    CtrlDown,
    /// Left or right Ctrl released.
    CtrlUp,
    /// Any other key pressed.
    OtherDown,
}

/// Detects two Ctrl taps in quick succession with no other key in between.
#[derive(Debug, Default)]
pub struct DoubleTapDetector {
    state: State,
}

#[derive(Debug, Default, Clone, Copy)]
enum State {
    #[default]
    Idle,
    CtrlHeld {
        since: Instant,
        previous_tap: Option<Instant>,
    },
    Tapped {
        at: Instant,
    },
}

impl DoubleTapDetector {
    /// Creates a detector in the idle state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds one event and returns `true` when a double tap completes.
    pub fn feed(&mut self, event: KeyEvent, now: Instant) -> bool {
        let (next, fired) = match (self.state, event) {
            (State::Idle, KeyEvent::CtrlDown) => (
                State::CtrlHeld {
                    since: now,
                    previous_tap: None,
                },
                false,
            ),
            (State::Tapped { at }, KeyEvent::CtrlDown) => (
                State::CtrlHeld {
                    since: now,
                    previous_tap: (now - at < DOUBLE_TAP_INTERVAL).then_some(at),
                },
                false,
            ),
            (
                State::CtrlHeld {
                    since,
                    previous_tap,
                },
                KeyEvent::CtrlUp,
            ) => {
                if now - since >= MAX_TAP_HOLD {
                    (State::Idle, false)
                } else if previous_tap.is_some_and(|p| now - p < DOUBLE_TAP_INTERVAL) {
                    (State::Idle, true)
                } else {
                    (State::Tapped { at: now }, false)
                }
            }
            // Other keys, a second Ctrl while one is held, or stray releases.
            _ => (State::Idle, false),
        };
        self.state = next;
        fired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    /// Feeds `(offset_ms, event)` pairs and returns the offsets that triggered.
    fn run(events: &[(u64, KeyEvent)]) -> Vec<u64> {
        let base = Instant::now();
        let mut detector = DoubleTapDetector::new();
        events
            .iter()
            .filter(|(t, e)| detector.feed(*e, base + ms(*t)))
            .map(|(t, _)| *t)
            .collect()
    }

    use KeyEvent::{CtrlDown, CtrlUp, OtherDown};

    #[test]
    fn double_tap_triggers_on_second_release() {
        let fired = run(&[(0, CtrlDown), (50, CtrlUp), (150, CtrlDown), (200, CtrlUp)]);
        assert_eq!(fired, vec![200]);
    }

    #[test]
    fn ctrl_c_then_ctrl_v_does_not_trigger() {
        let fired = run(&[
            (0, CtrlDown),
            (30, OtherDown),
            (60, CtrlUp),
            (120, CtrlDown),
            (150, OtherDown),
            (180, CtrlUp),
        ]);
        assert!(fired.is_empty());
    }

    #[test]
    fn other_key_between_taps_cancels() {
        let fired = run(&[
            (0, CtrlDown),
            (50, CtrlUp),
            (100, OtherDown),
            (150, CtrlDown),
            (200, CtrlUp),
        ]);
        assert!(fired.is_empty());
    }

    #[test]
    fn slow_second_tap_does_not_trigger() {
        let fired = run(&[(0, CtrlDown), (50, CtrlUp), (300, CtrlDown), (350, CtrlUp)]);
        assert!(fired.is_empty());
    }

    #[test]
    fn long_hold_is_not_a_tap() {
        let fired = run(&[(0, CtrlDown), (260, CtrlUp), (300, CtrlDown), (340, CtrlUp)]);
        assert!(fired.is_empty());
    }

    #[test]
    fn slow_tap_can_start_a_new_double_tap() {
        let fired = run(&[
            (0, CtrlDown),
            (50, CtrlUp),
            (400, CtrlDown),
            (450, CtrlUp),
            (550, CtrlDown),
            (600, CtrlUp),
        ]);
        assert_eq!(fired, vec![600]);
    }

    #[test]
    fn triple_tap_triggers_once() {
        let fired = run(&[
            (0, CtrlDown),
            (40, CtrlUp),
            (100, CtrlDown),
            (140, CtrlUp),
            (200, CtrlDown),
            (240, CtrlUp),
        ]);
        assert_eq!(fired, vec![140]);
    }

    #[test]
    fn both_ctrls_held_cancels() {
        let fired = run(&[
            (0, CtrlDown),
            (20, CtrlDown),
            (50, CtrlUp),
            (60, CtrlUp),
            (150, CtrlDown),
            (200, CtrlUp),
        ]);
        assert!(fired.is_empty());
    }
}
