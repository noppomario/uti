//! Main window visibility rules (no GTK types)

use std::time::{Duration, Instant};

use super::list::Tab;

/// Shell presses this soon after showing belong to the gesture that showed
/// the window (e.g. the second press of a desktop double click).
pub const SHOW_GRACE: Duration = Duration::from_millis(500);

/// What the UI layer must do after a state change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowCommand {
    /// Select the tab, ask the extension to position at the pointer, then
    /// `present()`. Mouse triggers open the Launcher, everything else the
    /// Clipboard.
    PositionAndPresent(Tab),
    /// Hide the window.
    Hide,
    /// Tell the extension to keep the window above others (or not).
    SetAbove(bool),
    /// Ask the extension whether the pointer is over the desktop; answer
    /// with [`WindowState::pointer_on_desktop`].
    QueryPointerOnDesktop,
}

/// Visibility and pin state of the main window.
#[derive(Debug, Default)]
pub struct WindowState {
    visible: bool,
    pinned: bool,
    shown_at: Option<Instant>,
    desktop_double_click: bool,
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

    /// Double Ctrl / panel click. Ignored while pinned.
    pub fn toggle(&mut self, now: Instant) -> Option<WindowCommand> {
        if self.pinned {
            None
        } else if self.visible {
            self.hide()
        } else {
            self.show(now)
        }
    }

    /// Explicit `show` action.
    pub fn show(&mut self, now: Instant) -> Option<WindowCommand> {
        self.show_tab(Tab::Clipboard, now)
    }

    fn show_tab(&mut self, tab: Tab, now: Instant) -> Option<WindowCommand> {
        if self.visible {
            return None;
        }
        self.open(tab, now)
    }

    fn open(&mut self, tab: Tab, now: Instant) -> Option<WindowCommand> {
        self.visible = true;
        self.shown_at = Some(now);
        Some(WindowCommand::PositionAndPresent(tab))
    }

    /// Enables or disables showing on a desktop double click.
    pub fn set_desktop_double_click(&mut self, enabled: bool) {
        self.desktop_double_click = enabled;
    }

    /// The daemon saw a left double click somewhere. While hidden and the
    /// feature is enabled, find out whether it was on the desktop.
    pub fn double_clicked(&self) -> Option<WindowCommand> {
        (self.desktop_double_click && !self.visible).then_some(WindowCommand::QueryPointerOnDesktop)
    }

    /// The extension answered [`WindowCommand::QueryPointerOnDesktop`].
    pub fn pointer_on_desktop(&mut self, on_desktop: bool, now: Instant) -> Option<WindowCommand> {
        if on_desktop && self.desktop_double_click {
            self.show_tab(Tab::Launcher, now)
        } else {
            None
        }
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

    /// GNOME Shell itself (desktop background, panel) was clicked: a click
    /// outside the window that does not move keyboard focus.
    pub fn shell_pressed(&mut self, now: Instant) -> Option<WindowCommand> {
        let just_shown =
            self.shown_at.is_some_and(|t| now.saturating_duration_since(t) < SHOW_GRACE);
        if self.pinned || just_shown {
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

    fn at(ms: u64) -> Instant {
        BASE.with(|b| *b + Duration::from_millis(ms))
    }

    thread_local! {
        static BASE: Instant = Instant::now();
    }

    #[test]
    fn toggle_shows_then_hides() {
        let mut s = WindowState::new();
        assert_eq!(s.toggle(at(0)), Some(PositionAndPresent(Tab::Clipboard)));
        assert!(s.is_visible());
        assert_eq!(s.toggle(at(0)), Some(Hide));
        assert!(!s.is_visible());
    }

    #[test]
    fn toggle_is_ignored_while_pinned() {
        let mut s = WindowState::new();
        s.toggle(at(0));
        s.set_pinned(true);
        assert_eq!(s.toggle(at(0)), None);
        assert!(s.is_visible());
    }

    #[test]
    fn focus_loss_hides_unless_pinned() {
        let mut s = WindowState::new();
        s.toggle(at(0));
        assert_eq!(s.focus_lost(), Some(Hide));

        s.toggle(at(0));
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
    fn shell_press_hides_unless_pinned() {
        let mut s = WindowState::new();
        s.show(at(0));
        assert_eq!(s.shell_pressed(at(1000)), Some(Hide));
        assert!(!s.is_visible());

        s.show(at(0));
        s.set_pinned(true);
        assert_eq!(s.shell_pressed(at(1000)), None);
        assert!(s.is_visible());
    }

    #[test]
    fn shell_press_while_hidden_does_nothing() {
        let mut s = WindowState::new();
        assert_eq!(s.shell_pressed(at(1000)), None);
    }

    #[test]
    fn show_presents_only_when_hidden() {
        let mut s = WindowState::new();
        assert_eq!(s.show(at(0)), Some(PositionAndPresent(Tab::Clipboard)));
        assert_eq!(s.show(at(0)), None);
    }

    #[test]
    fn hide_works_even_when_pinned() {
        let mut s = WindowState::new();
        s.show(at(0));
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

    #[test]
    fn shell_press_right_after_showing_is_ignored() {
        let mut s = WindowState::new();
        s.show(at(0));
        assert_eq!(s.shell_pressed(at(100)), None);
        assert!(s.is_visible());
        assert_eq!(s.shell_pressed(at(600)), Some(Hide));
    }

    #[test]
    fn desktop_double_click_is_off_by_default() {
        let mut s = WindowState::new();
        assert_eq!(s.double_clicked(), None);
        assert_eq!(s.pointer_on_desktop(true, at(0)), None);
        assert!(!s.is_visible());
    }

    #[test]
    fn desktop_double_click_shows_when_on_desktop() {
        let mut s = WindowState::new();
        s.set_desktop_double_click(true);
        assert_eq!(s.double_clicked(), Some(QueryPointerOnDesktop));
        assert_eq!(s.pointer_on_desktop(false, at(0)), None);
        assert_eq!(s.pointer_on_desktop(true, at(0)), Some(PositionAndPresent(Tab::Launcher)));
        assert!(s.is_visible());
    }

    #[test]
    fn double_click_while_visible_is_ignored() {
        let mut s = WindowState::new();
        s.set_desktop_double_click(true);
        s.show(at(0));
        assert_eq!(s.double_clicked(), None);
        assert_eq!(s.pointer_on_desktop(true, at(100)), None);
    }

    #[test]
    fn disabling_before_the_answer_cancels_the_show() {
        let mut s = WindowState::new();
        s.set_desktop_double_click(true);
        s.double_clicked();
        s.set_desktop_double_click(false);
        assert_eq!(s.pointer_on_desktop(true, at(0)), None);
    }
}
