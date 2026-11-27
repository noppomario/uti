//! Keyboard focus model of the main window
//!
//! Owns the current tab, where keyboard focus is (list / search / pin) and
//! each tab's list selection. Every key goes through [`Nav::on_key`]; the UI
//! then moves GTK focus to [`Nav::focus`]. Keeping this in one place means
//! there is always a defined focus target, whatever the tab contents are.

use std::collections::HashMap;

use super::list::{Action, Key, ListState, Tab, TabTraits};
use super::tree::{Row, Tree, TreeAction};

/// Where keyboard focus is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// The current tab's list (or its empty placeholder). On the Launcher
    /// tab this includes its open levels, which never take GTK focus.
    List,
    Search,
    Pin,
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
    ClearQuery,
    TogglePin,
    /// Build the children of a Launcher row and push them ([`Nav::tree_mut`]).
    TreeOpen {
        depth: usize,
        index: usize,
        enter: bool,
    },
    /// Activate a Launcher row.
    TreeActivate {
        depth: usize,
        index: usize,
    },
    /// Open the context menu of a Launcher row.
    TreeMenu {
        depth: usize,
        index: usize,
    },
    TreeEdit {
        depth: usize,
        index: usize,
    },
    TreeDelete {
        depth: usize,
        index: usize,
    },
    /// The Launcher levels or their selections changed.
    TreeChanged,
}

/// Focus and selection state of the main window.
#[derive(Debug)]
pub struct Nav {
    tab: Tab,
    focus: Focus,
    lists: HashMap<Tab, ListState>,
    /// Launcher levels; the root is the Launcher tab's list.
    tree: Tree,
}

impl Default for Nav {
    fn default() -> Self {
        Self {
            tab: Tab::Clipboard,
            focus: Focus::List,
            lists: Tab::ALL.iter().map(|t| (*t, ListState::default())).collect(),
            tree: Tree::new(),
        }
    }
}

impl From<TreeAction> for Effect {
    fn from(action: TreeAction) -> Self {
        match action {
            TreeAction::Open {
                depth,
                index,
                enter,
            } => Effect::TreeOpen {
                depth,
                index,
                enter,
            },
            TreeAction::Activate { depth, index } => Effect::TreeActivate { depth, index },
            TreeAction::Menu { depth, index } => Effect::TreeMenu { depth, index },
            TreeAction::Edit { depth, index } => Effect::TreeEdit { depth, index },
            TreeAction::Delete { depth, index } => Effect::TreeDelete { depth, index },
            TreeAction::Unhandled => Effect::TreeChanged,
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

    /// Selected row of the current tab (the Launcher root on the Launcher).
    pub fn selected(&self) -> usize {
        if self.tab == Tab::Launcher {
            return self.tree.selected(0).unwrap_or(0);
        }
        self.lists.get(&self.tab).map_or(0, ListState::selected)
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    /// For pointer input and pushing opened levels.
    pub fn tree_mut(&mut self) -> &mut Tree {
        &mut self.tree
    }

    /// Visible row count of `tab` changed (resets its selection).
    pub fn set_len(&mut self, tab: Tab, len: usize) {
        if let Some(list) = self.lists.get_mut(&tab) {
            list.set_len(len);
        }
    }

    /// New Launcher root rows: closes its levels.
    pub fn set_launcher_rows(&mut self, rows: Vec<Row>) {
        self.set_len(Tab::Launcher, rows.len());
        self.tree.set_root(rows);
    }

    /// Restores a selection after re-rendering `tab`.
    pub fn set_selected(&mut self, tab: Tab, index: usize) {
        if tab == Tab::Launcher {
            self.tree.select_root(index);
        } else if let Some(list) = self.lists.get_mut(&tab) {
            list.set_selected(index);
        }
    }

    /// The menu asked for by [`Effect::OpenMenu`] is shown. Pointer moves
    /// over the Launcher levels no longer change them (ADR-036).
    pub fn menu_opened(&mut self) {
        self.focus = Focus::Menu;
        self.tree.cancel_pending();
    }

    /// The Launcher tree for pointer input; `None` while a menu is open.
    pub fn pointer_tree(&mut self) -> Option<&mut Tree> {
        (self.focus != Focus::Menu).then_some(&mut self.tree)
    }

    /// The context menu closed; focus returns to the list.
    pub fn menu_closed(&mut self) {
        if self.focus == Focus::Menu {
            self.focus = Focus::List;
        }
    }

    fn traits(tab: Tab) -> TabTraits {
        TabTraits {
            can_add_snippet: tab == Tab::Clipboard,
            can_delete: matches!(tab, Tab::Clipboard | Tab::Snippets),
            can_edit: tab == Tab::Snippets,
            has_menu: matches!(tab, Tab::Clipboard | Tab::Snippets),
        }
    }

    /// A tab button was clicked. Leaving the Launcher closes its levels.
    pub fn select_tab(&mut self, tab: Tab) -> Option<Effect> {
        self.focus = Focus::List;
        if tab == self.tab {
            return None;
        }
        self.tree.close_all();
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
        let action = match self.focus {
            Focus::Menu => return None,
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
            Focus::List if tab == Tab::Launcher => match self.tree.on_key(key) {
                Some(TreeAction::Unhandled) => match key {
                    Key::Up => Action::FocusSearch,
                    Key::Left => Action::PrevTab,
                    Key::Right => Action::NextTab,
                    _ => return None,
                },
                Some(action) => return Some(action.into()),
                None => return Some(Effect::TreeChanged),
            },
            Focus::Search => self.lists.get_mut(&tab)?.on_search_key(key)?,
            Focus::List => self.lists.get_mut(&tab)?.on_list_key(key, Self::traits(tab))?,
        };
        match action {
            Action::Activate(index) if tab == Tab::Launcher => {
                Some(Effect::TreeActivate { depth: 0, index })
            },
            Action::Select(i) => Some(Effect::SelectRow(i)),
            Action::Activate(i) => Some(Effect::Activate(i)),
            Action::AddSnippet(i) => Some(Effect::AddSnippet(i)),
            Action::OpenMenu(i) => Some(Effect::OpenMenu(i)),
            Action::Delete(i) => Some(Effect::Delete(i)),
            Action::Edit(i) => Some(Effect::Edit(i)),
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
        n.set_launcher_rows(vec![Row::LEAF, Row::LEAF]);
        assert_eq!(n.on_key(Key::AddSnippet), None);
        assert_eq!(n.on_key(Key::Delete), Some(TreeDelete { depth: 0, index: 0 }));
        assert_eq!(n.on_key(Key::Rename), Some(TreeEdit { depth: 0, index: 0 }));
        assert_eq!(n.on_key(Key::Menu), Some(TreeMenu { depth: 0, index: 0 }));
        n.menu_opened();
        assert!(n.pointer_tree().is_none(), "the menu freezes the levels");
        n.menu_closed();
        assert!(n.pointer_tree().is_some());
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

    fn launcher(rows: &[Row]) -> Nav {
        let mut n = Nav::new();
        n.set_launcher_rows(rows.to_vec());
        n.select_tab(Tab::Launcher);
        n
    }

    #[test]
    fn launcher_keys_drive_the_tree() {
        let mut n = launcher(&[Row::FOLDER, Row::LEAF]);
        assert_eq!(
            n.on_key(Key::Right),
            Some(TreeOpen {
                depth: 0,
                index: 0,
                enter: true
            })
        );
        n.tree_mut().push(0, 0, vec![Row::LEAF, Row::LEAF], true);
        assert_eq!(n.on_key(Key::Down), Some(TreeChanged));
        assert_eq!(n.tree().selected(1), Some(1));
        assert_eq!(n.on_key(Key::Enter), Some(TreeActivate { depth: 1, index: 1 }));
        assert_eq!(n.on_key(Key::Left), Some(TreeChanged));
        assert_eq!(n.tree().depth(), 0);
        assert_eq!(n.on_key(Key::Left), Some(ShowTab(Tab::Snippets)));
    }

    #[test]
    fn launcher_root_falls_back_to_tabs_pin_and_search() {
        let mut n = launcher(&[Row::LEAF]);
        assert_eq!(n.on_key(Key::Right), None);
        assert_eq!(n.focus(), Focus::Pin);
        n.on_key(Key::Left);
        assert_eq!(n.on_key(Key::Up), None);
        assert_eq!(n.focus(), Focus::Search);
        assert_eq!(n.on_key(Key::Enter), Some(TreeActivate { depth: 0, index: 0 }));
    }

    #[test]
    fn leaving_the_launcher_closes_its_levels() {
        let mut n = launcher(&[Row::FOLDER]);
        n.on_key(Key::Right);
        n.tree_mut().push(0, 0, vec![Row::LEAF], true);
        n.select_tab(Tab::Clipboard);
        assert_eq!(n.tree().depth(), 0);
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
