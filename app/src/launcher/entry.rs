//! Rows of the launcher levels: tree nodes, recent files and apps (no GTK)

use super::recent_files::RecentFile;
use super::{CommandItem, LauncherNode, NodeKind};
use crate::state::tree::Row;

/// A row of a launcher level.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    /// A tree node, by its path of child indices from the root.
    Node(Vec<usize>),
    /// A recent file of the command at `command`.
    Recent {
        command: Vec<usize>,
        file: RecentFile,
    },
    /// An installed app, by its index in the app list.
    App(usize),
}

/// The node at `path`.
pub fn node_at<'a>(nodes: &'a [LauncherNode], path: &[usize]) -> Option<&'a LauncherNode> {
    let (first, rest) = path.split_first()?;
    let mut node = nodes.get(*first)?;
    for &i in rest {
        let NodeKind::Folder { children, .. } = &node.kind else {
            return None;
        };
        node = children.get(i)?;
    }
    Some(node)
}

/// The command at `path`.
pub fn command_at<'a>(nodes: &'a [LauncherNode], path: &[usize]) -> Option<&'a CommandItem> {
    match &node_at(nodes, path)?.kind {
        NodeKind::Command(c) => Some(c),
        _ => None,
    }
}

/// What a node's row can do.
pub fn node_row(node: &LauncherNode) -> Row {
    match &node.kind {
        NodeKind::Folder { .. } => Row::FOLDER,
        NodeKind::Command(c) if c.history_source.is_some() => Row::BRANCH_LEAF,
        NodeKind::Command(_) => Row::LEAF,
        NodeKind::Separator | NodeKind::Unknown => Row::SEPARATOR,
    }
}

impl Entry {
    pub fn row(&self, nodes: &[LauncherNode]) -> Row {
        match self {
            Entry::Node(path) => node_at(nodes, path).map_or(Row::SEPARATOR, node_row),
            Entry::Recent { .. } | Entry::App(_) => Row::LEAF,
        }
    }
}

/// Rows of the root level.
pub fn root_entries(nodes: &[LauncherNode]) -> Vec<Entry> {
    (0..nodes.len()).map(|i| Entry::Node(vec![i])).collect()
}

/// Children of a folder entry; `None` for anything else.
pub fn folder_children(nodes: &[LauncherNode], entry: &Entry) -> Option<Vec<Entry>> {
    let Entry::Node(path) = entry else {
        return None;
    };
    let NodeKind::Folder { children, .. } = &node_at(nodes, path)?.kind else {
        return None;
    };
    Some(
        (0..children.len())
            .map(|i| Entry::Node(path.iter().copied().chain([i]).collect()))
            .collect(),
    )
}

/// Children of an entry: a folder's nodes or a command's recent files
/// (read from disk).
pub fn children(nodes: &[LauncherNode], entry: &Entry) -> Vec<Entry> {
    if let Some(entries) = folder_children(nodes, entry) {
        return entries;
    }
    let Entry::Node(path) = entry else {
        return Vec::new();
    };
    let Some(source) = command_at(nodes, path).and_then(|c| c.history_source.as_ref()) else {
        return Vec::new();
    };
    source
        .recent_files()
        .into_iter()
        .map(|file| Entry::Recent {
            command: path.clone(),
            file,
        })
        .collect()
}

/// Names from the root down to the node at `path`: `Dev › Code`.
pub fn breadcrumb(nodes: &[LauncherNode], path: &[usize]) -> String {
    (1..=path.len())
        .filter_map(|n| match &node_at(nodes, &path[..n])?.kind {
            NodeKind::Folder { name, .. } => Some(name.as_str()),
            NodeKind::Command(c) => Some(c.name.as_str()),
            NodeKind::Separator | NodeKind::Unknown => None,
        })
        .collect::<Vec<_>>()
        .join(" › ")
}

/// A command found anywhere in the tree, for search.
#[derive(Debug, PartialEq)]
pub struct Searchable<'a> {
    pub path: Vec<usize>,
    pub command: &'a CommandItem,
    /// Folder names down to the command: `Dev › Code`.
    pub breadcrumb: String,
}

/// Every command in the tree, depth first.
pub fn searchable(nodes: &[LauncherNode]) -> Vec<Searchable<'_>> {
    fn walk<'a>(
        nodes: &'a [LauncherNode],
        path: &mut Vec<usize>,
        folders: &mut Vec<&'a str>,
        out: &mut Vec<Searchable<'a>>,
    ) {
        for (i, node) in nodes.iter().enumerate() {
            path.push(i);
            match &node.kind {
                NodeKind::Folder { name, children } => {
                    folders.push(name);
                    walk(children, path, folders, out);
                    folders.pop();
                },
                NodeKind::Command(command) => out.push(Searchable {
                    path: path.clone(),
                    command,
                    breadcrumb: folders
                        .iter()
                        .copied()
                        .chain([command.name.as_str()])
                        .collect::<Vec<_>>()
                        .join(" › "),
                }),
                NodeKind::Separator | NodeKind::Unknown => {},
            }
            path.pop();
        }
    }
    let mut out = Vec::new();
    walk(nodes, &mut Vec::new(), &mut Vec::new(), &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::launcher::LauncherConfig;

    fn nodes() -> Vec<LauncherNode> {
        serde_json::from_str::<LauncherConfig>(
            r#"{"items": [
                {"type": "command", "name": "Files", "command": "nautilus",
                 "historySource": {"type": "vscode", "path": "/nonexistent"}},
                {"type": "folder", "name": "Dev", "children": [
                    {"type": "command", "name": "Code", "command": "code"},
                    {"type": "separator"},
                    {"type": "folder", "name": "Tools", "children": [
                        {"type": "command", "name": "Calc", "command": "calc"}
                    ]}
                ]},
                {"type": "separator"}
            ]}"#,
        )
        .unwrap()
        .into_tree()
    }

    #[test]
    fn paths_walk_folders() {
        let n = nodes();
        assert_eq!(command_at(&n, &[1, 2, 0]).map(|c| c.name.as_str()), Some("Calc"));
        assert!(node_at(&n, &[1, 9]).is_none());
        assert!(node_at(&n, &[0, 0]).is_none(), "commands have no children");
        assert!(command_at(&n, &[1]).is_none());
    }

    #[test]
    fn rows_follow_node_types() {
        let n = nodes();
        let rows: Vec<Row> = root_entries(&n).iter().map(|e| e.row(&n)).collect();
        assert_eq!(rows, [Row::BRANCH_LEAF, Row::FOLDER, Row::SEPARATOR]);
        assert_eq!(Entry::App(0).row(&n), Row::LEAF);
    }

    #[test]
    fn folder_children_are_paths_below_it() {
        let n = nodes();
        let dev = Entry::Node(vec![1]);
        assert_eq!(
            children(&n, &dev),
            [Entry::Node(vec![1, 0]), Entry::Node(vec![1, 1]), Entry::Node(vec![1, 2])]
        );
        assert_eq!(folder_children(&n, &Entry::Node(vec![0])), None);
        assert!(children(&n, &Entry::Node(vec![0])).is_empty(), "unreadable history");
        assert!(children(&n, &Entry::App(0)).is_empty());
    }

    #[test]
    fn breadcrumbs_name_every_folder() {
        assert_eq!(breadcrumb(&nodes(), &[1, 2, 0]), "Dev › Tools › Calc");
    }

    #[test]
    fn search_finds_commands_at_any_depth_with_their_folders() {
        let n = nodes();
        let found: Vec<(Vec<usize>, String)> =
            searchable(&n).into_iter().map(|s| (s.path, s.breadcrumb)).collect();
        assert_eq!(
            found,
            [
                (vec![0], "Files".to_string()),
                (vec![1, 0], "Dev › Code".to_string()),
                (vec![1, 2, 0], "Dev › Tools › Calc".to_string()),
            ]
        );
    }
}
