//! Launcher configuration storage
//!
//! Loads `launcher.json` and saves edits to it (ADR-038): atomically, with
//! every id, keeping items and keys this version cannot read, and never
//! while the file fails to parse.

use super::{LauncherConfig, LauncherNode};
use serde_json::{Map, Value};
use std::io;
use std::path::{Path, PathBuf};

/// Get the path to the launcher configuration file
///
/// Returns `~/.config/uti/launcher.json`
pub fn get_launcher_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("uti")
        .join("launcher.json")
}

/// The launcher tree and what saving it needs.
#[derive(Debug, Clone)]
pub struct LauncherFile {
    pub nodes: Vec<LauncherNode>,
    /// Why the file could not be read; editing is off while set.
    pub error: Option<String>,
    /// Top-level keys other than `items` and `commands`.
    extra: Map<String, Value>,
    /// The file as last read or written; `None` when it did not exist.
    on_disk: Option<String>,
}

impl LauncherFile {
    /// Reads `path`. A missing file gives the default tree; an invalid one
    /// too, with [`LauncherFile::error`] set.
    pub fn load(path: &Path) -> Self {
        let contents = std::fs::read_to_string(path).ok();
        let parsed = match &contents {
            None => Ok(LauncherConfig::default()),
            Some(text) => serde_json::from_str::<LauncherConfig>(text).map_err(|e| e.to_string()),
        };
        let (mut config, error) = match parsed {
            Ok(config) => (config, None),
            Err(e) => {
                eprintln!("Failed to parse {}: {e}", path.display());
                (LauncherConfig::default(), Some(e))
            },
        };
        let extra = std::mem::take(&mut config.extra);
        Self {
            nodes: config.into_tree(),
            error,
            extra,
            on_disk: contents,
        }
    }

    /// Reads `path` again if it changed since it was read or written;
    /// `true` when it did.
    pub fn reload_if_changed(&mut self, path: &Path) -> bool {
        if std::fs::read_to_string(path).ok() == self.on_disk {
            return false;
        }
        *self = Self::load(path);
        true
    }

    pub fn editable(&self) -> bool {
        self.error.is_none()
    }

    /// Applies `edit` to the tree and saves it; `Ok(false)` when nothing
    /// changed. When the file changed on disk since it was read, the edit
    /// is applied to the file as it is now (edits find nodes by id).
    pub fn edit(
        &mut self,
        path: &Path,
        edit: impl FnOnce(&mut Vec<LauncherNode>) -> bool,
    ) -> Result<bool, EditError> {
        self.reload_if_changed(path);
        if let Some(e) = &self.error {
            return Err(EditError::Invalid(e.clone()));
        }
        if !edit(&mut self.nodes) {
            return Ok(false);
        }
        #[derive(serde::Serialize)]
        struct Saved<'a> {
            items: &'a [LauncherNode],
            #[serde(flatten)]
            extra: &'a Map<String, Value>,
        }
        let saved = Saved {
            items: &self.nodes,
            extra: &self.extra,
        };
        let text = serde_json::to_string_pretty(&saved).map_err(io::Error::other)? + "\n";
        crate::fs::write_atomic(path, text.as_bytes())?;
        self.on_disk = Some(text);
        Ok(true)
    }
}

/// Why an edit was not saved.
#[derive(Debug)]
pub enum EditError {
    /// `launcher.json` fails to parse (the message).
    Invalid(String),
    Io(io::Error),
}

impl From<io::Error> for EditError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(e) => write!(f, "launcher.json is invalid: {e}"),
            Self::Io(e) => write!(f, "failed to save launcher.json: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::launcher::NodeKind;
    use tempfile::TempDir;

    fn file(dir: &TempDir, contents: Option<&str>) -> PathBuf {
        let path = dir.path().join("launcher.json");
        if let Some(c) = contents {
            std::fs::write(&path, c).unwrap();
        }
        path
    }

    fn saved(path: &Path) -> Value {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn push_separator(nodes: &mut Vec<LauncherNode>) -> bool {
        nodes.push(LauncherNode::new(NodeKind::Separator));
        true
    }

    #[test]
    fn test_get_launcher_config_path() {
        let path = get_launcher_config_path();
        assert!(path.ends_with("launcher.json"));
        assert!(path.to_string_lossy().contains("uti"));
    }

    #[test]
    fn a_missing_file_is_the_default_tree_and_the_first_edit_writes_it() {
        let dir = TempDir::new().unwrap();
        let path = file(&dir, None);
        let mut f = LauncherFile::load(&path);
        assert!(f.editable());
        assert_eq!(f.nodes.len(), crate::launcher::default_tree().len());
        assert!(f.edit(&path, push_separator).unwrap());
        let items = saved(&path)["items"].as_array().unwrap().clone();
        assert_eq!(items.len(), 8);
        assert!(items.iter().all(|i| i["id"].as_str().is_some_and(|id| !id.is_empty())));
    }

    #[test]
    fn commands_move_to_items_and_other_keys_stay() {
        let dir = TempDir::new().unwrap();
        let path = file(
            &dir,
            Some(r#"{"commands": [{"id": "a", "name": "A", "command": "a"}], "future": 1}"#),
        );
        let mut f = LauncherFile::load(&path);
        f.edit(&path, push_separator).unwrap();
        let json = saved(&path);
        assert!(json.get("commands").is_none());
        assert_eq!(json["future"], 1);
        assert_eq!(json["items"][0]["id"], "a");
        assert_eq!(json["items"][0]["type"], "command");
    }

    #[test]
    fn an_invalid_file_shows_the_default_tree_and_is_not_overwritten() {
        let dir = TempDir::new().unwrap();
        let path = file(&dir, Some("{ not json"));
        let mut f = LauncherFile::load(&path);
        assert!(!f.editable());
        assert!(!f.nodes.is_empty());
        assert!(matches!(f.edit(&path, push_separator), Err(EditError::Invalid(_))));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ not json");
    }

    #[test]
    fn unreadable_items_survive_an_edit() {
        let dir = TempDir::new().unwrap();
        let path = file(&dir, Some(r#"{"items": [{"type": "future", "path": "~"}]}"#));
        let mut f = LauncherFile::load(&path);
        f.edit(&path, push_separator).unwrap();
        assert_eq!(saved(&path)["items"][0], serde_json::json!({"type": "future", "path": "~"}));
    }

    #[test]
    fn edits_apply_to_the_file_as_changed_on_disk() {
        let dir = TempDir::new().unwrap();
        let path = file(&dir, Some(r#"{"items": [{"type": "separator", "id": "s"}]}"#));
        let mut f = LauncherFile::load(&path);
        std::fs::write(&path, r#"{"items": [{"type": "folder", "id": "x", "name": "X"}]}"#)
            .unwrap();
        f.edit(&path, push_separator).unwrap();
        let items = saved(&path)["items"].clone();
        assert_eq!(items[0]["id"], "x");
        assert_eq!(items.as_array().unwrap().len(), 2);

        std::fs::write(&path, "broken").unwrap();
        assert!(matches!(f.edit(&path, push_separator), Err(EditError::Invalid(_))));
        assert!(!f.editable());
        assert!(!f.reload_if_changed(&path), "unchanged since");
    }

    #[test]
    fn unchanged_edits_do_not_write() {
        let dir = TempDir::new().unwrap();
        let path = file(&dir, None);
        let mut f = LauncherFile::load(&path);
        assert!(!f.edit(&path, |_| false).unwrap());
        assert!(!path.exists());
    }
}
