//! Edits of the launcher tree by node id (no GTK)

use super::{LauncherNode, NodeKind};

/// A position in the tree: before child `index` of folder `parent` (the
/// root for `None`); `index` past the end appends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub parent: Option<String>,
    pub index: usize,
}

/// Why a move was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum MoveError {
    NotFound,
    /// Into the node itself or one of its descendants.
    IntoItself,
}

/// The node with `id`.
pub fn find<'a>(nodes: &'a [LauncherNode], id: &str) -> Option<&'a LauncherNode> {
    let path = path_of(nodes, id)?;
    super::entry::node_at(nodes, &path)
}

/// Child indices from the root to the node with `id`.
pub fn path_of(nodes: &[LauncherNode], id: &str) -> Option<Vec<usize>> {
    for (i, node) in nodes.iter().enumerate() {
        if node.id == id {
            return Some(vec![i]);
        }
        if let NodeKind::Folder { children, .. } = &node.kind {
            if let Some(mut path) = path_of(children, id) {
                path.insert(0, i);
                return Some(path);
            }
        }
    }
    None
}

/// The place of the node with `id` (inserting there puts a node before it).
pub fn place_of(nodes: &[LauncherNode], id: &str) -> Option<Place> {
    let path = path_of(nodes, id)?;
    let (index, parent) = path.split_last()?;
    let parent = match parent {
        [] => None,
        p => Some(super::entry::node_at(nodes, p)?.id.clone()),
    };
    Some(Place {
        parent,
        index: *index,
    })
}

/// The children of folder `parent` (the root for `None`).
fn siblings_mut<'a>(
    nodes: &'a mut Vec<LauncherNode>,
    parent: Option<&str>,
) -> Option<&'a mut Vec<LauncherNode>> {
    let Some(parent) = parent else {
        return Some(nodes);
    };
    let path = path_of(nodes, parent)?;
    let mut level = nodes;
    for (depth, &i) in path.iter().enumerate() {
        let NodeKind::Folder { children, .. } = &mut level.get_mut(i)?.kind else {
            return None;
        };
        if depth + 1 == path.len() {
            return Some(children);
        }
        level = children;
    }
    None
}

/// Inserts `node` at `place`; `false` when its parent is not a folder.
pub fn insert(nodes: &mut Vec<LauncherNode>, place: &Place, node: LauncherNode) -> bool {
    let Some(level) = siblings_mut(nodes, place.parent.as_deref()) else {
        return false;
    };
    level.insert(place.index.min(level.len()), node);
    true
}

/// Removes the node with `id`, with its children.
pub fn remove(nodes: &mut Vec<LauncherNode>, id: &str) -> Option<LauncherNode> {
    let place = place_of(nodes, id)?;
    Some(siblings_mut(nodes, place.parent.as_deref())?.remove(place.index))
}

/// Replaces the kind of the node with `id`; a folder keeps its children.
pub fn update(nodes: &mut Vec<LauncherNode>, id: &str, mut kind: NodeKind) -> bool {
    let Some(place) = place_of(nodes, id) else {
        return false;
    };
    let Some(node) =
        siblings_mut(nodes, place.parent.as_deref()).and_then(|level| level.get_mut(place.index))
    else {
        return false;
    };
    if let (NodeKind::Folder { children: old, .. }, NodeKind::Folder { children, .. }) =
        (&mut node.kind, &mut kind)
    {
        *children = std::mem::take(old);
    }
    if node.kind == kind {
        return false;
    }
    node.kind = kind;
    true
}

/// Moves the node with `id` past the next sibling in `delta`'s direction
/// that `shown` accepts (hidden siblings are skipped); `false` at the end.
pub fn move_by(
    nodes: &mut Vec<LauncherNode>,
    id: &str,
    delta: isize,
    shown: impl Fn(&LauncherNode) -> bool,
) -> bool {
    let Some(place) = place_of(nodes, id) else {
        return false;
    };
    let Some(level) = siblings_mut(nodes, place.parent.as_deref()) else {
        return false;
    };
    let i = place.index;
    let target = if delta < 0 {
        (0..i).rev().find(|&j| shown(&level[j]))
    } else {
        (i + 1..level.len()).find(|&j| shown(&level[j]))
    };
    let Some(j) = target else {
        return false;
    };
    let node = level.remove(i);
    level.insert(j, node);
    true
}

/// Whether the node with `id` may move to `place`: `place` is in a folder
/// of the tree and not inside the node itself.
pub fn check_move(nodes: &[LauncherNode], id: &str, place: &Place) -> Result<(), MoveError> {
    let moved = path_of(nodes, id).ok_or(MoveError::NotFound)?;
    if let Some(parent) = &place.parent {
        let target = path_of(nodes, parent).ok_or(MoveError::NotFound)?;
        if target.starts_with(&moved) {
            return Err(MoveError::IntoItself);
        }
        if !matches!(find(nodes, parent).map(|n| &n.kind), Some(NodeKind::Folder { .. })) {
            return Err(MoveError::NotFound);
        }
    }
    Ok(())
}

/// Moving the node with `id` to `place` would leave it where it is
/// (before itself or before its next sibling).
pub fn is_noop(nodes: &[LauncherNode], id: &str, place: &Place) -> bool {
    place_of(nodes, id).is_some_and(|from| {
        from.parent == place.parent && (place.index == from.index || place.index == from.index + 1)
    })
}

/// Moves the node with `id` to `place` (as it is before the move).
pub fn move_to(nodes: &mut Vec<LauncherNode>, id: &str, place: &Place) -> Result<(), MoveError> {
    check_move(nodes, id, place)?;
    let from = place_of(nodes, id).ok_or(MoveError::NotFound)?;
    let node = remove(nodes, id).ok_or(MoveError::NotFound)?;
    let mut place = place.clone();
    if place.parent == from.parent && place.index > from.index {
        place.index -= 1;
    }
    insert(nodes, &place, node);
    Ok(())
}

/// A place that follows edits elsewhere in the tree: before a node, or at
/// the end of a folder (the root for `None`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Anchor {
    Before(String),
    End(Option<String>),
}

/// The anchor of `place`: the node now there, else the end of its folder.
pub fn anchor_of(nodes: &[LauncherNode], place: &Place) -> Anchor {
    let level = match &place.parent {
        None => Some(nodes),
        Some(id) => match find(nodes, id).map(|n| &n.kind) {
            Some(NodeKind::Folder { children, .. }) => Some(children.as_slice()),
            _ => None,
        },
    };
    match level.and_then(|l| l.get(place.index)) {
        Some(node) => Anchor::Before(node.id.clone()),
        None => Anchor::End(place.parent.clone()),
    }
}

/// Where `anchor` is now; the end of the root when its node is gone (or is
/// not a folder).
pub fn resolve(nodes: &[LauncherNode], anchor: &Anchor) -> Place {
    let root_end = || Place {
        parent: None,
        index: nodes.len(),
    };
    match anchor {
        Anchor::Before(id) => place_of(nodes, id).unwrap_or_else(root_end),
        Anchor::End(None) => root_end(),
        Anchor::End(Some(id)) => match find(nodes, id).map(|n| &n.kind) {
            Some(NodeKind::Folder { children, .. }) => Place {
                parent: Some(id.clone()),
                index: children.len(),
            },
            _ => root_end(),
        },
    }
}

/// Number of nodes below the node with `id` (0 for a non-folder).
pub fn descendants(nodes: &[LauncherNode], id: &str) -> usize {
    fn count(nodes: &[LauncherNode]) -> usize {
        nodes
            .iter()
            .map(|n| match &n.kind {
                NodeKind::Folder { children, .. } => 1 + count(children),
                _ => 1,
            })
            .sum()
    }
    match find(nodes, id).map(|n| &n.kind) {
        Some(NodeKind::Folder { children, .. }) => count(children),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `a`, folder `f` [`b`, folder `g` [`c`]], `d`: separators by id.
    fn tree() -> Vec<LauncherNode> {
        serde_json::from_str(
            r#"[{"type": "separator", "id": "a"},
                {"type": "folder", "id": "f", "name": "F", "children": [
                    {"type": "separator", "id": "b"},
                    {"type": "folder", "id": "g", "name": "G", "children": [
                        {"type": "separator", "id": "c"}]}]},
                {"type": "separator", "id": "d"}]"#,
        )
        .unwrap()
    }

    fn ids(nodes: &[LauncherNode]) -> String {
        nodes
            .iter()
            .map(|n| match &n.kind {
                NodeKind::Folder { children, .. } => format!("{}[{}]", n.id, ids(children)),
                _ => n.id.clone(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn sep(id: &str) -> LauncherNode {
        LauncherNode {
            id: id.to_owned(),
            kind: NodeKind::Separator,
        }
    }

    fn place(parent: Option<&str>, index: usize) -> Place {
        Place {
            parent: parent.map(str::to_owned),
            index,
        }
    }

    #[test]
    fn finds_paths_and_places() {
        let t = tree();
        assert_eq!(path_of(&t, "c"), Some(vec![1, 1, 0]));
        assert_eq!(place_of(&t, "c"), Some(place(Some("g"), 0)));
        assert_eq!(place_of(&t, "d"), Some(place(None, 2)));
        assert_eq!(path_of(&t, "zz"), None);
        assert_eq!(find(&t, "g").map(|n| n.id.as_str()), Some("g"));
    }

    #[test]
    fn inserts_before_a_place_or_appends() {
        let mut t = tree();
        let before_b = place_of(&t, "b").unwrap();
        assert!(insert(&mut t, &before_b, sep("n")));
        assert!(insert(&mut t, &place(Some("g"), 99), sep("m")));
        assert!(insert(&mut t, &place(None, 0), sep("r")));
        assert_eq!(ids(&t), "r a f[n b g[c m]] d");
        assert!(!insert(&mut t, &place(Some("a"), 0), sep("x")), "a separator has no children");
    }

    #[test]
    fn removes_with_children() {
        let mut t = tree();
        assert_eq!(remove(&mut t, "g").map(|n| n.id), Some("g".to_owned()));
        assert_eq!(ids(&t), "a f[b] d");
        assert!(remove(&mut t, "c").is_none());
    }

    #[test]
    fn updates_keep_folder_children() {
        let mut t = tree();
        let renamed = NodeKind::Folder {
            name: "Renamed".to_owned(),
            children: Vec::new(),
        };
        assert!(update(&mut t, "f", renamed));
        assert!(
            matches!(&t[1].kind, NodeKind::Folder { name, children } if name == "Renamed" && children.len() == 2)
        );
        assert!(!update(&mut t, "a", NodeKind::Separator), "unchanged");
        assert!(!update(&mut t, "zz", NodeKind::Separator));
    }

    #[test]
    fn moves_within_a_level_skipping_hidden_siblings() {
        let mut t = tree();
        assert!(move_by(&mut t, "d", -1, |_| true));
        assert_eq!(ids(&t), "a d f[b g[c]]");
        assert!(move_by(&mut t, "a", 1, |n| n.id != "d"), "d is hidden");
        assert_eq!(ids(&t), "d f[b g[c]] a");
        assert!(!move_by(&mut t, "a", 1, |_| true), "already last");
        assert!(!move_by(&mut t, "b", -1, |_| true), "already first in its folder");
    }

    #[test]
    fn moves_anywhere_but_into_itself() {
        let mut t = tree();
        assert_eq!(move_to(&mut t, "f", &place(Some("g"), 0)), Err(MoveError::IntoItself));
        assert_eq!(move_to(&mut t, "f", &place(Some("f"), 0)), Err(MoveError::IntoItself));
        assert_eq!(move_to(&mut t, "a", &place(Some("d"), 0)), Err(MoveError::NotFound));
        assert_eq!(ids(&t), "a f[b g[c]] d");
        assert_eq!(move_to(&mut t, "a", &place(None, 3)), Ok(()));
        assert_eq!(ids(&t), "f[b g[c]] d a");
        assert_eq!(move_to(&mut t, "c", &place(None, 0)), Ok(()));
        assert_eq!(ids(&t), "c f[b g[]] d a");
    }

    #[test]
    fn checks_moves_without_changing_the_tree() {
        let t = tree();
        assert_eq!(check_move(&t, "g", &place(Some("g"), 0)), Err(MoveError::IntoItself));
        assert_eq!(check_move(&t, "zz", &place(None, 0)), Err(MoveError::NotFound));
        assert_eq!(check_move(&t, "c", &place(Some("f"), 0)), Ok(()));
        assert!(is_noop(&t, "b", &place(Some("f"), 0)), "before itself");
        assert!(is_noop(&t, "b", &place(Some("f"), 1)), "before its next sibling");
        assert!(!is_noop(&t, "b", &place(Some("f"), 2)));
        assert!(!is_noop(&t, "b", &place(None, 0)));
    }

    #[test]
    fn counts_descendants() {
        let t = tree();
        assert_eq!(descendants(&t, "f"), 3);
        assert_eq!(descendants(&t, "g"), 1);
        assert_eq!(descendants(&t, "a"), 0);
    }
    #[test]
    fn anchor_of_a_place_names_the_node_after_it_or_the_folder_end() {
        let nodes = tree();
        assert_eq!(anchor_of(&nodes, &place(None, 1)), Anchor::Before("f".into()));
        assert_eq!(anchor_of(&nodes, &place(Some("f"), 2)), Anchor::End(Some("f".into())));
        assert_eq!(anchor_of(&nodes, &place(None, 9)), Anchor::End(None));
    }

    #[test]
    fn anchors_resolve_where_their_nodes_are_now() {
        let mut nodes = tree();
        let before_d = Anchor::Before("d".into());
        let end_g = Anchor::End(Some("g".into()));
        assert_eq!(resolve(&nodes, &before_d), place(None, 2));
        assert_eq!(resolve(&nodes, &end_g), place(Some("g"), 1));
        remove(&mut nodes, "a");
        let at = resolve(&nodes, &end_g);
        assert!(insert(&mut nodes, &at, sep("x")));
        assert_eq!(resolve(&nodes, &before_d), place(None, 1));
        assert_eq!(resolve(&nodes, &end_g), place(Some("g"), 2));
    }

    #[test]
    fn anchors_of_removed_nodes_resolve_to_the_root_end() {
        let mut nodes = tree();
        remove(&mut nodes, "f");
        assert_eq!(resolve(&nodes, &Anchor::Before("b".into())), place(None, 2));
        assert_eq!(resolve(&nodes, &Anchor::End(Some("g".into()))), place(None, 2));
        assert_eq!(resolve(&nodes, &Anchor::End(Some("a".into()))), place(None, 2));
    }
}
