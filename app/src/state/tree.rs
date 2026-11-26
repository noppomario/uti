//! Cascading tree navigation (no GTK types)
//!
//! A stack of open levels, each a list of rows with its own selection:
//! level 0 is the tab's own list, deeper levels are the open children of a
//! row in the level above. Keys, hover and clicks change the stack; the UI
//! builds a level's rows when asked to ([`TreeAction::Open`]) and mirrors
//! the stack in its widgets. Rows only say whether they can be selected,
//! opened and activated, so any tree-shaped content can use it.

use std::time::Duration;

use super::list::Key;

/// How long the pointer rests on a row before its children open, or before
/// the open child of another row closes. Moving into the open child within
/// this time keeps it open.
pub const HOVER_DELAY: Duration = Duration::from_millis(300);

/// What a row can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    pub selectable: bool,
    /// Has children that open beside it.
    pub branch: bool,
    /// Enter / click activates it (otherwise they open its children).
    pub activates: bool,
}

impl Row {
    pub const SEPARATOR: Row = Row {
        selectable: false,
        branch: false,
        activates: false,
    };
    pub const LEAF: Row = Row {
        selectable: true,
        branch: false,
        activates: true,
    };
    pub const FOLDER: Row = Row {
        selectable: true,
        branch: true,
        activates: false,
    };
    /// Activates, and also has children (a command with recent files).
    pub const BRANCH_LEAF: Row = Row {
        selectable: true,
        branch: true,
        activates: true,
    };
}

/// What the UI must do besides mirroring the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeAction {
    /// Build the children of row `index` at `depth` and pass them to
    /// [`Tree::push`] with `enter`.
    Open {
        depth: usize,
        index: usize,
        enter: bool,
    },
    /// Activate row `index` at `depth`.
    Activate { depth: usize, index: usize },
    /// A key the root level does not use (tab switching, search focus).
    Unhandled,
}

#[derive(Debug)]
struct Level {
    rows: Vec<Row>,
    selected: Option<usize>,
    /// Row of the level above this level belongs to.
    owner: usize,
}

impl Level {
    fn new(rows: Vec<Row>, owner: usize) -> Self {
        Self {
            rows,
            selected: None,
            owner,
        }
    }

    fn first(&self) -> Option<usize> {
        self.rows.iter().position(|r| r.selectable)
    }
}

#[derive(Debug, Clone, Copy)]
struct Pending {
    depth: usize,
    index: usize,
    token: u64,
}

/// Open levels, their selections and the pending hover action.
#[derive(Debug)]
pub struct Tree {
    levels: Vec<Level>,
    /// Level keys act on: the one last pointed at or entered.
    active: usize,
    pending: Option<Pending>,
    next_token: u64,
}

impl Default for Tree {
    fn default() -> Self {
        Self {
            levels: vec![Level::new(Vec::new(), 0)],
            active: 0,
            pending: None,
            next_token: 0,
        }
    }
}

impl Tree {
    /// An empty root level.
    pub fn new() -> Self {
        Self::default()
    }

    /// New root rows: closes every child and selects the first row.
    pub fn set_root(&mut self, rows: Vec<Row>) {
        let mut root = Level::new(rows, 0);
        root.selected = root.first();
        self.levels = vec![root];
        self.active = 0;
        self.pending = None;
    }

    /// Number of open child levels (0: only the root).
    pub fn depth(&self) -> usize {
        self.levels.len() - 1
    }

    pub fn active(&self) -> usize {
        self.active
    }

    pub fn selected(&self, depth: usize) -> Option<usize> {
        self.levels.get(depth)?.selected
    }

    /// Row of level `depth - 1` that open level `depth` belongs to.
    pub fn owner(&self, depth: usize) -> Option<usize> {
        (depth > 0).then(|| self.levels.get(depth).map(|l| l.owner)).flatten()
    }

    /// Selects a root row (e.g. restored after re-rendering).
    pub fn select_root(&mut self, index: usize) {
        if self.levels[0].rows.get(index).is_some_and(|r| r.selectable) {
            self.levels[0].selected = Some(index);
        }
    }

    /// Opens `rows` as the children of row `index` at `depth`, replacing
    /// deeper levels; `enter` makes it the active level with its first row
    /// selected.
    pub fn push(&mut self, depth: usize, index: usize, rows: Vec<Row>, enter: bool) {
        if depth >= self.levels.len() {
            return;
        }
        self.close_from(depth + 1);
        let mut level = Level::new(rows, index);
        if enter {
            level.selected = level.first();
            self.active = depth + 1;
        }
        self.levels.push(level);
    }

    /// Closes every child level.
    pub fn close_all(&mut self) {
        self.close_from(1);
        self.pending = None;
    }

    fn close_from(&mut self, depth: usize) {
        self.levels.truncate(depth.max(1));
        self.active = self.active.min(self.levels.len() - 1);
    }

    fn child_open(&self, depth: usize, index: usize) -> bool {
        self.levels.get(depth + 1).is_some_and(|l| l.owner == index)
    }

    fn row(&self, depth: usize, index: usize) -> Option<Row> {
        self.levels.get(depth)?.rows.get(index).copied()
    }

    /// Moves the selection of level `depth` to the next selectable row in
    /// `forward` direction; `false` at the end.
    fn step(&mut self, depth: usize, forward: bool) -> bool {
        let level = &mut self.levels[depth];
        let len = level.rows.len();
        let mut i = level.selected;
        loop {
            let next = match (i, forward) {
                (None, true) => Some(0),
                (None, false) => len.checked_sub(1),
                (Some(i), true) => (i + 1 < len).then_some(i + 1),
                (Some(i), false) => i.checked_sub(1),
            };
            let Some(next) = next else {
                return false;
            };
            if level.rows[next].selectable {
                level.selected = Some(next);
                return true;
            }
            i = Some(next);
        }
    }

    /// Enters the open child of (`depth`, `index`), or asks to open it.
    fn open_or_enter(&mut self, depth: usize, index: usize) -> Option<TreeAction> {
        self.pending = None;
        if self.child_open(depth, index) {
            self.close_from(depth + 2);
            let child = &mut self.levels[depth + 1];
            if child.selected.is_none() {
                child.selected = child.first();
            }
            self.active = depth + 1;
            return None;
        }
        Some(TreeAction::Open {
            depth,
            index,
            enter: true,
        })
    }

    /// Enter on a row: activate it, or open its children.
    fn choose(&mut self, depth: usize, index: usize) -> Option<TreeAction> {
        let row = self.row(depth, index)?;
        if row.activates {
            Some(TreeAction::Activate { depth, index })
        } else if row.branch {
            self.open_or_enter(depth, index)
        } else {
            None
        }
    }

    /// Handles a key at the active level.
    pub fn on_key(&mut self, key: Key) -> Option<TreeAction> {
        let k = self.active;
        let at_root = k == 0;
        let selected = self.levels[k].selected;
        match key {
            Key::Up | Key::Down => {
                self.pending = None;
                self.close_from(k + 1);
                let moved = self.step(k, key == Key::Down);
                (!moved && key == Key::Up && at_root).then_some(TreeAction::Unhandled)
            },
            Key::Right => match selected {
                Some(i) if self.row(k, i).is_some_and(|r| r.branch) => self.open_or_enter(k, i),
                _ => at_root.then_some(TreeAction::Unhandled),
            },
            Key::Left | Key::Escape if at_root => Some(TreeAction::Unhandled),
            Key::Left | Key::Escape => {
                self.pending = None;
                self.close_from(k);
                self.active = k - 1;
                None
            },
            Key::Enter => selected.and_then(|i| self.choose(k, i)),
            Key::Digit(n) => {
                let i = (0..self.levels[k].rows.len())
                    .filter(|&i| self.levels[k].rows[i].selectable)
                    .nth(usize::from(n).checked_sub(1)?)?;
                self.levels[k].selected = Some(i);
                self.close_from(k + 1);
                self.choose(k, i)
            },
            Key::AddSnippet | Key::Menu | Key::Delete | Key::Rename => {
                at_root.then_some(TreeAction::Unhandled)
            },
        }
    }

    /// The pointer moved onto row `index` at `depth`: selects it. Returns a
    /// token when the UI must call [`Tree::hover_elapsed`] after
    /// [`HOVER_DELAY`].
    pub fn hover(&mut self, depth: usize, index: usize) -> Option<u64> {
        let row = self.row(depth, index)?;
        if !row.selectable {
            return None;
        }
        // Reaching a deeper level keeps the child the pointer moved into.
        if self.pending.is_some_and(|p| p.depth < depth) {
            self.pending = None;
        }
        self.active = depth;
        self.levels[depth].selected = Some(index);
        if self.child_open(depth, index) {
            self.pending = None;
            return None;
        }
        if self.pending.is_some_and(|p| p.depth == depth && p.index == index) {
            return None;
        }
        if !row.branch && self.levels.len() <= depth + 1 {
            self.pending = None;
            return None;
        }
        self.next_token += 1;
        self.pending = Some(Pending {
            depth,
            index,
            token: self.next_token,
        });
        Some(self.next_token)
    }

    /// The hover delay of `token` passed: opens the hovered row's children
    /// or closes the other open child. Stale tokens do nothing.
    pub fn hover_elapsed(&mut self, token: u64) -> Option<TreeAction> {
        let p = self.pending.filter(|p| p.token == token)?;
        self.pending = None;
        let row = self.row(p.depth, p.index)?;
        if row.branch {
            Some(TreeAction::Open {
                depth: p.depth,
                index: p.index,
                enter: false,
            })
        } else {
            self.close_from(p.depth + 1);
            None
        }
    }

    /// The pointer left the rows of level `depth` (not for another row).
    pub fn leave(&mut self, depth: usize) {
        if self.pending.is_some_and(|p| p.depth == depth) {
            self.pending = None;
        }
    }

    /// Row `index` at `depth` was clicked.
    pub fn click(&mut self, depth: usize, index: usize) -> Option<TreeAction> {
        let row = self.row(depth, index)?;
        if !row.selectable {
            return None;
        }
        self.pending = None;
        self.active = depth;
        self.levels[depth].selected = Some(index);
        if row.activates {
            Some(TreeAction::Activate { depth, index })
        } else if row.branch && !self.child_open(depth, index) {
            Some(TreeAction::Open {
                depth,
                index,
                enter: false,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TreeAction::*;

    const S: Row = Row::SEPARATOR;
    const L: Row = Row::LEAF;
    const F: Row = Row::FOLDER;
    const B: Row = Row::BRANCH_LEAF;

    fn tree(root: &[Row]) -> Tree {
        let mut t = Tree::new();
        t.set_root(root.to_vec());
        t
    }

    /// Root [F, L], with the folder's children [L, F, S, L] entered.
    fn entered() -> Tree {
        let mut t = tree(&[F, L]);
        assert_eq!(
            t.on_key(Key::Right),
            Some(Open {
                depth: 0,
                index: 0,
                enter: true
            })
        );
        t.push(0, 0, vec![L, F, S, L], true);
        t
    }

    #[test]
    fn up_down_skip_separators_and_up_at_the_top_is_unhandled() {
        let mut t = tree(&[S, L, S, L]);
        assert_eq!(t.selected(0), Some(1));
        assert_eq!(t.on_key(Key::Down), None);
        assert_eq!(t.selected(0), Some(3));
        assert_eq!(t.on_key(Key::Down), None);
        assert_eq!(t.selected(0), Some(3));
        t.on_key(Key::Up);
        assert_eq!(t.on_key(Key::Up), Some(Unhandled));
        assert_eq!(t.selected(0), Some(1));
        assert_eq!(tree(&[]).on_key(Key::Up), Some(Unhandled));
    }

    #[test]
    fn right_opens_and_enters_a_branch() {
        let t = entered();
        assert_eq!((t.depth(), t.active(), t.selected(1), t.owner(1)), (1, 1, Some(0), Some(0)));
    }

    #[test]
    fn right_on_a_leaf_is_unhandled_only_at_the_root() {
        let mut t = tree(&[L]);
        assert_eq!(t.on_key(Key::Right), Some(Unhandled));
        let mut t = entered();
        assert_eq!(t.on_key(Key::Right), None);
        assert_eq!(t.depth(), 1);
    }

    #[test]
    fn left_and_escape_close_one_level() {
        let mut t = entered();
        t.on_key(Key::Down);
        assert_eq!(
            t.on_key(Key::Right),
            Some(Open {
                depth: 1,
                index: 1,
                enter: true
            })
        );
        t.push(1, 1, vec![L], true);
        assert_eq!(t.on_key(Key::Escape), None);
        assert_eq!((t.depth(), t.active(), t.selected(1)), (1, 1, Some(1)));
        assert_eq!(t.on_key(Key::Left), None);
        assert_eq!((t.depth(), t.active(), t.selected(0)), (0, 0, Some(0)));
        assert_eq!(t.on_key(Key::Left), Some(Unhandled));
        assert_eq!(t.on_key(Key::Escape), Some(Unhandled));
    }

    #[test]
    fn enter_activates_leaves_and_opens_folders() {
        let mut t = tree(&[F, B, L]);
        assert_eq!(
            t.on_key(Key::Enter),
            Some(Open {
                depth: 0,
                index: 0,
                enter: true
            })
        );
        t.on_key(Key::Down);
        assert_eq!(t.on_key(Key::Enter), Some(Activate { depth: 0, index: 1 }));
        assert_eq!(
            t.on_key(Key::Right),
            Some(Open {
                depth: 0,
                index: 1,
                enter: true
            })
        );
    }

    #[test]
    fn digits_pick_the_nth_selectable_row_of_the_active_level() {
        let mut t = entered();
        assert_eq!(t.on_key(Key::Digit(3)), Some(Activate { depth: 1, index: 3 }));
        assert_eq!(
            t.on_key(Key::Digit(2)),
            Some(Open {
                depth: 1,
                index: 1,
                enter: true
            })
        );
        assert_eq!(t.on_key(Key::Digit(9)), None);
    }

    #[test]
    fn right_on_an_open_branch_enters_without_reopening() {
        let mut t = tree(&[F]);
        assert_eq!(
            t.hover(0, 0).and_then(|tok| t.hover_elapsed(tok)),
            Some(Open {
                depth: 0,
                index: 0,
                enter: false
            })
        );
        t.push(0, 0, vec![S, L], false);
        assert_eq!((t.active(), t.selected(1)), (0, None));
        assert_eq!(t.on_key(Key::Right), None);
        assert_eq!((t.active(), t.selected(1)), (1, Some(1)));
    }

    #[test]
    fn moving_the_selection_by_key_closes_children() {
        let mut t = tree(&[F, L]);
        t.hover(0, 0).and_then(|tok| t.hover_elapsed(tok));
        t.push(0, 0, vec![L], false);
        t.on_key(Key::Down);
        assert_eq!(t.depth(), 0);
    }

    #[test]
    fn hover_selects_at_once_and_opens_after_the_delay() {
        let mut t = tree(&[L, F]);
        assert_eq!(t.hover(0, 0), None);
        let tok = t.hover(0, 1).unwrap();
        assert_eq!(t.selected(0), Some(1));
        assert_eq!(t.hover(0, 1), None, "same row: the timer keeps running");
        assert_eq!(
            t.hover_elapsed(tok),
            Some(Open {
                depth: 0,
                index: 1,
                enter: false
            })
        );
        assert_eq!(t.hover_elapsed(tok), None);
    }

    #[test]
    fn a_newer_hover_makes_older_tokens_stale() {
        let mut t = tree(&[F, F]);
        let old = t.hover(0, 0).unwrap();
        let new = t.hover(0, 1).unwrap();
        assert_eq!(t.hover_elapsed(old), None);
        assert_eq!(
            t.hover_elapsed(new),
            Some(Open {
                depth: 0,
                index: 1,
                enter: false
            })
        );
    }

    #[test]
    fn hovering_another_row_closes_the_open_child_after_the_delay() {
        let mut t = tree(&[F, L]);
        t.hover(0, 0).and_then(|tok| t.hover_elapsed(tok));
        t.push(0, 0, vec![L], false);
        let tok = t.hover(0, 1).unwrap();
        assert_eq!(t.depth(), 1, "kept until the delay passes");
        assert_eq!(t.hover_elapsed(tok), None);
        assert_eq!(t.depth(), 0);
    }

    #[test]
    fn reaching_the_open_child_in_time_keeps_it() {
        let mut t = tree(&[F, F]);
        t.hover(0, 0).and_then(|tok| t.hover_elapsed(tok));
        t.push(0, 0, vec![L], false);
        let tok = t.hover(0, 1).unwrap();
        assert_eq!(t.hover(1, 0), None);
        assert_eq!(t.hover_elapsed(tok), None);
        assert_eq!((t.depth(), t.owner(1), t.active()), (1, Some(0), 1));
    }

    #[test]
    fn returning_to_the_owner_row_keeps_its_child() {
        let mut t = tree(&[F, L]);
        t.hover(0, 0).and_then(|tok| t.hover_elapsed(tok));
        t.push(0, 0, vec![L], false);
        let tok = t.hover(0, 1).unwrap();
        assert_eq!(t.hover(0, 0), None);
        assert_eq!(t.hover_elapsed(tok), None);
        assert_eq!(t.depth(), 1);
    }

    #[test]
    fn leaving_a_level_cancels_its_pending_hover() {
        let mut t = tree(&[F]);
        let tok = t.hover(0, 0).unwrap();
        t.leave(1);
        t.leave(0);
        assert_eq!(t.hover_elapsed(tok), None);
    }

    #[test]
    fn separators_ignore_the_pointer() {
        let mut t = tree(&[L, S]);
        assert_eq!(t.hover(0, 1), None);
        assert_eq!(t.click(0, 1), None);
        assert_eq!(t.selected(0), Some(0));
    }

    #[test]
    fn clicks_open_folders_at_once_and_activate_leaves() {
        let mut t = tree(&[F, B]);
        assert_eq!(
            t.click(0, 0),
            Some(Open {
                depth: 0,
                index: 0,
                enter: false
            })
        );
        t.push(0, 0, vec![L], false);
        assert_eq!(t.click(0, 0), None, "already open");
        assert_eq!(t.click(1, 0), Some(Activate { depth: 1, index: 0 }));
        assert_eq!(t.click(0, 1), Some(Activate { depth: 0, index: 1 }));
    }

    #[test]
    fn new_root_rows_close_everything() {
        let mut t = entered();
        t.set_root(vec![S, L]);
        assert_eq!((t.depth(), t.active(), t.selected(0)), (0, 0, Some(1)));
    }
}
