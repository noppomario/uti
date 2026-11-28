//! Rows of the launcher levels: tree nodes, generated rows and recent files
//! (no GTK)

use std::borrow::Cow;
use std::collections::HashSet;
use std::path::Path;

use super::apps::{self, AppMeta, Category};
use super::edit::{self, Place};
use super::external;
use super::places::{self, DirItem, LISTING_CAP};
use super::power::PowerAction;
use super::recent_files::{self, RecentFile};
use super::{CommandItem, HistorySource, LauncherNode, NodeKind};
use crate::state::drop::Spot;
use crate::state::menu::LauncherRow;
use crate::state::tree::Row;

/// Where rows come from: the tree, installed apps, the app names that
/// have recent files and folders on disk.
#[derive(Clone, Copy)]
pub struct Sources<'a> {
    pub nodes: &'a [LauncherNode],
    pub apps: &'a [AppMeta],
    /// Application names in `recently-used.xbel`.
    pub recent_apps: &'a HashSet<String>,
    /// Reads a folder (path or URI) when its level opens.
    pub list_dir: &'a dyn Fn(&str) -> Vec<DirItem>,
    /// For `~` in `file` and `directory` paths.
    pub home: &'a Path,
}

/// What opens a recent file.
#[derive(Debug, Clone, PartialEq)]
pub enum Opener {
    /// The command at this path.
    Command(Vec<usize>),
    /// The installed app at this index.
    App(usize),
    /// The file's default app (`recent` nodes).
    Default,
}

/// A row of a launcher level.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    /// A tree node, by its path of child indices from the root.
    Node(Vec<usize>),
    /// A category of installed apps.
    Category(Category),
    /// An installed app, by its index in the app list.
    App(usize),
    Power(PowerAction),
    Recent {
        opener: Opener,
        file: RecentFile,
    },
    /// A file or folder listed by a `directory` level (path or URI).
    Listed {
        path: String,
        dir: bool,
    },
    /// The first row of a folder's level: opens it in the file manager.
    OpenDir(String),
    /// The last row of a folder's level when `hidden` entries are left
    /// out: opens it in the file manager.
    More {
        dir: String,
        hidden: usize,
    },
    /// A separator in a generated level.
    Gap,
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

/// `node` has a row: not [`NodeKind::Raw`], and for an `app` node, its app
/// is in `apps`.
pub fn node_shown(apps: &[AppMeta], node: &LauncherNode) -> bool {
    match &node.kind {
        NodeKind::App { desktop_id, .. } => apps.iter().any(|a| a.id == *desktop_id),
        kind => kind.is_known(),
    }
}

/// The command at `path`.
pub fn command_at<'a>(nodes: &'a [LauncherNode], path: &[usize]) -> Option<&'a CommandItem> {
    match &node_at(nodes, path)?.kind {
        NodeKind::Command(c) => Some(c),
        _ => None,
    }
}

impl<'a> Sources<'a> {
    /// Index of the installed app with this desktop id.
    pub fn app_index(&self, desktop_id: &str) -> Option<usize> {
        self.apps.iter().position(|a| a.id == desktop_id)
    }

    /// The installed app an `app` node at `path` names.
    pub fn app_at(&self, path: &[usize]) -> Option<usize> {
        match &node_at(self.nodes, path)?.kind {
            NodeKind::App { desktop_id, .. } => self.app_index(desktop_id),
            _ => None,
        }
    }

    fn has_recent(&self, app: usize) -> bool {
        self.apps
            .get(app)
            .is_some_and(|a| apps::xbel_names(a).iter().any(|n| self.recent_apps.contains(n)))
    }

    fn app_row(&self, app: usize) -> Row {
        if self.has_recent(app) {
            Row::BRANCH_LEAF
        } else {
            Row::LEAF
        }
    }

    /// The action of a `power` node of one action at `path`.
    pub fn power_at(&self, path: &[usize]) -> Option<PowerAction> {
        match &node_at(self.nodes, path)?.kind {
            NodeKind::Power { action, .. } => *action,
            _ => None,
        }
    }

    /// The node at `path` has a row: not [`NodeKind::Raw`], and for an
    /// `app` node, its app is installed.
    pub fn shown(&self, path: &[usize]) -> bool {
        node_at(self.nodes, path).is_some_and(|n| node_shown(self.apps, n))
    }

    /// Rows for the nodes at `paths`; `app` nodes of missing apps are left
    /// out.
    fn level(&self, paths: impl Iterator<Item = Vec<usize>>) -> Vec<Entry> {
        paths.filter(|path| self.shown(path)).map(Entry::Node).collect()
    }

    fn categories(&self) -> Vec<Entry> {
        apps::group(self.apps).into_iter().map(|(c, _)| Entry::Category(c)).collect()
    }

    /// Rows of the root level.
    pub fn root_entries(&self) -> Vec<Entry> {
        self.level((0..self.nodes.len()).map(|i| vec![i]))
    }

    /// Children of a menu row (folder, `apps`, `power`, category); `None`
    /// for anything else.
    pub fn menu_children(&self, entry: &Entry) -> Option<Vec<Entry>> {
        match entry {
            Entry::Category(c) => {
                Some(apps::in_category(self.apps, *c).into_iter().map(Entry::App).collect())
            },
            Entry::Node(path) => match &node_at(self.nodes, path)?.kind {
                NodeKind::Folder { children, .. } => {
                    Some(self.level(
                        (0..children.len()).map(|i| path.iter().copied().chain([i]).collect()),
                    ))
                },
                NodeKind::Apps { category: None, .. } => Some(self.categories()),
                NodeKind::Apps {
                    category: Some(c), ..
                } => Some(apps::in_category(self.apps, *c).into_iter().map(Entry::App).collect()),
                NodeKind::Power { action: None, .. } => Some(power_entries()),
                _ => None,
            },
            _ => None,
        }
    }

    /// Children of an entry: a menu's rows, or the recent files of a
    /// command or app (read from disk).
    pub fn children(&self, entry: &Entry) -> Vec<Entry> {
        if let Some(entries) = self.menu_children(entry) {
            return entries;
        }
        let (opener, files) = match entry {
            Entry::Node(path) => match &node_at(self.nodes, path).map(|n| &n.kind) {
                Some(NodeKind::Directory { path, .. }) => {
                    return self.dir_children(&places::expand(path, self.home));
                },
                Some(NodeKind::Recent { .. }) => {
                    (Opener::Default, recent_files::get_recent_files_from_xbel(None, None))
                },
                _ => self.node_recent(path),
            },
            Entry::Listed { path, dir: true } => return self.dir_children(path),
            Entry::App(app) => (Opener::App(*app), self.app_recent(*app)),
            _ => return Vec::new(),
        };
        files
            .into_iter()
            .map(|file| Entry::Recent {
                opener: opener.clone(),
                file,
            })
            .collect()
    }

    /// Recent files of the `app` or `command` node at `path`.
    fn node_recent(&self, path: &[usize]) -> (Opener, Vec<RecentFile>) {
        if let Some(app) = self.app_at(path) {
            return (Opener::App(app), self.app_recent(app));
        }
        match command_at(self.nodes, path).and_then(|c| c.history_source.as_ref()) {
            Some(source) => (Opener::Command(path.to_vec()), source.recent_files()),
            None => (Opener::Default, Vec::new()),
        }
    }

    /// Rows of a folder's level: "open this folder", then its listing, then
    /// how many entries were left out.
    fn dir_children(&self, dir: &str) -> Vec<Entry> {
        let (items, hidden) = places::listing((self.list_dir)(dir), LISTING_CAP);
        let mut rows = vec![Entry::OpenDir(dir.to_string())];
        if !items.is_empty() {
            rows.push(Entry::Gap);
        }
        rows.extend(items.into_iter().map(|i| Entry::Listed {
            path: i.path,
            dir: i.dir,
        }));
        if hidden > 0 {
            rows.push(Entry::More {
                dir: dir.to_string(),
                hidden,
            });
        }
        rows
    }

    fn app_recent(&self, app: usize) -> Vec<RecentFile> {
        if !self.has_recent(app) {
            return Vec::new();
        }
        recent_files::get_recent_files_for_apps(&apps::xbel_names(&self.apps[app]))
    }

    /// What a row can do.
    pub fn row(&self, entry: &Entry) -> Row {
        match entry {
            Entry::Node(path) => match node_at(self.nodes, path).map(|n| &n.kind) {
                Some(
                    NodeKind::Folder { .. }
                    | NodeKind::Apps { .. }
                    | NodeKind::Power { action: None, .. }
                    | NodeKind::Directory { .. }
                    | NodeKind::Recent { .. },
                ) => Row::FOLDER,
                Some(NodeKind::Power { .. } | NodeKind::File { .. } | NodeKind::Url { .. }) => {
                    Row::LEAF
                },
                Some(NodeKind::Command(c)) if c.history_source.is_some() => Row::BRANCH_LEAF,
                Some(NodeKind::Command(_)) => Row::LEAF,
                Some(NodeKind::App { .. }) => {
                    self.app_at(path).map_or(Row::SEPARATOR, |a| self.app_row(a))
                },
                Some(NodeKind::Separator | NodeKind::Raw(_)) | None => Row::SEPARATOR,
            },
            Entry::Category(_) => Row::FOLDER,
            Entry::App(app) => self.app_row(*app),
            Entry::Listed { dir: true, .. } => Row::FOLDER,
            Entry::Power(_)
            | Entry::Recent { .. }
            | Entry::Listed { .. }
            | Entry::OpenDir(_)
            | Entry::More { .. } => Row::LEAF,
            Entry::Gap => Row::SEPARATOR,
        }
    }

    /// Display name of a node: folders, commands and apps by their name,
    /// `apps` and `power` by theirs or that of their category or action,
    /// files and folders by theirs or their file name, links by theirs or
    /// their host.
    pub fn node_name(&self, path: &[usize]) -> Option<Cow<'a, str>> {
        fn named(name: &Option<String>, default: impl FnOnce() -> String) -> Cow<'_, str> {
            name.as_deref().map_or_else(|| Cow::Owned(default()), Cow::Borrowed)
        }
        let nodes: &'a [LauncherNode] = self.nodes;
        Some(Cow::Borrowed(match &node_at(nodes, path)?.kind {
            NodeKind::File { path, name, .. } | NodeKind::Directory { path, name, .. } => {
                return Some(named(name, || places::base_name(path)));
            },
            NodeKind::Url { url, name, .. } => return Some(named(name, || external::host(url))),
            NodeKind::Recent { name } => name.as_deref().unwrap_or("Recent Files"),
            NodeKind::Folder { name, .. } => name,
            NodeKind::Command(c) => &c.name,
            NodeKind::Apps { name, category } => name
                .as_deref()
                .unwrap_or_else(|| category.map_or("Applications", Category::label)),
            NodeKind::Power { name, action } => {
                name.as_deref().unwrap_or_else(|| action.map_or("Power", PowerAction::label))
            },
            NodeKind::App {
                name: Some(name), ..
            } if self.app_at(path).is_some() => name,
            NodeKind::App { .. } => {
                let apps: &'a [AppMeta] = self.apps;
                &apps[self.app_at(path)?].name
            },
            NodeKind::Separator | NodeKind::Raw(_) => return None,
        }))
    }

    /// Names from the root down to the node at `path`: `Dev › Code`.
    pub fn breadcrumb(&self, path: &[usize]) -> String {
        (1..=path.len())
            .filter_map(|n| self.node_name(&path[..n]))
            .collect::<Vec<_>>()
            .join(" › ")
    }

    /// What typing searches: commands and `app` nodes of the whole tree
    /// (depth first), then installed apps not already in the tree.
    pub fn searchable(&self) -> Vec<Searchable<'_>> {
        fn walk<'a>(
            src: &Sources<'a>,
            nodes: &'a [LauncherNode],
            path: &mut Vec<usize>,
            out: &mut Vec<Searchable<'a>>,
            in_tree: &mut HashSet<usize>,
        ) {
            for (i, node) in nodes.iter().enumerate() {
                path.push(i);
                match &node.kind {
                    NodeKind::Folder { children, .. } => walk(src, children, path, out, in_tree),
                    NodeKind::Command(c) => out.push(Searchable {
                        entry: Entry::Node(path.clone()),
                        name: Cow::Borrowed(&c.name),
                        description: &c.command,
                    }),
                    NodeKind::File { path: p, .. }
                    | NodeKind::Directory { path: p, .. }
                    | NodeKind::Url { url: p, .. } => out.push(Searchable {
                        entry: Entry::Node(path.clone()),
                        name: src.node_name(path).unwrap_or_default(),
                        description: p,
                    }),
                    NodeKind::App { name, .. } => {
                        if let Some(a) = src.app_at(path) {
                            in_tree.insert(a);
                            out.push(Searchable {
                                entry: Entry::Node(path.clone()),
                                name: Cow::Borrowed(name.as_deref().unwrap_or(&src.apps[a].name)),
                                description: &src.apps[a].description,
                            });
                        }
                    },
                    _ => {},
                }
                path.pop();
            }
        }
        let mut out = Vec::new();
        let mut in_tree = HashSet::new();
        walk(self, self.nodes, &mut Vec::new(), &mut out, &mut in_tree);
        out.extend(self.apps.iter().enumerate().filter(|(i, _)| !in_tree.contains(i)).map(
            |(i, a)| Searchable {
                entry: Entry::App(i),
                name: Cow::Borrowed(&a.name),
                description: &a.description,
            },
        ));
        out
    }
}

impl Sources<'_> {
    /// What row `index` of `level` is, for its menu (an index past the end
    /// is the empty space below the rows).
    pub fn menu_row(&self, level: &[Entry], index: usize, searching: bool) -> LauncherRow {
        match level.get(index) {
            None => LauncherRow::Empty,
            Some(Entry::Node(path)) => LauncherRow::Node {
                separator: matches!(
                    node_at(self.nodes, path).map(|n| &n.kind),
                    Some(NodeKind::Separator)
                ),
                index,
                len: level.len(),
                searching,
            },
            Some(Entry::Recent { opener, .. }) => LauncherRow::Recent {
                removable: self.system_history(opener),
            },
            Some(Entry::Listed { .. }) => LauncherRow::Listed,
            Some(Entry::OpenDir(_) | Entry::More { .. } | Entry::Gap) => LauncherRow::Fixed,
            Some(_) => LauncherRow::Generated,
        }
    }

    /// Recent files of `opener` come from the system `recently-used.xbel`.
    fn system_history(&self, opener: &Opener) -> bool {
        match opener {
            Opener::App(_) | Opener::Default => true,
            Opener::Command(path) => matches!(
                command_at(self.nodes, path).and_then(|c| c.history_source.as_ref()),
                Some(HistorySource::RecentlyUsed { path: None, .. })
            ),
        }
    }

    /// Where a node created from row `at` of a level goes: before that
    /// row's node, or at the end of the level for `None` (empty space, an
    /// empty folder). `owners` are the rows the open levels belong to,
    /// from the root down to this level; `None` when the level is not a
    /// folder of the tree.
    pub fn new_place(&self, owners: &[&Entry], at: Option<&Entry>) -> Option<Place> {
        if let Some(Entry::Node(path)) = at {
            return edit::place_of(self.nodes, &node_at(self.nodes, path)?.id);
        }
        let Some(owner) = owners.last() else {
            return Some(Place {
                parent: None,
                index: self.nodes.len(),
            });
        };
        let Entry::Node(path) = owner else {
            return None;
        };
        let node = node_at(self.nodes, path)?;
        let NodeKind::Folder { children, .. } = &node.kind else {
            return None;
        };
        Some(Place {
            parent: Some(node.id.clone()),
            index: children.len(),
        })
    }

    /// Where "Add to Launcher" puts a generated row of the level `owners`
    /// lead to: before the nearest tree node it was generated by, or at the
    /// end of the root (search results).
    pub fn add_place(&self, owners: &[&Entry]) -> Place {
        owners
            .iter()
            .rev()
            .find_map(|e| match e {
                Entry::Node(path) => edit::place_of(self.nodes, &node_at(self.nodes, path)?.id),
                _ => None,
            })
            .unwrap_or(Place {
                parent: None,
                index: self.nodes.len(),
            })
    }

    /// The node "Add to Launcher" creates for a generated row.
    pub fn node_for(&self, entry: &Entry) -> Option<NodeKind> {
        Some(match entry {
            Entry::Category(c) => NodeKind::Apps {
                name: None,
                category: Some(*c),
            },
            Entry::App(a) => NodeKind::App {
                desktop_id: self.apps.get(*a)?.id.clone(),
                name: None,
                icon: None,
            },
            Entry::Power(p) => NodeKind::Power {
                name: None,
                action: Some(*p),
            },
            Entry::Listed { path, dir } => {
                let path = places::contract(path, self.home);
                if *dir {
                    NodeKind::Directory {
                        path,
                        name: None,
                        icon: None,
                    }
                } else {
                    NodeKind::File {
                        path,
                        name: None,
                        icon: None,
                    }
                }
            },
            Entry::Node(_)
            | Entry::Recent { .. }
            | Entry::OpenDir(_)
            | Entry::More { .. }
            | Entry::Gap => return None,
        })
    }
}

/// What dragging a row carries.
#[derive(Debug, Clone, PartialEq)]
pub enum Drag {
    /// A tree node, moved.
    Move(String),
    /// A generated row, added as this new node (as "Add to Launcher").
    Copy(NodeKind),
}

impl Sources<'_> {
    /// What dragging `entry` carries; `None` for rows that cannot be
    /// dragged (recent files).
    pub fn drag(&self, entry: &Entry) -> Option<Drag> {
        match entry {
            Entry::Node(path) => Some(Drag::Move(node_at(self.nodes, path)?.id.clone())),
            _ => self.node_for(entry).map(Drag::Copy),
        }
    }

    /// `entry` is a folder of the tree: it takes drops and opens while
    /// dragging over it.
    pub fn is_folder(&self, entry: &Entry) -> bool {
        matches!(entry, Entry::Node(path)
            if matches!(node_at(self.nodes, path).map(|n| &n.kind), Some(NodeKind::Folder { .. })))
    }

    /// Where a drop at `spot` in `level` (the open levels lead to it
    /// through `owners`) puts a node; `None` when the level is not a folder
    /// of the tree.
    pub fn drop_place(&self, owners: &[&Entry], level: &[Entry], spot: Spot) -> Option<Place> {
        match spot {
            Spot::Before(i) => match level.get(i)? {
                at @ Entry::Node(_) => self.new_place(owners, Some(at)),
                _ => None,
            },
            Spot::Into(i) => {
                let Entry::Node(path) = level.get(i)? else {
                    return None;
                };
                let node = node_at(self.nodes, path)?;
                let NodeKind::Folder { children, .. } = &node.kind else {
                    return None;
                };
                Some(Place {
                    parent: Some(node.id.clone()),
                    index: children.len(),
                })
            },
            Spot::End => self.new_place(owners, None),
        }
    }
}

fn power_entries() -> Vec<Entry> {
    PowerAction::ALL.into_iter().map(Entry::Power).collect()
}

/// A search candidate: its row and the text matched.
#[derive(Debug, PartialEq)]
pub struct Searchable<'a> {
    pub entry: Entry,
    pub name: Cow<'a, str>,
    pub description: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::launcher::LauncherConfig;

    fn nodes(json: &str) -> Vec<LauncherNode> {
        serde_json::from_str::<LauncherConfig>(json).unwrap().into_tree()
    }

    fn tree() -> Vec<LauncherNode> {
        nodes(
            r#"{"items": [
                {"type": "command", "name": "Files", "command": "nautilus",
                 "historySource": {"type": "vscode", "path": "/nonexistent"}},
                {"type": "folder", "name": "Dev", "children": [
                    {"type": "command", "name": "Code", "command": "code"},
                    {"type": "separator"},
                    {"type": "folder", "name": "Tools", "children": [
                        {"type": "command", "name": "Calc", "command": "calc"},
                        {"type": "app", "desktopId": "editor.desktop"}
                    ]}
                ]},
                {"type": "separator"}
            ]}"#,
        )
    }

    fn apps() -> Vec<AppMeta> {
        let app = |id: &str, name: &str, category: &str| AppMeta {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("{name} app"),
            exec: Some(id.trim_end_matches(".desktop").to_string()),
            categories: vec![category.to_string()],
        };
        vec![
            app("firefox.desktop", "Firefox", "Network"),
            app("editor.desktop", "Editor", "Development"),
            app("chess.desktop", "Chess", "Game"),
        ]
    }

    fn with<R>(nodes: &[LauncherNode], f: impl FnOnce(Sources<'_>) -> R) -> R {
        let apps = apps();
        let recent: HashSet<String> = ["editor".to_string()].into();
        f(Sources {
            nodes,
            apps: &apps,
            recent_apps: &recent,
            list_dir: &fake_dir,
            home: Path::new("/home/u"),
        })
    }

    /// `/home/u/Docs`: a hidden file, `b.txt`, folder `Sub`; `/big`: more
    /// files than a level lists; anything else empty.
    fn fake_dir(dir: &str) -> Vec<DirItem> {
        let item = |name: &str, d: bool| DirItem {
            name: name.to_string(),
            path: format!("{dir}/{name}"),
            dir: d,
        };
        match dir {
            "/home/u/Docs" => vec![item(".hidden", false), item("b.txt", false), item("Sub", true)],
            "/big" => (0..LISTING_CAP + 3).map(|i| item(&format!("f{i:03}"), false)).collect(),
            _ => Vec::new(),
        }
    }

    #[test]
    fn directories_list_open_row_then_folders_then_files() {
        let n = nodes(
            r#"{"items": [{"type": "directory", "path": "~/Docs"},
                          {"type": "directory", "path": "/big", "name": "Big"},
                          {"type": "directory", "path": "/empty"}]}"#,
        );
        with(&n, |s| {
            let root = s.root_entries();
            assert_eq!(s.row(&root[0]), Row::FOLDER);
            assert_eq!(s.node_name(&[0]).as_deref(), Some("Docs"));
            assert_eq!(s.node_name(&[1]).as_deref(), Some("Big"));
            let docs = s.children(&root[0]);
            let dir = |p: &str| Entry::Listed {
                path: p.to_string(),
                dir: true,
            };
            assert_eq!(
                docs,
                [
                    Entry::OpenDir("/home/u/Docs".to_string()),
                    Entry::Gap,
                    dir("/home/u/Docs/Sub"),
                    Entry::Listed {
                        path: "/home/u/Docs/b.txt".to_string(),
                        dir: false
                    },
                ]
            );
            let rows: Vec<Row> = docs.iter().map(|e| s.row(e)).collect();
            assert_eq!(rows, [Row::LEAF, Row::SEPARATOR, Row::FOLDER, Row::LEAF]);
            assert_eq!(s.menu_children(&root[0]), None, "read from disk");
            let big = s.children(&root[1]);
            assert_eq!(big.len(), 2 + LISTING_CAP + 1);
            assert_eq!(
                big.last(),
                Some(&Entry::More {
                    dir: "/big".to_string(),
                    hidden: 3
                })
            );
            assert_eq!(s.children(&root[2]), [Entry::OpenDir("/empty".to_string())]);
        });
    }

    #[test]
    fn listed_rows_open_their_folders_and_become_nodes() {
        let n = tree();
        with(&n, |s| {
            let sub = Entry::Listed {
                path: "/home/u/Docs".to_string(),
                dir: true,
            };
            assert_eq!(s.children(&sub).len(), 4);
            assert_eq!(s.menu_row(std::slice::from_ref(&sub), 0, false), LauncherRow::Listed);
            assert_eq!(
                s.node_for(&sub),
                Some(NodeKind::Directory {
                    path: "~/Docs".to_string(),
                    name: None,
                    icon: None
                })
            );
            let file = Entry::Listed {
                path: "/tmp/a.txt".to_string(),
                dir: false,
            };
            assert!(
                matches!(s.drag(&file), Some(Drag::Copy(NodeKind::File { path, .. })) if path == "/tmp/a.txt")
            );
            let open = Entry::OpenDir("/tmp".to_string());
            assert_eq!(s.menu_row(&[open.clone(), Entry::Gap], 0, false), LauncherRow::Fixed);
            assert_eq!(s.menu_row(&[open.clone(), Entry::Gap], 1, false), LauncherRow::Fixed);
            assert_eq!(s.drag(&open), None);
            assert_eq!(s.drag(&Entry::Gap), None);
        });
    }

    #[test]
    fn files_links_and_recent_nodes() {
        let n = nodes(
            r#"{"items": [{"type": "file", "path": "~/a%20b.pdf"},
                          {"type": "url", "url": "https://example.com/x"},
                          {"type": "url", "url": "https://example.com", "name": "Example"},
                          {"type": "recent"}]}"#,
        );
        with(&n, |s| {
            let root = s.root_entries();
            let rows: Vec<Row> = root.iter().map(|e| s.row(e)).collect();
            assert_eq!(rows, [Row::LEAF, Row::LEAF, Row::LEAF, Row::FOLDER]);
            assert_eq!(s.node_name(&[0]).as_deref(), Some("a b.pdf"));
            assert_eq!(s.node_name(&[1]).as_deref(), Some("example.com"));
            assert_eq!(s.node_name(&[2]).as_deref(), Some("Example"));
            assert_eq!(s.node_name(&[3]).as_deref(), Some("Recent Files"));
            assert_eq!(s.menu_children(&root[3]), None, "\"No recent files\" when empty");
            assert!(s.children(&root[0]).is_empty());
            let found: Vec<(Entry, String)> = s
                .searchable()
                .into_iter()
                .take(3)
                .map(|c| (c.entry, format!("{} {}", c.name, c.description)))
                .collect();
            assert_eq!(found[0].1, "a b.pdf ~/a%20b.pdf");
            assert_eq!(found[2], (Entry::Node(vec![2]), "Example https://example.com".to_string()));
        });
    }

    #[test]
    fn recent_files_of_every_app_are_the_system_history() {
        let n = tree();
        with(&n, |s| {
            let file = RecentFile {
                name: "a".into(),
                path: "/a".into(),
                timestamp: String::new(),
            };
            let row = Entry::Recent {
                opener: Opener::Default,
                file,
            };
            assert_eq!(s.menu_row(&[row], 0, false), LauncherRow::Recent { removable: true });
        });
    }

    #[test]
    fn menu_rows_by_entry() {
        let n = tree();
        with(&n, |s| {
            let root = s.root_entries();
            assert_eq!(
                s.menu_row(&root, 2, false),
                LauncherRow::Node {
                    separator: true,
                    index: 2,
                    len: 3,
                    searching: false
                }
            );
            assert_eq!(s.menu_row(&root, 3, false), LauncherRow::Empty);
            assert_eq!(s.menu_row(&[Entry::App(0)], 0, true), LauncherRow::Generated);
            let recent = |opener| Entry::Recent {
                opener,
                file: RecentFile {
                    name: "a".into(),
                    path: "/a".into(),
                    timestamp: String::new(),
                },
            };
            assert_eq!(
                s.menu_row(&[recent(Opener::App(1))], 0, false),
                LauncherRow::Recent { removable: true }
            );
            assert_eq!(
                s.menu_row(&[recent(Opener::Command(vec![0]))], 0, false),
                LauncherRow::Recent { removable: false },
                "VS Code history is not the system history"
            );
        });
    }

    #[test]
    fn new_nodes_go_before_the_row_or_to_the_end_of_the_level() {
        let n = tree();
        let id = |path: &[usize]| node_at(&n, path).unwrap().id.clone();
        with(&n, |s| {
            let dev = Entry::Node(vec![1]);
            let place = |parent: Option<String>, index| Some(Place { parent, index });
            assert_eq!(s.new_place(&[], Some(&Entry::Node(vec![2]))), place(None, 2));
            assert_eq!(s.new_place(&[], None), place(None, 3));
            assert_eq!(
                s.new_place(&[&dev], Some(&Entry::Node(vec![1, 1]))),
                place(Some(id(&[1])), 1)
            );
            assert_eq!(s.new_place(&[&dev], None), place(Some(id(&[1])), 3));
            let category = Entry::Category(Category::Game);
            assert_eq!(s.new_place(&[&Entry::Node(vec![0]), &category], None), None);
        });
    }

    #[test]
    fn added_rows_go_before_the_node_that_generated_them() {
        let n = tree();
        with(&n, |s| {
            let category = Entry::Category(Category::Game);
            let tools = Entry::Node(vec![1, 2]);
            let place = s.add_place(&[&Entry::Node(vec![1]), &tools, &category]);
            assert_eq!(place.parent, Some(node_at(&n, &[1]).unwrap().id.clone()));
            assert_eq!(place.index, 2);
            assert_eq!(
                s.add_place(&[]),
                Place {
                    parent: None,
                    index: 3
                }
            );
        });
    }

    #[test]
    fn drops_land_between_rows_into_folders_or_at_the_end() {
        let n = tree();
        let id = |path: &[usize]| node_at(&n, path).unwrap().id.clone();
        with(&n, |s| {
            let root = s.root_entries();
            let place = |parent: Option<String>, index| Some(Place { parent, index });
            assert_eq!(s.drop_place(&[], &root, Spot::Before(2)), place(None, 2));
            assert_eq!(s.drop_place(&[], &root, Spot::End), place(None, 3));
            assert_eq!(s.drop_place(&[], &root, Spot::Into(1)), place(Some(id(&[1])), 3));
            assert_eq!(s.drop_place(&[], &root, Spot::Into(0)), None, "not a folder");
            let dev = Entry::Node(vec![1]);
            let level = s.children(&dev);
            assert_eq!(s.drop_place(&[&dev], &level, Spot::Before(1)), place(Some(id(&[1])), 1));
            let category = Entry::Category(Category::Development);
            let apps = s.children(&category);
            assert_eq!(s.drop_place(&[&category], &apps, Spot::Before(0)), None);
            assert_eq!(s.drop_place(&[&category], &apps, Spot::End), None, "generated level");
        });
    }

    #[test]
    fn nodes_move_and_generated_rows_copy() {
        let n = tree();
        with(&n, |s| {
            assert_eq!(s.drag(&Entry::Node(vec![1])), Some(Drag::Move(n[1].id.clone())));
            assert!(matches!(s.drag(&Entry::App(1)), Some(Drag::Copy(NodeKind::App { .. }))));
            let recent = Entry::Recent {
                opener: Opener::App(1),
                file: RecentFile {
                    name: "a".into(),
                    path: "/a".into(),
                    timestamp: String::new(),
                },
            };
            assert_eq!(s.drag(&recent), None);
            assert!(s.is_folder(&Entry::Node(vec![1])));
            assert!(!s.is_folder(&Entry::Node(vec![0])));
            assert!(!s.is_folder(&Entry::Category(Category::Game)));
        });
    }

    #[test]
    fn generated_rows_become_nodes() {
        let n = tree();
        with(&n, |s| {
            assert!(matches!(
                s.node_for(&Entry::App(1)),
                Some(NodeKind::App { desktop_id, name: None, icon: None }) if desktop_id == "editor.desktop"
            ));
            assert_eq!(
                s.node_for(&Entry::Power(PowerAction::Lock)),
                Some(NodeKind::Power {
                    name: None,
                    action: Some(PowerAction::Lock)
                })
            );
            assert!(s.node_for(&Entry::Node(vec![0])).is_none());
        });
    }

    #[test]
    fn paths_walk_folders() {
        let n = tree();
        assert_eq!(command_at(&n, &[1, 2, 0]).map(|c| c.name.as_str()), Some("Calc"));
        assert!(node_at(&n, &[1, 9]).is_none());
        assert!(node_at(&n, &[0, 0]).is_none(), "commands have no children");
        assert!(command_at(&n, &[1]).is_none());
    }

    #[test]
    fn rows_follow_node_types() {
        let n = tree();
        with(&n, |s| {
            let rows: Vec<Row> = s.root_entries().iter().map(|e| s.row(e)).collect();
            assert_eq!(rows, [Row::BRANCH_LEAF, Row::FOLDER, Row::SEPARATOR]);
            assert_eq!(s.row(&Entry::App(0)), Row::LEAF);
            assert_eq!(s.row(&Entry::App(1)), Row::BRANCH_LEAF, "Editor has recent files");
            assert_eq!(s.row(&Entry::Node(vec![1, 2, 1])), Row::BRANCH_LEAF);
            assert_eq!(s.row(&Entry::Category(Category::Game)), Row::FOLDER);
            assert_eq!(s.row(&Entry::Power(PowerAction::Lock)), Row::LEAF);
        });
    }

    #[test]
    fn folder_children_are_paths_below_it() {
        let n = tree();
        with(&n, |s| {
            assert_eq!(
                s.children(&Entry::Node(vec![1])),
                [Entry::Node(vec![1, 0]), Entry::Node(vec![1, 1]), Entry::Node(vec![1, 2])]
            );
            assert_eq!(s.menu_children(&Entry::Node(vec![0])), None);
            assert!(s.children(&Entry::Node(vec![0])).is_empty(), "unreadable history");
            assert!(s.children(&Entry::App(0)).is_empty(), "no recent files");
        });
    }

    #[test]
    fn generated_menus_list_categories_apps_and_power() {
        let n = nodes(r#"{"items": [{"type": "apps"}, {"type": "power"}]}"#);
        with(&n, |s| {
            let root = s.root_entries();
            assert_eq!(root, [Entry::Node(vec![0]), Entry::Node(vec![1])]);
            assert_eq!(s.row(&root[0]), Row::FOLDER);
            assert_eq!(
                s.children(&root[0]),
                [
                    Entry::Category(Category::Game),
                    Entry::Category(Category::Network),
                    Entry::Category(Category::Development)
                ]
            );
            assert_eq!(s.children(&Entry::Category(Category::Network)), [Entry::App(0)]);
            assert_eq!(s.children(&root[1]).len(), 5);
            assert_eq!(s.node_name(&[0]).as_deref(), Some("Applications"));
            assert_eq!(s.node_name(&[1]).as_deref(), Some("Power"));
        });
    }

    #[test]
    fn category_and_action_nodes_stand_alone() {
        let n = nodes(
            r#"{"items": [{"type": "apps", "category": "Development"},
                          {"type": "power", "action": "lock", "name": "Lock Screen"},
                          {"type": "power", "action": "powerOff"}]}"#,
        );
        with(&n, |s| {
            let root = s.root_entries();
            assert_eq!(root, [Entry::Node(vec![0]), Entry::Node(vec![1]), Entry::Node(vec![2])]);
            assert_eq!(s.row(&root[0]), Row::FOLDER);
            assert_eq!(s.children(&root[0]), [Entry::App(1)]);
            assert_eq!(s.node_name(&[0]).as_deref(), Some("Programming"));
            assert_eq!(s.row(&root[1]), Row::LEAF);
            assert_eq!(s.menu_children(&root[1]), None);
            assert_eq!(s.node_name(&[1]).as_deref(), Some("Lock Screen"));
            assert_eq!(s.node_name(&[2]).as_deref(), Some("Power Off"));
            assert_eq!(s.power_at(&[2]), Some(PowerAction::PowerOff));
            assert_eq!(s.power_at(&[0]), None);
        });
    }

    #[test]
    fn app_nodes_of_missing_apps_are_left_out() {
        let n = nodes(
            r#"{"items": [{"type": "app", "desktopId": "gone.desktop"},
                          {"type": "app", "desktopId": "chess.desktop"}]}"#,
        );
        with(&n, |s| {
            assert_eq!(s.root_entries(), [Entry::Node(vec![1])]);
            assert_eq!(s.node_name(&[1]).as_deref(), Some("Chess"));
            assert_eq!(s.row(&Entry::Node(vec![1])), Row::LEAF);
        });
    }

    #[test]
    fn breadcrumbs_name_every_folder() {
        let n = tree();
        with(&n, |s| {
            assert_eq!(s.breadcrumb(&[1, 2, 0]), "Dev › Tools › Calc");
            assert_eq!(s.breadcrumb(&[1, 2, 1]), "Dev › Tools › Editor");
        });
    }

    #[test]
    fn search_lists_the_tree_then_apps_not_in_it() {
        let n = tree();
        with(&n, |s| {
            let found: Vec<(Entry, String)> =
                s.searchable().into_iter().map(|c| (c.entry, c.name.into_owned())).collect();
            assert_eq!(
                found,
                [
                    (Entry::Node(vec![0]), "Files".to_string()),
                    (Entry::Node(vec![1, 0]), "Code".to_string()),
                    (Entry::Node(vec![1, 2, 0]), "Calc".to_string()),
                    (Entry::Node(vec![1, 2, 1]), "Editor".to_string()),
                    (Entry::App(0), "Firefox".to_string()),
                    (Entry::App(2), "Chess".to_string()),
                ]
            );
        });
    }
}
