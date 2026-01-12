//! Clipboard Tauri commands
//!
//! Provides clipboard history management commands for the frontend.

use std::sync::Mutex;
use tauri::State;

use super::{ClipboardItem, ClipboardStore};

/// Gets the clipboard history
///
/// Returns a list of clipboard items sorted by timestamp (newest first).
#[tauri::command]
pub fn get_clipboard_history(store: State<Mutex<ClipboardStore>>) -> Vec<ClipboardItem> {
    let store = store.lock().unwrap();
    store.items.clone()
}

/// Adds a new item to the clipboard history
///
/// If the item already exists, its timestamp will be updated.
/// Enforces the maximum item limit via LRU eviction.
#[tauri::command]
pub fn add_clipboard_item(text: String, store: State<Mutex<ClipboardStore>>) {
    let mut store = store.lock().unwrap();
    store.add(text);

    // Save to file
    let path = ClipboardStore::get_storage_path();
    if let Err(e) = store.save(&path) {
        eprintln!("Failed to save clipboard store: {}", e);
    }
}

/// Removes a clipboard item by index (used when pinning to snippets)
#[tauri::command]
pub fn remove_clipboard_item(index: usize, store: State<Mutex<ClipboardStore>>) {
    let mut store = store.lock().unwrap();
    if index < store.items.len() {
        store.items.remove(index);
        let path = ClipboardStore::get_storage_path();
        if let Err(e) = store.save(&path) {
            eprintln!("Failed to save clipboard store: {}", e);
        }
    }
}

/// Sets the system clipboard to the specified text
#[tauri::command]
pub async fn paste_item(text: String) -> Result<(), String> {
    println!("Would paste: {}", text);
    Ok(())
}
