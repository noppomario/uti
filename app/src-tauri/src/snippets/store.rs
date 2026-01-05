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
            }
            Err(e) => {
                eprintln!("Failed to parse snippets: {}", e);
                SnippetsStore::default()
            }
        },
        Err(_) => {
            println!("Snippets file not found at {:?}", path);
            SnippetsStore::default()
        }
    }
}

/// Save snippets to file
pub fn save_snippets(store: &SnippetsStore) -> Result<(), std::io::Error> {
    let path = SnippetsStore::get_storage_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(store)?;
    fs::write(&path, json)?;
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
        store.items.push(SnippetItem::new(
            "test".to_string(),
            Some("Test".to_string()),
        ));

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

    #[test]
    fn test_deserialize_missing_items_defaults_to_empty() {
        let json = r#"{}"#;
        let store: SnippetsStore = serde_json::from_str(json).unwrap();
        assert!(store.items.is_empty());
    }
}
