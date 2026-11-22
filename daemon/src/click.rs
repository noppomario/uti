//! Left double click detection
//!
//! Location-independent: the app asks the extension where the pointer is.

use std::time::{Duration, Instant};

/// Maximum time between the two presses (GNOME's default double-click time).
pub const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(400);

/// Detects two left presses in quick succession.
#[derive(Debug, Default)]
pub struct DoubleClickDetector {
    last_press: Option<Instant>,
}

impl DoubleClickDetector {
    /// Creates a detector with no previous press.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds one left press and returns `true` when it completes a double click.
    pub fn press(&mut self, now: Instant) -> bool {
        match self.last_press.take() {
            Some(previous) if now - previous < DOUBLE_CLICK_INTERVAL => true,
            _ => {
                self.last_press = Some(now);
                false
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Feeds presses at `offsets_ms` and returns the offsets that fired.
    fn run(offsets_ms: &[u64]) -> Vec<u64> {
        let base = Instant::now();
        let mut detector = DoubleClickDetector::new();
        offsets_ms
            .iter()
            .copied()
            .filter(|t| detector.press(base + Duration::from_millis(*t)))
            .collect()
    }

    #[test]
    fn two_quick_presses_fire_on_the_second() {
        assert_eq!(run(&[0, 200]), vec![200]);
    }

    #[test]
    fn slow_presses_do_not_fire() {
        assert!(run(&[0, 400]).is_empty());
    }

    #[test]
    fn slow_press_starts_a_new_double_click() {
        assert_eq!(run(&[0, 500, 700]), vec![700]);
    }

    #[test]
    fn triple_click_fires_once() {
        assert_eq!(run(&[0, 150, 300]), vec![150]);
    }
}
