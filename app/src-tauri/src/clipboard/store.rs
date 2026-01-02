//! Clipboard history storage with LRU eviction
//!
//! Manages clipboard history persistence to JSON file with configurable
//! maximum item limit. Uses LRU (Least Recently Used) strategy for eviction.

use super::ClipboardItem;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardStore {
    /// List of clipboard items, sorted by timestamp (newest first)
    pub items: Vec<ClipboardItem>,
    /// Maximum number of items to keep
    pub max_items: usize,
}

impl ClipboardStore {
    /// Creates a new clipboard store with default settings
    ///
    /// Default max_items is 50.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            max_items: 50,
        }
    }

    /// Creates a new clipboard store with a custom item limit
    ///
    /// # Arguments
    ///
    /// * `max_items` - Maximum number of items to store
    #[allow(dead_code)]
    pub fn new_with_limit(max_items: usize) -> Self {
        Self {
            items: Vec::new(),
            max_items,
        }
    }

    /// Loads clipboard store from file
    ///
    /// Returns a new empty store if file doesn't exist or is invalid.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON storage file
    pub fn load(path: &PathBuf) -> Self {
        if !path.exists() {
            return Self::new();
        }

        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Self::new()),
            Err(_) => Self::new(),
        }
    }

    /// Saves clipboard store to file
    ///
    /// Creates parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON storage file
    ///
    /// # Errors
    ///
    /// Returns error if file write fails
    pub fn save(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Adds a new item to the clipboard history
    ///
    /// If the item already exists (same text), updates its timestamp.
    /// Enforces max_items limit by removing oldest items.
    ///
    /// # Arguments
    ///
    /// * `text` - The clipboard text content
    pub fn add(&mut self, text: String) {
        // Remove existing item with same text if present
        self.items.retain(|item| item.text != text);

        // Add new item at the beginning (most recent)
        self.items.insert(0, ClipboardItem::new(text));

        // Enforce max_items limit
        if self.items.len() > self.max_items {
            self.items.truncate(self.max_items);
        }
    }

    /// Gets the storage file path
    ///
    /// Uses XDG Base Directory specification: ~/.config/uti/clipboard.json
    pub fn get_storage_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("uti").join("clipboard.json")
    }
}

impl Default for ClipboardStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_new_creates_empty_store() {
        let store = ClipboardStore::new();
        assert_eq!(store.items.len(), 0);
        assert_eq!(store.max_items, 50);
    }

    #[test]
    fn test_new_with_limit_sets_custom_limit() {
        let store = ClipboardStore::new_with_limit(5);
        assert_eq!(store.max_items, 5);
    }

    #[test]
    fn test_add_inserts_item_at_beginning() {
        let mut store = ClipboardStore::new();
        store.add("first".to_string());
        store.add("second".to_string());

        assert_eq!(store.items.len(), 2);
        assert_eq!(store.items[0].text, "second");
        assert_eq!(store.items[1].text, "first");
    }

    #[test]
    fn test_add_removes_duplicate_and_updates_timestamp() {
        let mut store = ClipboardStore::new();
        store.add("item1".to_string());
        std::thread::sleep(std::time::Duration::from_secs(1));
        store.add("item2".to_string());

        let first_timestamp = store.items[1].timestamp;

        std::thread::sleep(std::time::Duration::from_secs(1));
        store.add("item1".to_string());

        // Should still have 2 items
        assert_eq!(store.items.len(), 2);
        // item1 should be first now
        assert_eq!(store.items[0].text, "item1");
        // Timestamp should be updated (newer)
        assert!(store.items[0].timestamp > first_timestamp);
    }

    #[test]
    fn test_add_enforces_max_items_limit() {
        let mut store = ClipboardStore::new_with_limit(3);
        store.add("item1".to_string());
        store.add("item2".to_string());
        store.add("item3".to_string());
        store.add("item4".to_string());

        assert_eq!(store.items.len(), 3);
        // Most recent items should be kept
        assert_eq!(store.items[0].text, "item4");
        assert_eq!(store.items[1].text, "item3");
        assert_eq!(store.items[2].text, "item2");
        // item1 should be evicted
        assert!(!store.items.iter().any(|i| i.text == "item1"));
    }

    #[test]
    fn test_load_returns_empty_store_when_file_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.json");

        let store = ClipboardStore::load(&path);
        assert_eq!(store.items.len(), 0);
        assert_eq!(store.max_items, 50);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_clipboard.json");

        let mut store = ClipboardStore::new_with_limit(5);
        store.add("test1".to_string());
        store.add("test2".to_string());

        store.save(&path).unwrap();

        let loaded = ClipboardStore::load(&path);
        assert_eq!(loaded.items.len(), 2);
        assert_eq!(loaded.max_items, 5);
        assert_eq!(loaded.items[0].text, "test2");
        assert_eq!(loaded.items[1].text, "test1");
    }

    #[test]
    fn test_save_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("clipboard.json");

        let store = ClipboardStore::new();
        store.save(&path).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn test_load_returns_empty_store_on_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalid.json");

        fs::write(&path, "invalid json content").unwrap();

        let store = ClipboardStore::load(&path);
        assert_eq!(store.items.len(), 0);
    }
}
