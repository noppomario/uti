//! Main window visibility rules (no GTK types)

/// What the UI layer must do after a state change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowCommand {
    /// Ask the extension to position at the pointer, then `present()`.
    PositionAndPresent,
    /// Hide the window.
    Hide,
    /// Tell the extension to keep the window above others (or not).
    SetAbove(bool),
}

/// Visibility and pin state of the main window.
#[derive(Debug, Default)]
pub struct WindowState {
    visible: bool,
    pinned: bool,
}

impl WindowState {
    /// Creates the state for a window that starts hidden.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the window is currently shown.
    #[cfg(test)]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Whether the window is pinned (always on top, no auto-hide).
    pub fn is_pinned(&self) -> bool {
        self.pinned
    }

    /// Double Ctrl / panel click. Ignored while pinned.
    pub fn toggle(&mut self) -> Option<WindowCommand> {
        if self.pinned {
            None
        } else if self.visible {
            self.hide()
        } else {
            self.show()
        }
    }

    /// Explicit `show` action.
    pub fn show(&mut self) -> Option<WindowCommand> {
        if self.visible {
            return None;
        }
        self.visible = true;
        Some(WindowCommand::PositionAndPresent)
    }

    /// Explicit `hide` action or an item was chosen; works even when pinned.
    pub fn hide(&mut self) -> Option<WindowCommand> {
        if !self.visible {
            return None;
        }
        self.visible = false;
        Some(WindowCommand::Hide)
    }

    /// The window lost keyboard focus.
    pub fn focus_lost(&mut self) -> Option<WindowCommand> {
        if self.pinned {
            None
        } else {
            self.hide()
        }
    }

    /// Pin button toggled.
    pub fn set_pinned(&mut self, pinned: bool) -> Option<WindowCommand> {
        if self.pinned == pinned {
            return None;
        }
        self.pinned = pinned;
        Some(WindowCommand::SetAbove(pinned))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use WindowCommand::*;

    #[test]
    fn toggle_shows_then_hides() {
        let mut s = WindowState::new();
        assert_eq!(s.toggle(), Some(PositionAndPresent));
        assert!(s.is_visible());
        assert_eq!(s.toggle(), Some(Hide));
        assert!(!s.is_visible());
    }

    #[test]
    fn toggle_is_ignored_while_pinned() {
        let mut s = WindowState::new();
        s.toggle();
        s.set_pinned(true);
        assert_eq!(s.toggle(), None);
        assert!(s.is_visible());
    }

    #[test]
    fn focus_loss_hides_unless_pinned() {
        let mut s = WindowState::new();
        s.toggle();
        assert_eq!(s.focus_lost(), Some(Hide));

        s.toggle();
        s.set_pinned(true);
        assert_eq!(s.focus_lost(), None);
        assert!(s.is_visible());
    }

    #[test]
    fn focus_loss_while_hidden_does_nothing() {
        let mut s = WindowState::new();
        assert_eq!(s.focus_lost(), None);
    }

    #[test]
    fn show_presents_only_when_hidden() {
        let mut s = WindowState::new();
        assert_eq!(s.show(), Some(PositionAndPresent));
        assert_eq!(s.show(), None);
    }

    #[test]
    fn hide_works_even_when_pinned() {
        let mut s = WindowState::new();
        s.show();
        s.set_pinned(true);
        assert_eq!(s.hide(), Some(Hide));
        assert!(!s.is_visible());
        assert_eq!(s.hide(), None);
    }

    #[test]
    fn pin_changes_are_forwarded_once() {
        let mut s = WindowState::new();
        assert_eq!(s.set_pinned(true), Some(SetAbove(true)));
        assert_eq!(s.set_pinned(true), None);
        assert_eq!(s.set_pinned(false), Some(SetAbove(false)));
    }
}
