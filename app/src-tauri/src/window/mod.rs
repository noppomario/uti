//! Window management module
//!
//! Provides window control commands and pin state management.

mod commands;

pub use commands::{hide_for_paste, set_pinned, show_window, toggle_window, type_text};

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Application state for window pin functionality
///
/// Tracks whether the window is pinned (always-on-top with auto-hide disabled).
pub struct PinState {
    pub is_pinned: Arc<AtomicBool>,
}

impl PinState {
    pub fn new() -> Self {
        Self {
            is_pinned: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for PinState {
    fn default() -> Self {
        Self::new()
    }
}
