//! Context menu entries of list rows

use super::list::Tab;

/// One context menu entry; the UI maps it to a label and an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    AddToSnippets,
    DeleteFromHistory,
    Edit,
    Delete,
    AddNew,
    MoveUp,
    MoveDown,
}

/// Sections (separated in the menu) for row `index` of `len` visible rows
/// of `tab`. Empty when the tab has no menu.
pub fn row_menu(tab: Tab, index: usize, len: usize) -> Vec<Vec<MenuItem>> {
    use MenuItem::*;
    let has_row = index < len;
    match tab {
        Tab::Clipboard if has_row => vec![vec![AddToSnippets], vec![DeleteFromHistory]],
        Tab::Snippets if has_row => {
            let mut moves = Vec::new();
            if index > 0 {
                moves.push(MoveUp);
            }
            if index + 1 < len {
                moves.push(MoveDown);
            }
            let mut sections = vec![vec![Edit, AddNew]];
            if !moves.is_empty() {
                sections.push(moves);
            }
            sections.push(vec![Delete]);
            sections
        },
        Tab::Snippets => vec![vec![AddNew]],
        Tab::Clipboard | Tab::Launcher => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MenuItem::*;

    #[test]
    fn clipboard_rows() {
        assert_eq!(row_menu(Tab::Clipboard, 0, 2), [vec![AddToSnippets], vec![DeleteFromHistory]]);
        assert!(row_menu(Tab::Clipboard, 0, 0).is_empty());
    }

    #[test]
    fn snippet_moves_depend_on_position() {
        assert_eq!(
            row_menu(Tab::Snippets, 0, 3),
            [vec![Edit, AddNew], vec![MoveDown], vec![Delete]]
        );
        assert_eq!(
            row_menu(Tab::Snippets, 1, 3),
            [vec![Edit, AddNew], vec![MoveUp, MoveDown], vec![Delete]]
        );
        assert_eq!(row_menu(Tab::Snippets, 2, 3), [vec![Edit, AddNew], vec![MoveUp], vec![Delete]]);
        assert_eq!(row_menu(Tab::Snippets, 0, 1), [vec![Edit, AddNew], vec![Delete]]);
    }

    #[test]
    fn empty_snippets_only_add_and_launcher_has_none() {
        assert_eq!(row_menu(Tab::Snippets, 0, 0), [vec![AddNew]]);
        assert!(row_menu(Tab::Launcher, 0, 3).is_empty());
    }
}
