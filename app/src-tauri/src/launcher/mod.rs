//! Launcher module
//!
//! Provides launcher configuration and recent files functionality.

pub mod desktop;
pub mod recent_files;
mod store;

pub use desktop::{search_desktop_files, DesktopApp};
pub use recent_files::RecentFile;
pub use store::{get_launcher_config_path, load_launcher_config};

use serde::{Deserialize, Serialize};

/// History source for retrieving recent files
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A command entry in the launcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherItem {
    /// Unique identifier for the command
    pub id: String,
    /// Display name
    pub name: String,
    /// Command to execute
    pub command: String,
    /// Additional arguments (optional)
    #[serde(default)]
    pub args: Vec<String>,
    /// History source for jump list (optional)
    /// If not specified, will try to auto-detect from recently-used.xbel
    #[serde(default)]
    pub history_source: Option<HistorySource>,
}

/// Launcher configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LauncherConfig {
    /// List of launcher commands
    #[serde(default)]
    pub commands: Vec<LauncherItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_launcher_item_minimal() {
        let json = r#"{
            "id": "nautilus",
            "name": "Files",
            "command": "nautilus"
        }"#;

        let item: LauncherItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id, "nautilus");
        assert_eq!(item.name, "Files");
        assert_eq!(item.command, "nautilus");
        assert!(item.args.is_empty());
        assert!(item.history_source.is_none());
    }

    #[test]
    fn test_deserialize_launcher_item_with_history_source() {
        let json = r#"{
            "id": "vscode",
            "name": "Visual Studio Code",
            "command": "code",
            "historySource": {
                "type": "vscode",
                "path": "~/.config/Code/User/History"
            }
        }"#;

        let item: LauncherItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id, "vscode");
        assert!(matches!(
            item.history_source,
            Some(HistorySource::Vscode { .. })
        ));
    }

    #[test]
    fn test_deserialize_launcher_item_with_recently_used() {
        let json = r#"{
            "id": "nautilus",
            "name": "Files",
            "command": "nautilus",
            "historySource": {
                "type": "recently-used",
                "appName": "org.gnome.Nautilus"
            }
        }"#;

        let item: LauncherItem = serde_json::from_str(json).unwrap();
        assert!(matches!(
            item.history_source,
            Some(HistorySource::RecentlyUsed { app_name: Some(ref name), path: _ }) if name == "org.gnome.Nautilus"
        ));
    }

    #[test]
    fn test_deserialize_launcher_config() {
        let json = r#"{
            "commands": [
                {
                    "id": "nautilus",
                    "name": "Files",
                    "command": "nautilus"
                },
                {
                    "id": "gnome-text-editor",
                    "name": "Text Editor",
                    "command": "gnome-text-editor"
                }
            ]
        }"#;

        let config: LauncherConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.commands.len(), 2);
        assert_eq!(config.commands[0].id, "nautilus");
        assert_eq!(config.commands[1].id, "gnome-text-editor");
    }

    #[test]
    fn test_deserialize_empty_config() {
        let json = "{}";

        let config: LauncherConfig = serde_json::from_str(json).unwrap();
        assert!(config.commands.is_empty());
    }

    #[test]
    fn test_serialize_launcher_item() {
        let item = LauncherItem {
            id: "test".to_string(),
            name: "Test".to_string(),
            command: "test-cmd".to_string(),
            args: vec!["--arg1".to_string()],
            history_source: Some(HistorySource::RecentlyUsed {
                app_name: Some("org.test.App".to_string()),
                path: None,
            }),
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"id\":\"test\""));
        assert!(json.contains("\"historySource\""));
    }

    #[test]
    fn test_deserialize_launcher_item_with_custom_xbel_path() {
        let json = r#"{
            "id": "gnome-text-editor",
            "name": "Text Editor",
            "command": "gnome-text-editor",
            "historySource": {
                "type": "recently-used",
                "appName": "gnome-text-editor",
                "path": "~/.local/share/org.gnome.TextEditor/recently-used.xbel"
            }
        }"#;

        let item: LauncherItem = serde_json::from_str(json).unwrap();
        match item.history_source {
            Some(HistorySource::RecentlyUsed { app_name, path }) => {
                assert_eq!(app_name, Some("gnome-text-editor".to_string()));
                assert_eq!(
                    path,
                    Some("~/.local/share/org.gnome.TextEditor/recently-used.xbel".to_string())
                );
            }
            _ => panic!("Expected RecentlyUsed with path"),
        }
    }

    #[test]
    fn test_deserialize_launcher_item_with_path_only() {
        // Test that appName is optional when path is specified
        let json = r#"{
            "id": "gnome-text-editor",
            "name": "Text Editor",
            "command": "gnome-text-editor",
            "historySource": {
                "type": "recently-used",
                "path": "~/.local/share/org.gnome.TextEditor/recently-used.xbel"
            }
        }"#;

        let item: LauncherItem = serde_json::from_str(json).unwrap();
        match item.history_source {
            Some(HistorySource::RecentlyUsed { app_name, path }) => {
                assert_eq!(app_name, None);
                assert_eq!(
                    path,
                    Some("~/.local/share/org.gnome.TextEditor/recently-used.xbel".to_string())
                );
            }
            _ => panic!("Expected RecentlyUsed with path only"),
        }
    }
}
