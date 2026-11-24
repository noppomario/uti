//! Snippets storage
//!
//! Handles loading and saving snippets to JSON file.

use super::SnippetItem;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Snippets configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnippetsStore {
    #[serde(default)]
    pub items: Vec<SnippetItem>,
}

impl SnippetsStore {
    /// Get storage file path (~/.config/uti/snippets.json)
    pub fn get_storage_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("uti")
            .join("snippets.json")
    }

    /// Appends a snippet and returns its new id (duplicates are allowed).
    pub fn add(&mut self, value: String, label: Option<String>) -> String {
        let item = SnippetItem::new(value, label);
        let id = item.id.clone();
        self.items.push(item);
        id
    }

    /// Replaces the label and value of snippet `id`; `false` if absent.
    pub fn update(&mut self, id: &str, label: Option<String>, value: String) -> bool {
        let Some(item) = self.items.iter_mut().find(|i| i.id == id) else {
            return false;
        };
        item.label = label;
        item.value = value;
        true
    }

    /// Removes snippet `id`; `false` if absent.
    pub fn remove(&mut self, id: &str) -> bool {
        let len = self.items.len();
        self.items.retain(|i| i.id != id);
        self.items.len() != len
    }

    /// Moves snippet `id` by `delta` positions, clamped to the list;
    /// `false` if absent or already at that end.
    pub fn move_by(&mut self, id: &str, delta: isize) -> bool {
        let Some(from) = self.items.iter().position(|i| i.id == id) else {
            return false;
        };
        let to = from.saturating_add_signed(delta).min(self.items.len() - 1);
        if to == from {
            return false;
        }
        let item = self.items.remove(from);
        self.items.insert(to, item);
        true
    }
}

/// Load snippets from file
///
/// Returns empty store if file doesn't exist or is invalid.
pub fn load_snippets() -> SnippetsStore {
    let path = SnippetsStore::get_storage_path();

    match fs::read_to_string(&path) {
        Ok(contents) => match serde_json::from_str::<SnippetsStore>(&contents) {
            Ok(store) => {
                println!("Loaded {} snippets from: {:?}", store.items.len(), path);
                store
            },
            Err(e) => {
                eprintln!("Failed to parse snippets: {}", e);
                SnippetsStore::default()
            },
        },
        Err(_) => {
            println!("Snippets file not found at {:?}", path);
            SnippetsStore::default()
        },
    }
}

/// Save snippets to file
pub fn save_snippets(store: &SnippetsStore) -> Result<(), std::io::Error> {
    let path = SnippetsStore::get_storage_path();

    let json = serde_json::to_string_pretty(store)?;
    crate::fs::write_atomic(&path, json.as_bytes())?;
    println!("Saved {} snippets to: {:?}", store.items.len(), path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_storage_path() {
        let path = SnippetsStore::get_storage_path();
        assert!(path.ends_with("snippets.json"));
        assert!(path.to_string_lossy().contains("uti"));
    }

    #[test]
    fn test_default_store_is_empty() {
        let store = SnippetsStore::default();
        assert!(store.items.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("snippets.json");

        let mut store = SnippetsStore::default();
        store.items.push(SnippetItem::new("test".to_string(), Some("Test".to_string())));

        // Save
        let json = serde_json::to_string_pretty(&store).unwrap();
        std::fs::write(&path, json).unwrap();

        // Load
        let contents = std::fs::read_to_string(&path).unwrap();
        let loaded: SnippetsStore = serde_json::from_str(&contents).unwrap();

        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].value, "test");
    }

    #[test]
    fn test_deserialize_empty_store() {
        let json = r#"{"items": []}"#;
        let store: SnippetsStore = serde_json::from_str(json).unwrap();
        assert!(store.items.is_empty());
    }

    fn store(values: &[&str]) -> SnippetsStore {
        let mut s = SnippetsStore::default();
        for v in values {
            s.add((*v).to_string(), None);
        }
        s
    }

    fn values(s: &SnippetsStore) -> Vec<&str> {
        s.items.iter().map(|i| i.value.as_str()).collect()
    }

    #[test]
    fn add_appends_and_allows_duplicates() {
        let mut s = store(&["a"]);
        let id = s.add("a".to_string(), Some("A".to_string()));
        assert_eq!(values(&s), ["a", "a"]);
        assert_eq!(s.items[1].id, id);
        assert_ne!(s.items[0].id, id);
        assert_eq!(s.items[1].label.as_deref(), Some("A"));
    }

    #[test]
    fn update_changes_label_and_value_by_id() {
        let mut s = store(&["a", "b"]);
        let id = s.items[1].id.clone();
        assert!(s.update(&id, Some("B".to_string()), "bb".to_string()));
        assert_eq!(values(&s), ["a", "bb"]);
        assert_eq!(s.items[1].label.as_deref(), Some("B"));
        assert!(!s.update("missing", None, "x".to_string()));
    }

    #[test]
    fn remove_by_id() {
        let mut s = store(&["a", "b"]);
        let id = s.items[0].id.clone();
        assert!(s.remove(&id));
        assert_eq!(values(&s), ["b"]);
        assert!(!s.remove(&id));
    }

    #[test]
    fn move_by_clamps_at_both_ends() {
        let mut s = store(&["a", "b", "c"]);
        let a = s.items[0].id.clone();
        assert!(s.move_by(&a, 1));
        assert_eq!(values(&s), ["b", "a", "c"]);
        assert!(s.move_by(&a, 5));
        assert_eq!(values(&s), ["b", "c", "a"]);
        assert!(!s.move_by(&a, 1));
        assert!(s.move_by(&a, -2));
        assert_eq!(values(&s), ["a", "b", "c"]);
        assert!(!s.move_by(&a, -1));
        assert!(!s.move_by("missing", 1));
    }

    #[test]
    fn test_deserialize_missing_items_defaults_to_empty() {
        let json = r#"{}"#;
        let store: SnippetsStore = serde_json::from_str(json).unwrap();
        assert!(store.items.is_empty());
    }
}
