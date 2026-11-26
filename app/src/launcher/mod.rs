//! Launcher module
//!
//! Provides launcher configuration and recent files functionality.

pub mod entry;
pub mod recent_files;
mod store;

pub use store::{get_launcher_config_path, load_launcher_tree};

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
        #[serde(rename = "appName", default)]
        app_name: Option<String>,
        /// Custom path to XBEL file (default: ~/.local/share/recently-used.xbel)
        #[serde(default)]
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
}

/// One node of the launcher tree (`items` in `launcher.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LauncherNode {
    /// Stable id for editing; generated at load when missing (not saved).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(flatten)]
    pub kind: NodeKind,
}

/// Node types, by their `type` field. Later types (`app`, `apps`,
/// `directory`, `power`, `file`, `url`, `recent`, `snippets`) are added
/// here; see ADR-037.
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
    /// A type this version does not know; dropped at load.
    #[serde(other)]
    Unknown,
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
}

impl LauncherConfig {
    /// The tree to show: `items`, or else the old `commands`. Unknown node
    /// types are dropped and missing ids generated.
    pub fn into_tree(self) -> Vec<LauncherNode> {
        let mut nodes = match self.items {
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

fn normalize(nodes: &mut Vec<LauncherNode>) {
    nodes.retain(|n| {
        let known = n.kind != NodeKind::Unknown;
        if !known {
            eprintln!("launcher.json: skipping an item of unknown type");
        }
        known
    });
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
    fn unknown_types_are_dropped_at_any_depth() {
        let nodes = tree(
            r#"{"items": [{"type": "power"},
                          {"type": "folder", "name": "A", "children": [{"type": "apps", "inline": true}]}]}"#,
        );
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0].kind, NodeKind::Folder { children, .. } if children.is_empty()));
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
        assert!(tree("{}").is_empty());
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
