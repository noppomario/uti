//! Rows of the launcher levels: tree nodes, generated rows and recent files
//! (no GTK)

use std::collections::HashSet;

use super::apps::{self, AppMeta, Category};
use super::edit::{self, Place};
use super::power::PowerAction;
use super::recent_files::{self, RecentFile};
use super::{CommandItem, HistorySource, LauncherNode, NodeKind};
use crate::state::menu::LauncherRow;
use crate::state::tree::Row;

/// Where rows come from: the tree, installed apps and the app names that
/// have recent files.
#[derive(Clone, Copy)]
pub struct Sources<'a> {
    pub nodes: &'a [LauncherNode],
    pub apps: &'a [AppMeta],
    /// Application names in `recently-used.xbel`.
    pub recent_apps: &'a HashSet<String>,
}

/// What opens a recent file.
#[derive(Debug, Clone, PartialEq)]
pub enum Opener {
    /// The command at this path.
    Command(Vec<usize>),
    /// The installed app at this index.
    App(usize),
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

impl Sources<'_> {
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
            Entry::Node(path) => {
                if let Some(app) = self.app_at(path) {
                    (Opener::App(app), self.app_recent(app))
                } else {
                    let Some(source) =
                        command_at(self.nodes, path).and_then(|c| c.history_source.as_ref())
                    else {
                        return Vec::new();
                    };
                    (Opener::Command(path.clone()), source.recent_files())
                }
            },
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
                    | NodeKind::Power { action: None, .. },
                ) => Row::FOLDER,
                Some(NodeKind::Power { .. }) => Row::LEAF,
                Some(NodeKind::Command(c)) if c.history_source.is_some() => Row::BRANCH_LEAF,
                Some(NodeKind::Command(_)) => Row::LEAF,
                Some(NodeKind::App { .. }) => {
                    self.app_at(path).map_or(Row::SEPARATOR, |a| self.app_row(a))
                },
                Some(NodeKind::Separator | NodeKind::Raw(_)) | None => Row::SEPARATOR,
            },
            Entry::Category(_) => Row::FOLDER,
            Entry::App(app) => self.app_row(*app),
            Entry::Power(_) | Entry::Recent { .. } => Row::LEAF,
        }
    }

    /// Display name of a node: folders, commands and apps by their name,
    /// `apps` and `power` by theirs or that of their category or action.
    pub fn node_name(&self, path: &[usize]) -> Option<&str> {
        Some(match &node_at(self.nodes, path)?.kind {
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
            NodeKind::App { .. } => &self.apps[self.app_at(path)?].name,
            NodeKind::Separator | NodeKind::Raw(_) => return None,
        })
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
                        name: &c.name,
                        description: &c.command,
                    }),
                    NodeKind::App { name, .. } => {
                        if let Some(a) = src.app_at(path) {
                            in_tree.insert(a);
                            out.push(Searchable {
                                entry: Entry::Node(path.clone()),
                                name: name.as_deref().unwrap_or(&src.apps[a].name),
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
                name: &a.name,
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
            Some(_) => LauncherRow::Generated,
        }
    }

    /// Recent files of `opener` come from the system `recently-used.xbel`.
    fn system_history(&self, opener: &Opener) -> bool {
        match opener {
            Opener::App(_) => true,
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
            Entry::Node(_) | Entry::Recent { .. } => return None,
        })
    }
}

fn power_entries() -> Vec<Entry> {
    PowerAction::ALL.into_iter().map(Entry::Power).collect()
}

/// A search candidate: its row and the text matched.
#[derive(Debug, PartialEq)]
pub struct Searchable<'a> {
    pub entry: Entry,
    pub name: &'a str,
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
        })
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
            assert_eq!(s.node_name(&[0]), Some("Applications"));
            assert_eq!(s.node_name(&[1]), Some("Power"));
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
            assert_eq!(s.node_name(&[0]), Some("Programming"));
            assert_eq!(s.row(&root[1]), Row::LEAF);
            assert_eq!(s.menu_children(&root[1]), None);
            assert_eq!(s.node_name(&[1]), Some("Lock Screen"));
            assert_eq!(s.node_name(&[2]), Some("Power Off"));
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
            assert_eq!(s.node_name(&[1]), Some("Chess"));
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
            let found: Vec<(Entry, &str)> =
                s.searchable().into_iter().map(|c| (c.entry, c.name)).collect();
            assert_eq!(
                found,
                [
                    (Entry::Node(vec![0]), "Files"),
                    (Entry::Node(vec![1, 0]), "Code"),
                    (Entry::Node(vec![1, 2, 0]), "Calc"),
                    (Entry::Node(vec![1, 2, 1]), "Editor"),
                    (Entry::App(0), "Firefox"),
                    (Entry::App(2), "Chess"),
                ]
            );
        });
    }
}
