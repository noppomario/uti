//! Keyboard focus model of the main window
//!
//! Owns the current tab, where keyboard focus is (list / search / pin) and
//! each tab's list selection. Every key goes through [`Nav::on_key`]; the UI
//! then moves GTK focus to [`Nav::focus`]. Keeping this in one place means
//! there is always a defined focus target, whatever the tab contents are.

use std::collections::HashMap;

use super::list::{Action, Key, ListState, Tab, TabTraits};

/// Where keyboard focus is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// The current tab's list (or its empty placeholder).
    List,
    Search,
    Pin,
}

/// What the UI must do after a key press, besides moving focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Show `tab` (stack page, tab buttons, search text, footer).
    ShowTab(Tab),
    /// Highlight this row of the current tab.
    SelectRow(usize),
    /// Activate this row of the current tab.
    Activate(usize),
    ToggleStar(usize),
    OpenJumpList(usize),
    ClearQuery,
    TogglePin,
}

fn traits(tab: Tab) -> TabTraits {
    TabTraits {
        can_star: tab == Tab::Clipboard,
        right_opens_jump_list: tab == Tab::Launcher,
    }
}

/// Focus and selection state of the main window.
#[derive(Debug)]
pub struct Nav {
    tab: Tab,
    focus: Focus,
    lists: HashMap<Tab, ListState>,
}

impl Default for Nav {
    fn default() -> Self {
        Self {
            tab: Tab::Clipboard,
            focus: Focus::List,
            lists: Tab::ALL.iter().map(|t| (*t, ListState::default())).collect(),
        }
    }
}

impl Nav {
    /// Starts on the Clipboard list.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tab(&self) -> Tab {
        self.tab
    }

    pub fn focus(&self) -> Focus {
        self.focus
    }

    /// Selected row of the current tab.
    pub fn selected(&self) -> usize {
        self.lists.get(&self.tab).map_or(0, ListState::selected)
    }

    /// Visible row count of `tab` changed (resets its selection).
    pub fn set_len(&mut self, tab: Tab, len: usize) {
        if let Some(list) = self.lists.get_mut(&tab) {
            list.set_len(len);
        }
    }

    /// Restores a selection after re-rendering `tab`.
    pub fn set_selected(&mut self, tab: Tab, index: usize) {
        if let Some(list) = self.lists.get_mut(&tab) {
            list.set_selected(index);
        }
    }

    /// A tab button was clicked.
    pub fn select_tab(&mut self, tab: Tab) -> Option<Effect> {
        self.focus = Focus::List;
        if tab == self.tab {
            return None;
        }
        self.tab = tab;
        Some(Effect::ShowTab(tab))
    }

    /// GTK focus moved by mouse (e.g. a click into the search entry).
    pub fn focus_moved(&mut self, focus: Focus) {
        self.focus = focus;
    }

    /// `Ctrl+F`.
    pub fn focus_search(&mut self) {
        self.focus = Focus::Search;
    }

    /// Handles a key; the caller then applies the effect and syncs focus.
    pub fn on_key(&mut self, key: Key) -> Option<Effect> {
        let tab = self.tab;
        let list = self.lists.get_mut(&tab)?;
        let action = match self.focus {
            Focus::Pin => {
                return match key {
                    Key::Left | Key::Down | Key::Escape => {
                        self.focus = Focus::List;
                        None
                    },
                    Key::Enter => Some(Effect::TogglePin),
                    _ => None,
                };
            },
            Focus::Search => list.on_search_key(key)?,
            Focus::List => list.on_list_key(key, traits(tab))?,
        };
        match action {
            Action::Select(i) => Some(Effect::SelectRow(i)),
            Action::Activate(i) => Some(Effect::Activate(i)),
            Action::ToggleStar(i) => Some(Effect::ToggleStar(i)),
            Action::OpenJumpList(i) => Some(Effect::OpenJumpList(i)),
            Action::FocusSearch => {
                self.focus = Focus::Search;
                None
            },
            Action::FocusList => {
                self.focus = Focus::List;
                None
            },
            Action::ClearQuery => {
                self.focus = Focus::List;
                Some(Effect::ClearQuery)
            },
            Action::PrevTab => match tab.prev() {
                Some(t) => self.select_tab(t),
                None => {
                    self.focus = Focus::List;
                    None
                },
            },
            Action::NextTab => match tab.next() {
                Some(t) => self.select_tab(t),
                None => {
                    self.focus = Focus::Pin;
                    None
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Effect::*;

    fn nav_with(lens: &[(Tab, usize)]) -> Nav {
        let mut n = Nav::new();
        for (t, len) in lens {
            n.set_len(*t, *len);
        }
        n
    }

    #[test]
    fn right_walks_every_tab_then_the_pin_and_left_walks_back() {
        let mut n = nav_with(&[(Tab::Clipboard, 3), (Tab::Snippets, 0), (Tab::Launcher, 0)]);
        assert_eq!(n.on_key(Key::Right), Some(ShowTab(Tab::Snippets)));
        assert_eq!(n.on_key(Key::Right), Some(ShowTab(Tab::Launcher)));
        assert_eq!((n.tab(), n.focus()), (Tab::Launcher, Focus::List));
        assert_eq!(n.on_key(Key::Right), None);
        assert_eq!(n.focus(), Focus::Pin);
        assert_eq!(n.on_key(Key::Right), None);
        assert_eq!(n.focus(), Focus::Pin);

        assert_eq!(n.on_key(Key::Left), None);
        assert_eq!((n.tab(), n.focus()), (Tab::Launcher, Focus::List));
        assert_eq!(n.on_key(Key::Left), Some(ShowTab(Tab::Snippets)));
        assert_eq!(n.on_key(Key::Left), Some(ShowTab(Tab::Clipboard)));
        assert_eq!(n.on_key(Key::Left), None);
        assert_eq!((n.tab(), n.focus()), (Tab::Clipboard, Focus::List));
    }

    #[test]
    fn empty_tabs_still_handle_keys() {
        let mut n = nav_with(&[(Tab::Snippets, 0)]);
        n.select_tab(Tab::Snippets);
        assert_eq!(n.on_key(Key::Up), None);
        assert_eq!(n.focus(), Focus::Search);
        assert_eq!(n.on_key(Key::Down), None);
        assert_eq!(n.focus(), Focus::List);
        assert_eq!(n.on_key(Key::Enter), None);
    }

    #[test]
    fn enter_on_pin_toggles_it() {
        let mut n = nav_with(&[(Tab::Launcher, 0)]);
        n.select_tab(Tab::Launcher);
        n.on_key(Key::Right);
        assert_eq!(n.on_key(Key::Enter), Some(TogglePin));
        assert_eq!(n.focus(), Focus::Pin);
    }

    #[test]
    fn list_keys_select_activate_and_star() {
        let mut n = nav_with(&[(Tab::Clipboard, 3)]);
        assert_eq!(n.on_key(Key::Down), Some(SelectRow(1)));
        assert_eq!(n.on_key(Key::Star), Some(ToggleStar(1)));
        assert_eq!(n.on_key(Key::Enter), Some(Activate(1)));
        assert_eq!(n.on_key(Key::Digit(3)), Some(Activate(2)));
    }

    #[test]
    fn search_keys_and_tab_switch_from_search() {
        let mut n = nav_with(&[(Tab::Clipboard, 2), (Tab::Snippets, 1)]);
        n.focus_search();
        assert_eq!(n.focus(), Focus::Search);
        assert_eq!(n.on_key(Key::Enter), Some(Activate(0)));
        assert_eq!(n.on_key(Key::Escape), Some(ClearQuery));
        assert_eq!(n.focus(), Focus::List);

        n.focus_search();
        assert_eq!(n.on_key(Key::Right), Some(ShowTab(Tab::Snippets)));
        assert_eq!(n.focus(), Focus::List);
    }

    #[test]
    fn launcher_right_opens_jump_list_when_it_has_items() {
        let mut n = nav_with(&[(Tab::Launcher, 2)]);
        n.select_tab(Tab::Launcher);
        assert_eq!(n.on_key(Key::Right), Some(OpenJumpList(0)));
        assert_eq!(n.focus(), Focus::List);
    }

    #[test]
    fn selection_is_kept_per_tab() {
        let mut n = nav_with(&[(Tab::Clipboard, 3), (Tab::Snippets, 3)]);
        n.on_key(Key::Down);
        n.on_key(Key::Right);
        assert_eq!(n.selected(), 0);
        n.on_key(Key::Left);
        assert_eq!(n.selected(), 1);
    }

    #[test]
    fn mouse_focus_changes_are_followed() {
        let mut n = Nav::new();
        n.focus_moved(Focus::Search);
        assert_eq!(n.focus(), Focus::Search);
        n.focus_moved(Focus::Pin);
        assert_eq!(n.on_key(Key::Left), None);
        assert_eq!(n.focus(), Focus::List);
    }
}
