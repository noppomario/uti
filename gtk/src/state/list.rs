//! Tabs, search filtering and keyboard navigation shared by the list tabs

/// Main window tabs, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tab {
    /// Retired: a workaround for Japanese input in CLI tools, no longer
    /// needed. Kept but not wired (absent from [`Tab::ALL`]).
    #[allow(dead_code)]
    Prompt,
    Clipboard,
    Snippets,
    Launcher,
}

impl Tab {
    /// Tabs shown, in display order.
    pub const ALL: [Tab; 3] = [Tab::Clipboard, Tab::Snippets, Tab::Launcher];

    /// Tab to the left; `None` at the first tab (no wrap-around).
    pub fn prev(self) -> Option<Tab> {
        let i = Self::ALL.iter().position(|t| *t == self)?;
        i.checked_sub(1).map(|i| Self::ALL[i])
    }

    /// Tab to the right; `None` at the last tab (no wrap-around).
    pub fn next(self) -> Option<Tab> {
        let i = Self::ALL.iter().position(|t| *t == self)?;
        Self::ALL.get(i + 1).copied()
    }
}

/// Indices of `texts` that contain `query`, case-insensitively.
pub fn filter<'a>(texts: impl IntoIterator<Item = &'a str>, query: &str) -> Vec<usize> {
    let query = query.to_lowercase();
    texts
        .into_iter()
        .enumerate()
        .filter(|(_, t)| t.to_lowercase().contains(&query))
        .map(|(i, _)| i)
        .collect()
}

/// Keys the list tabs react to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Escape,
    /// `1`-`9`
    Digit(u8),
    /// `S`: star a clipboard item.
    Star,
}

/// Result of a key press for the UI layer to carry out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Selection changed; scroll it into view.
    Select(usize),
    /// Activate the visible item at this position.
    Activate(usize),
    FocusSearch,
    FocusList,
    /// Clear the query and focus the list.
    ClearQuery,
    PrevTab,
    NextTab,
    /// Toggle the star on the visible item at this position.
    ToggleStar(usize),
    /// Open the jump list of the visible item at this position.
    OpenJumpList(usize),
}

/// Per-tab behaviour switches.
#[derive(Debug, Clone, Copy, Default)]
pub struct TabTraits {
    /// `S` stars items (Clipboard).
    pub can_star: bool,
    /// Right opens a jump list instead of moving to the next tab (Launcher).
    pub right_opens_jump_list: bool,
}

/// Selection over the currently visible (filtered) items of one tab.
#[derive(Debug, Default)]
pub struct ListState {
    len: usize,
    selected: usize,
}

impl ListState {
    /// Creates an empty list.
    #[cfg(test)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of visible items changed; resets the selection to the top.
    pub fn set_len(&mut self, len: usize) {
        self.len = len;
        self.selected = 0;
    }

    /// Currently selected position.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Restores a selection (e.g. after re-rendering), clamped to the list.
    pub fn set_selected(&mut self, index: usize) {
        self.selected = index.min(self.len.saturating_sub(1));
    }

    /// Key pressed while the list has focus.
    pub fn on_list_key(&mut self, key: Key, traits: TabTraits) -> Option<Action> {
        let has_items = self.len > 0;
        match key {
            Key::Up if self.selected == 0 => Some(Action::FocusSearch),
            Key::Up => {
                self.selected -= 1;
                Some(Action::Select(self.selected))
            },
            Key::Down if self.selected + 1 < self.len => {
                self.selected += 1;
                Some(Action::Select(self.selected))
            },
            Key::Down => None,
            Key::Enter => has_items.then_some(Action::Activate(self.selected)),
            Key::Digit(n) => {
                let i = usize::from(n).checked_sub(1)?;
                (i < self.len).then_some(Action::Activate(i))
            },
            Key::Left => Some(Action::PrevTab),
            Key::Right if traits.right_opens_jump_list && has_items => {
                Some(Action::OpenJumpList(self.selected))
            },
            Key::Right => Some(Action::NextTab),
            Key::Star if traits.can_star && has_items => Some(Action::ToggleStar(self.selected)),
            Key::Star | Key::Escape => None,
        }
    }

    /// Key pressed while the search entry has focus.
    pub fn on_search_key(&mut self, key: Key) -> Option<Action> {
        match key {
            Key::Down => Some(Action::FocusList),
            Key::Enter => (self.len > 0).then_some(Action::Activate(0)),
            Key::Escape => Some(Action::ClearQuery),
            Key::Left => Some(Action::PrevTab),
            Key::Right => Some(Action::NextTab),
            Key::Up | Key::Digit(_) | Key::Star => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Action::*;

    const PLAIN: TabTraits = TabTraits {
        can_star: false,
        right_opens_jump_list: false,
    };

    fn list(len: usize) -> ListState {
        let mut l = ListState::new();
        l.set_len(len);
        l
    }

    #[test]
    fn tabs_stop_at_both_ends() {
        assert_eq!(Tab::Clipboard.prev(), None);
        assert_eq!(Tab::Clipboard.next(), Some(Tab::Snippets));
        assert_eq!(Tab::Snippets.prev(), Some(Tab::Clipboard));
        assert_eq!(Tab::Launcher.next(), None);
    }

    #[test]
    fn retired_prompt_tab_is_not_reachable() {
        assert!(!Tab::ALL.contains(&Tab::Prompt));
        assert!(Tab::ALL.iter().all(|t| t.prev() != Some(Tab::Prompt)));
    }

    #[test]
    fn filter_is_case_insensitive_substring() {
        let texts = ["Hello World", "foo", "WORLD peace", "bar"];
        assert_eq!(filter(texts, "world"), vec![0, 2]);
        assert_eq!(filter(texts, ""), vec![0, 1, 2, 3]);
        assert_eq!(filter(texts, "zzz"), Vec::<usize>::new());
    }

    #[test]
    fn up_down_move_without_wrapping() {
        let mut l = list(3);
        assert_eq!(l.on_list_key(Key::Down, PLAIN), Some(Select(1)));
        assert_eq!(l.on_list_key(Key::Down, PLAIN), Some(Select(2)));
        assert_eq!(l.on_list_key(Key::Down, PLAIN), None);
        assert_eq!(l.on_list_key(Key::Up, PLAIN), Some(Select(1)));
    }

    #[test]
    fn up_on_first_item_or_empty_list_focuses_search() {
        assert_eq!(list(3).on_list_key(Key::Up, PLAIN), Some(FocusSearch));
        assert_eq!(list(0).on_list_key(Key::Up, PLAIN), Some(FocusSearch));
    }

    #[test]
    fn enter_and_digits_activate() {
        let mut l = list(3);
        l.on_list_key(Key::Down, PLAIN);
        assert_eq!(l.on_list_key(Key::Enter, PLAIN), Some(Activate(1)));
        assert_eq!(l.on_list_key(Key::Digit(3), PLAIN), Some(Activate(2)));
        assert_eq!(l.on_list_key(Key::Digit(4), PLAIN), None);
        assert_eq!(list(0).on_list_key(Key::Enter, PLAIN), None);
    }

    #[test]
    fn left_right_switch_tabs_or_open_jump_list() {
        let mut l = list(2);
        assert_eq!(l.on_list_key(Key::Left, PLAIN), Some(PrevTab));
        assert_eq!(l.on_list_key(Key::Right, PLAIN), Some(NextTab));
        let launcher = TabTraits {
            right_opens_jump_list: true,
            ..PLAIN
        };
        assert_eq!(l.on_list_key(Key::Right, launcher), Some(OpenJumpList(0)));
        // Nothing to open: Right moves on (to the pin button after the last tab).
        assert_eq!(list(0).on_list_key(Key::Right, launcher), Some(NextTab));
    }

    #[test]
    fn star_only_where_allowed() {
        let mut l = list(2);
        assert_eq!(l.on_list_key(Key::Star, PLAIN), None);
        let clipboard = TabTraits {
            can_star: true,
            ..PLAIN
        };
        assert_eq!(l.on_list_key(Key::Star, clipboard), Some(ToggleStar(0)));
    }

    #[test]
    fn new_items_reset_selection() {
        let mut l = list(5);
        l.on_list_key(Key::Down, PLAIN);
        l.set_len(4);
        assert_eq!(l.selected(), 0);
    }

    #[test]
    fn set_selected_is_clamped() {
        let mut l = list(3);
        l.set_selected(5);
        assert_eq!(l.selected(), 2);
        let mut empty = list(0);
        empty.set_selected(1);
        assert_eq!(empty.selected(), 0);
    }

    #[test]
    fn search_keys() {
        let mut l = list(2);
        assert_eq!(l.on_search_key(Key::Down), Some(FocusList));
        assert_eq!(l.on_search_key(Key::Enter), Some(Activate(0)));
        assert_eq!(l.on_search_key(Key::Escape), Some(ClearQuery));
        assert_eq!(l.on_search_key(Key::Left), Some(PrevTab));
        assert_eq!(l.on_search_key(Key::Right), Some(NextTab));
        assert_eq!(list(0).on_search_key(Key::Enter), None);
    }
}
