//! Clipboard management module
//!
//! Provides clipboard history management with LRU eviction
//! and JSON-based persistence.

mod commands;
mod store;

pub use commands::{add_clipboard_item, get_clipboard_history, paste_item, remove_clipboard_item};
pub use store::ClipboardStore;

use serde::{Deserialize, Serialize};

/// Clipboard item data structure
///
/// Represents a single clipboard history entry with its text content
/// and timestamp for LRU ordering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipboardItem {
    /// The text content of the clipboard item
    pub text: String,
    /// Unix timestamp when this item was copied
    pub timestamp: u64,
}

impl ClipboardItem {
    /// Creates a new clipboard item with the given text and current timestamp
    ///
    /// # Arguments
    ///
    /// * `text` - The clipboard text content
    ///
    /// # Examples
    ///
    /// ```
    /// use uti_lib::clipboard::ClipboardItem;
    ///
    /// let item = ClipboardItem::new("Hello".to_string());
    /// assert_eq!(item.text, "Hello");
    /// ```
    pub fn new(text: String) -> Self {
        Self {
            text,
            timestamp: Self::current_timestamp(),
        }
    }

    /// Creates a new clipboard item with a specific timestamp
    ///
    /// # Arguments
    ///
    /// * `text` - The clipboard text content
    /// * `timestamp` - Unix timestamp
    #[allow(dead_code)]
    pub fn with_timestamp(text: String, timestamp: u64) -> Self {
        Self { text, timestamp }
    }

    /// Gets the current Unix timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_item_with_text() {
        let item = ClipboardItem::new("test text".to_string());
        assert_eq!(item.text, "test text");
        assert!(item.timestamp > 0);
    }

    #[test]
    fn test_with_timestamp_creates_item_with_specific_timestamp() {
        let item = ClipboardItem::with_timestamp("test".to_string(), 1234567890);
        assert_eq!(item.text, "test");
        assert_eq!(item.timestamp, 1234567890);
    }

    #[test]
    fn test_items_are_equal_if_text_and_timestamp_match() {
        let item1 = ClipboardItem::with_timestamp("test".to_string(), 123);
        let item2 = ClipboardItem::with_timestamp("test".to_string(), 123);
        assert_eq!(item1, item2);
    }
}
