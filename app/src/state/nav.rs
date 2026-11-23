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
    /// The Launcher jump list (recent files) next to the list.
    JumpList,
    /// A row's context menu is open; it handles its own keys.
    Menu,
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
    AddSnippet(usize),
    /// Open the context menu of this row (0 for an empty list).
    OpenMenu(usize),
    Delete(usize),
    Edit(usize),
    OpenJumpList(usize),
    ClearQuery,
    TogglePin,
    JumpSelect(usize),
    JumpActivate(usize),
    CloseJumpList,
}

/// Focus and selection state of the main window.
#[derive(Debug)]
pub struct Nav {
    tab: Tab,
    focus: Focus,
    lists: HashMap<Tab, ListState>,
    /// Per tab: which visible rows have a jump list.
    jumpable: HashMap<Tab, Vec<bool>>,
    jump: ListState,
}

impl Default for Nav {
    fn default() -> Self {
        Self {
            tab: Tab::Clipboard,
            focus: Focus::List,
            lists: Tab::ALL.iter().map(|t| (*t, ListState::default())).collect(),
            jumpable: HashMap::new(),
            jump: ListState::default(),
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

    /// Selected row of the open jump list.
    pub fn jump_selected(&self) -> usize {
        self.jump.selected()
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

    /// Which visible rows of `tab` have a jump list.
    pub fn set_jumpable(&mut self, tab: Tab, rows: Vec<bool>) {
        self.jumpable.insert(tab, rows);
    }

    /// The jump list asked for by [`Effect::OpenJumpList`] is shown.
    pub fn jump_list_opened(&mut self, len: usize) {
        self.jump.set_len(len);
        self.focus = Focus::JumpList;
    }

    /// The menu asked for by [`Effect::OpenMenu`] is shown.
    pub fn menu_opened(&mut self) {
        self.focus = Focus::Menu;
    }

    /// The context menu closed; focus returns to the list.
    pub fn menu_closed(&mut self) {
        if self.focus == Focus::Menu {
            self.focus = Focus::List;
        }
    }

    fn traits(&self, tab: Tab) -> TabTraits {
        let selected = self.lists.get(&tab).map_or(0, ListState::selected);
        TabTraits {
            can_add_snippet: tab == Tab::Clipboard,
            can_delete: matches!(tab, Tab::Clipboard | Tab::Snippets),
            can_edit: tab == Tab::Snippets,
            has_menu: matches!(tab, Tab::Clipboard | Tab::Snippets),
            right_opens_jump_list: self
                .jumpable
                .get(&tab)
                .and_then(|rows| rows.get(selected))
                .copied()
                .unwrap_or(false),
        }
    }

    fn on_jump_key(&mut self, key: Key) -> Option<Effect> {
        match key {
            Key::Left | Key::Escape => {
                self.focus = Focus::List;
                Some(Effect::CloseJumpList)
            },
            Key::Up | Key::Down => match self.jump.on_list_key(key, TabTraits::default())? {
                Action::Select(i) => Some(Effect::JumpSelect(i)),
                _ => None, // Up at the top stays in the jump list.
            },
            Key::Enter | Key::Digit(_) => match self.jump.on_list_key(key, TabTraits::default())? {
                Action::Activate(i) => Some(Effect::JumpActivate(i)),
                _ => None,
            },
            Key::Right | Key::AddSnippet | Key::Menu | Key::Delete | Key::Rename => None,
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
    /// Ignored while a menu is open: the window reports no moves then, so
    /// any that arrive are from before it opened.
    pub fn focus_moved(&mut self, focus: Focus) {
        if self.focus != Focus::Menu {
            self.focus = focus;
        }
    }

    /// `Ctrl+F`.
    pub fn focus_search(&mut self) {
        self.focus = Focus::Search;
    }

    /// Handles a key; the caller then applies the effect and syncs focus.
    pub fn on_key(&mut self, key: Key) -> Option<Effect> {
        let tab = self.tab;
        match self.focus {
            Focus::JumpList => return self.on_jump_key(key),
            Focus::Menu => return None,
            _ => {},
        }
        let traits = self.traits(tab);
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
            Focus::List => list.on_list_key(key, traits)?,
            Focus::JumpList | Focus::Menu => unreachable!("handled above"),
        };
        match action {
            Action::Select(i) => Some(Effect::SelectRow(i)),
            Action::Activate(i) => Some(Effect::Activate(i)),
            Action::AddSnippet(i) => Some(Effect::AddSnippet(i)),
            Action::OpenMenu(i) => Some(Effect::OpenMenu(i)),
            Action::Delete(i) => Some(Effect::Delete(i)),
            Action::Edit(i) => Some(Effect::Edit(i)),
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
    fn list_keys_select_activate_and_add_snippet() {
        let mut n = nav_with(&[(Tab::Clipboard, 3)]);
        assert_eq!(n.on_key(Key::Down), Some(SelectRow(1)));
        assert_eq!(n.on_key(Key::AddSnippet), Some(AddSnippet(1)));
        assert_eq!(n.on_key(Key::Enter), Some(Activate(1)));
        assert_eq!(n.on_key(Key::Digit(3)), Some(Activate(2)));
    }

    #[test]
    fn item_keys_per_tab() {
        let mut n = nav_with(&[(Tab::Clipboard, 2), (Tab::Snippets, 2), (Tab::Launcher, 2)]);
        assert_eq!(n.on_key(Key::Delete), Some(Delete(0)));
        assert_eq!(n.on_key(Key::Rename), None);
        assert_eq!(n.on_key(Key::Menu), Some(OpenMenu(0)));

        n.select_tab(Tab::Snippets);
        assert_eq!(n.on_key(Key::AddSnippet), None);
        assert_eq!(n.on_key(Key::Delete), Some(Delete(0)));
        assert_eq!(n.on_key(Key::Rename), Some(Edit(0)));
        assert_eq!(n.on_key(Key::Menu), Some(OpenMenu(0)));

        n.select_tab(Tab::Launcher);
        for key in [Key::AddSnippet, Key::Delete, Key::Rename, Key::Menu] {
            assert_eq!(n.on_key(key), None);
        }
    }

    #[test]
    fn open_menu_swallows_keys_until_closed() {
        let mut n = nav_with(&[(Tab::Clipboard, 3)]);
        n.on_key(Key::Menu);
        n.menu_opened();
        assert_eq!(n.focus(), Focus::Menu);
        assert_eq!(n.on_key(Key::Down), None);
        assert_eq!(n.on_key(Key::Enter), None);
        assert_eq!(n.selected(), 0);
        n.menu_closed();
        assert_eq!(n.focus(), Focus::List);
        assert_eq!(n.on_key(Key::Down), Some(SelectRow(1)));
    }

    #[test]
    fn focus_moves_reported_while_the_menu_is_open_are_stale() {
        // Selecting the clicked row before opening the menu reports a focus
        // move that arrives after the menu opened.
        let mut n = Nav::new();
        n.menu_opened();
        n.focus_moved(Focus::List);
        assert_eq!(n.focus(), Focus::Menu);
        n.menu_closed();
        assert_eq!(n.focus(), Focus::List);
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
    fn launcher_right_opens_jump_list_only_for_items_with_history() {
        let mut n = nav_with(&[(Tab::Launcher, 2)]);
        n.set_jumpable(Tab::Launcher, vec![true, false]);
        n.select_tab(Tab::Launcher);
        assert_eq!(n.on_key(Key::Right), Some(OpenJumpList(0)));
        assert_eq!(n.focus(), Focus::List);

        n.on_key(Key::Down);
        assert_eq!(n.on_key(Key::Right), None);
        assert_eq!(n.focus(), Focus::Pin);
    }

    #[test]
    fn jump_list_keys() {
        let mut n = nav_with(&[(Tab::Launcher, 1)]);
        n.set_jumpable(Tab::Launcher, vec![true]);
        n.select_tab(Tab::Launcher);
        n.on_key(Key::Right);
        n.jump_list_opened(3);
        assert_eq!(n.focus(), Focus::JumpList);
        assert_eq!(n.on_key(Key::Down), Some(JumpSelect(1)));
        assert_eq!(n.on_key(Key::Up), Some(JumpSelect(0)));
        assert_eq!(n.on_key(Key::Up), None);
        assert_eq!(n.focus(), Focus::JumpList);
        assert_eq!(n.on_key(Key::Digit(3)), Some(JumpActivate(2)));
        assert_eq!(n.on_key(Key::Enter), Some(JumpActivate(0)));
        assert_eq!(n.on_key(Key::Left), Some(CloseJumpList));
        assert_eq!((n.tab(), n.focus()), (Tab::Launcher, Focus::List));
    }

    #[test]
    fn escape_closes_jump_list_and_tab_change_resets_it() {
        let mut n = nav_with(&[(Tab::Launcher, 1)]);
        n.set_jumpable(Tab::Launcher, vec![true]);
        n.select_tab(Tab::Launcher);
        n.on_key(Key::Right);
        n.jump_list_opened(0);
        assert_eq!(n.on_key(Key::Escape), Some(CloseJumpList));
        assert_eq!(n.focus(), Focus::List);

        n.on_key(Key::Right);
        n.jump_list_opened(2);
        n.select_tab(Tab::Clipboard);
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
