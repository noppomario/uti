//! Launcher module
//!
//! Provides launcher configuration and recent files functionality.

pub mod apps;
pub mod edit;
pub mod entry;
pub mod form;
pub mod palette;
pub mod power;
pub mod recent_files;
mod store;

pub use store::{get_launcher_config_path, LauncherFile};

use apps::Category;
use power::PowerAction;
use serde::{Deserialize, Serialize};

/// History source for retrieving recent files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum HistorySource {
    /// Use recently-used.xbel
    /// - If path is specified: read from that file (appName optional for filtering)
    /// - If path is not specified: read from system XBEL (appName required)
    RecentlyUsed {
        /// Application name to filter by (optional if path is specified)
        #[serde(rename = "appName", default, skip_serializing_if = "Option::is_none")]
        app_name: Option<String>,
        /// Custom path to XBEL file (default: ~/.local/share/recently-used.xbel)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    /// Use VSCode history from specified path
    Vscode { path: String },
}

impl HistorySource {
    /// Recent files for this source, newest first.
    ///
    /// The system XBEL needs `appName` to filter by; without it nothing is
    /// returned.
    pub fn recent_files(&self) -> Vec<recent_files::RecentFile> {
        match self {
            Self::RecentlyUsed {
                app_name: None,
                path: None,
            } => Vec::new(),
            Self::RecentlyUsed { app_name, path } => {
                recent_files::get_recent_files_from_xbel(app_name.as_deref(), path.as_deref())
            },
            Self::Vscode { path } => recent_files::get_recent_files_from_vscode(path),
        }
    }
}

/// A command to start: `command args..` (plus a recent file when opened
/// from its children).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommandItem {
    /// Display name
    pub name: String,
    /// Command to execute
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// Recent files shown as the command's children.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_source: Option<HistorySource>,
    /// Icon name or file path; by default the icon of the installed app
    /// with the same command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// One node of the launcher tree (`items` in `launcher.json`).
#[derive(Debug, Clone, PartialEq)]
pub struct LauncherNode {
    /// Stable id for editing; generated at load when missing and saved
    /// with the next edit.
    pub id: String,
    pub kind: NodeKind,
}

impl LauncherNode {
    /// A node of `kind` with a new id.
    pub fn new(kind: NodeKind) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            kind,
        }
    }
}

#[derive(Serialize)]
struct KnownNode<'a> {
    id: &'a str,
    #[serde(flatten)]
    kind: &'a NodeKind,
}

impl Serialize for LauncherNode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.kind {
            NodeKind::Raw(value) => value.serialize(serializer),
            kind => KnownNode { id: &self.id, kind }.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for LauncherNode {
    /// A node this version cannot read (unknown `type`, invalid fields)
    /// becomes [`NodeKind::Raw`] instead of failing the whole file.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let id = value.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_owned();
        let kind = NodeKind::deserialize(&value).unwrap_or_else(|e| {
            eprintln!("launcher.json: keeping an item it cannot show ({e})");
            NodeKind::Raw(value)
        });
        Ok(Self { id, kind })
    }
}

/// Node types, by their `type` field. Later types (`directory`, `file`,
/// `url`, `recent`, `snippets`) are added here; see ADR-037.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum NodeKind {
    Folder {
        name: String,
        #[serde(default)]
        children: Vec<LauncherNode>,
    },
    Command(CommandItem),
    Separator,
    /// Installed apps: a menu of every main category, or the apps of one.
    Apps {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        category: Option<Category>,
    },
    /// One installed app, optionally renamed or with another icon.
    App {
        #[serde(rename = "desktopId")]
        desktop_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon: Option<String>,
    },
    /// One session or power action, or a menu of all of them.
    Power {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action: Option<PowerAction>,
    },
    /// An item this version cannot read, kept as written (not shown).
    #[serde(skip)]
    Raw(serde_json::Value),
}

impl NodeKind {
    /// Shown as a row (not [`NodeKind::Raw`]).
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Raw(_))
    }
}

/// Entry of the pre-tree `commands` array.
#[derive(Debug, Clone, Deserialize)]
pub struct LegacyCommand {
    #[serde(default)]
    pub id: String,
    #[serde(flatten)]
    pub item: CommandItem,
}

/// Launcher configuration file structure
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LauncherConfig {
    #[serde(default)]
    pub items: Option<Vec<LauncherNode>>,
    /// Read as `command` items when `items` is absent.
    #[serde(default)]
    pub commands: Vec<LegacyCommand>,
    /// Other top-level keys, kept when saving.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl LauncherConfig {
    /// The tree to show: `items`, or else the old `commands`, or else
    /// [`default_tree`]. Missing ids are generated.
    pub fn into_tree(self) -> Vec<LauncherNode> {
        let mut nodes = match self.items {
            None if self.commands.is_empty() => default_tree(),
            Some(items) => {
                if !self.commands.is_empty() {
                    eprintln!("launcher.json: \"commands\" is ignored because \"items\" is set");
                }
                items
            },
            None => self
                .commands
                .into_iter()
                .map(|c| LauncherNode {
                    id: c.id,
                    kind: NodeKind::Command(c.item),
                })
                .collect(),
        };
        normalize(&mut nodes);
        nodes
    }
}

/// The tree without a configuration: installed apps, then every power
/// action as its own node.
pub fn default_tree() -> Vec<LauncherNode> {
    let apps = NodeKind::Apps {
        name: None,
        category: None,
    };
    let power = PowerAction::ALL.map(|a| NodeKind::Power {
        name: None,
        action: Some(a),
    });
    [apps, NodeKind::Separator]
        .into_iter()
        .chain(power)
        .map(|kind| LauncherNode {
            id: String::new(),
            kind,
        })
        .collect()
}

fn normalize(nodes: &mut [LauncherNode]) {
    for node in nodes {
        if node.id.is_empty() {
            node.id = uuid::Uuid::new_v4().to_string();
        }
        if let NodeKind::Folder { children, .. } = &mut node.kind {
            normalize(children);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(json: &str) -> Vec<LauncherNode> {
        serde_json::from_str::<LauncherConfig>(json).unwrap().into_tree()
    }

    fn command(node: &LauncherNode) -> &CommandItem {
        match &node.kind {
            NodeKind::Command(c) => c,
            other => panic!("expected a command, got {other:?}"),
        }
    }

    #[test]
    fn items_are_a_tree_of_typed_nodes() {
        let nodes = tree(
            r#"{"items": [
                {"type": "folder", "id": "dev", "name": "Dev", "children": [
                    {"type": "command", "name": "Code", "command": "code", "args": ["-n"]}
                ]},
                {"type": "separator"},
                {"type": "command", "name": "Files", "command": "nautilus",
                 "historySource": {"type": "recently-used", "appName": "org.gnome.Nautilus"}}
            ]}"#,
        );
        assert_eq!(nodes.len(), 3);
        let NodeKind::Folder { name, children } = &nodes[0].kind else {
            panic!("expected a folder");
        };
        assert_eq!((nodes[0].id.as_str(), name.as_str()), ("dev", "Dev"));
        assert_eq!(command(&children[0]).args, ["-n"]);
        assert_eq!(nodes[1].kind, NodeKind::Separator);
        assert!(matches!(
            command(&nodes[2]).history_source,
            Some(HistorySource::RecentlyUsed { app_name: Some(ref a), path: None }) if a == "org.gnome.Nautilus"
        ));
    }

    #[test]
    fn missing_ids_are_generated_and_unique() {
        let nodes = tree(
            r#"{"items": [{"type": "folder", "name": "A", "children": [{"type": "separator"}]},
                          {"type": "separator"}]}"#,
        );
        let NodeKind::Folder { children, .. } = &nodes[0].kind else {
            panic!("expected a folder");
        };
        let ids = [&nodes[0].id, &nodes[1].id, &children[0].id];
        assert!(ids.iter().all(|id| !id.is_empty()));
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[0], ids[2]);
    }

    #[test]
    fn unreadable_items_are_kept_as_written_at_any_depth() {
        let json = r#"{"items": [{"type": "directory", "id": "d", "path": "~"},
                          {"type": "folder", "name": "A", "children": [
                              {"type": "snippets"}, {"type": "command", "name": "no command"}]}]}"#;
        let nodes = tree(json);
        assert_eq!(nodes.len(), 2);
        assert!(!nodes[0].kind.is_known());
        assert_eq!(nodes[0].id, "d");
        let NodeKind::Folder { children, .. } = &nodes[1].kind else {
            panic!("expected a folder");
        };
        assert!(children.iter().all(|c| !c.kind.is_known()));
        let saved = serde_json::to_value(&nodes).unwrap();
        assert_eq!(saved[0], serde_json::json!({"type": "directory", "id": "d", "path": "~"}));
        assert_eq!(
            saved[1]["children"][1],
            serde_json::json!({"type": "command", "name": "no command"})
        );
    }

    #[test]
    fn generated_ids_are_saved() {
        let nodes = tree(r#"{"items": [{"type": "separator"}]}"#);
        let json = serde_json::to_value(&nodes[0]).unwrap();
        assert_eq!(json["id"], nodes[0].id.as_str());
        assert_eq!(serde_json::from_value::<LauncherNode>(json).unwrap(), nodes[0]);
    }

    #[test]
    fn old_commands_are_read_as_command_items() {
        let nodes = tree(
            r#"{"commands": [
                {"id": "nautilus", "name": "Files", "command": "nautilus",
                 "historySource": {"type": "vscode", "path": "~/x.vscdb"}},
                {"name": "Terminal", "command": "ptyxis", "args": ["--new-window"]}
            ]}"#,
        );
        assert_eq!(nodes[0].id, "nautilus");
        assert!(matches!(command(&nodes[0]).history_source, Some(HistorySource::Vscode { .. })));
        assert_eq!(command(&nodes[1]).args, ["--new-window"]);
        assert!(!nodes[1].id.is_empty());
    }

    #[test]
    fn items_win_over_commands() {
        let nodes = tree(
            r#"{"items": [{"type": "separator"}], "commands": [{"name": "A", "command": "a"}]}"#,
        );
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].kind, NodeKind::Separator);
        assert!(tree(r#"{"items": []}"#).is_empty(), "an empty tree is kept");
    }

    #[test]
    fn generated_nodes_read_their_options() {
        let nodes = tree(
            r#"{"items": [{"type": "apps", "name": "Programs"},
                          {"type": "apps", "category": "AudioVideo"},
                          {"type": "power", "action": "logOut"},
                          {"type": "app", "desktopId": "org.gnome.Nautilus.desktop", "name": "Files"}]}"#,
        );
        let kinds: Vec<&NodeKind> = nodes.iter().map(|n| &n.kind).collect();
        assert_eq!(
            kinds,
            [
                &NodeKind::Apps {
                    name: Some("Programs".to_string()),
                    category: None
                },
                &NodeKind::Apps {
                    name: None,
                    category: Some(Category::AudioVideo)
                },
                &NodeKind::Power {
                    name: None,
                    action: Some(PowerAction::LogOut)
                },
                &NodeKind::App {
                    desktop_id: "org.gnome.Nautilus.desktop".to_string(),
                    name: Some("Files".to_string()),
                    icon: None,
                },
            ]
        );
        let json = serde_json::to_value(&nodes[2]).unwrap();
        assert_eq!(json["type"], "power");
        assert_eq!(json["action"], "logOut");
        assert!(json.get("name").is_none());
        let json = serde_json::to_value(&nodes[0]).unwrap();
        assert!(json.get("category").is_none());
    }

    #[test]
    fn without_items_or_commands_the_default_tree_is_shown() {
        let kinds: Vec<NodeKind> = tree("{}").into_iter().map(|n| n.kind).collect();
        let power = PowerAction::ALL.map(|a| NodeKind::Power {
            name: None,
            action: Some(a),
        });
        assert_eq!(
            kinds[..2],
            [
                NodeKind::Apps {
                    name: None,
                    category: None
                },
                NodeKind::Separator
            ]
        );
        assert_eq!(kinds[2..], power);
        assert_eq!(tree(r#"{"commands": []}"#).len(), 7);
    }

    #[test]
    fn nodes_serialize_with_their_type() {
        let node = LauncherNode {
            id: "t".to_string(),
            kind: NodeKind::Command(CommandItem {
                name: "Test".to_string(),
                command: "test-cmd".to_string(),
                args: vec![],
                history_source: None,
                icon: None,
            }),
        };
        let json = serde_json::to_value(&node).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"id": "t", "type": "command", "name": "Test", "command": "test-cmd", "args": []})
        );
        assert_eq!(serde_json::from_value::<LauncherNode>(json).unwrap(), node);
    }

    #[test]
    fn per_app_xbel_needs_no_app_name() {
        let nodes = tree(
            r#"{"items": [{"type": "command", "name": "Text Editor", "command": "gnome-text-editor",
                "historySource": {"type": "recently-used", "path": "~/.local/share/x.xbel"}}]}"#,
        );
        assert_eq!(
            command(&nodes[0]).history_source,
            Some(HistorySource::RecentlyUsed {
                app_name: None,
                path: Some("~/.local/share/x.xbel".to_string()),
            })
        );
    }

    #[test]
    fn system_xbel_without_app_name_returns_nothing() {
        let source = HistorySource::RecentlyUsed {
            app_name: None,
            path: None,
        };
        assert!(source.recent_files().is_empty());
    }
}
