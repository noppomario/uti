//! Snippets Tauri commands
//!
//! Provides snippets management commands for the frontend.

use std::sync::Mutex;
use tauri::State;

use super::{save_snippets, SnippetItem, SnippetsStore};

/// Gets all snippets
#[tauri::command]
pub fn get_snippets(store: State<Mutex<SnippetsStore>>) -> Vec<SnippetItem> {
    let store = store.lock().unwrap();
    store.items.clone()
}

/// Adds a new snippet (used when pinning from clipboard)
#[tauri::command]
pub fn add_snippet(
    value: String,
    label: Option<String>,
    store: State<Mutex<SnippetsStore>>,
) -> SnippetItem {
    let mut store = store.lock().unwrap();
    let item = SnippetItem::new(value, label);
    store.items.push(item.clone());

    if let Err(e) = save_snippets(&store) {
        eprintln!("Failed to save snippets: {}", e);
    }
    item
}
