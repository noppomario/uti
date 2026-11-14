//! Clipboard management module
//!
//! Provides clipboard history management with LRU eviction
//! and JSON-based persistence.

mod store;

pub use store::ClipboardStore;

use serde::{Deserialize, Serialize};

/// Clipboard item data structure
///
/// Represents a single clipboard history entry with its text content
/// and timestamp for LRU ordering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipboardItem {
    /// Stable identifier (UUID v4); generated for legacy entries without one
    #[serde(default = "new_id")]
    pub id: String,
    /// The text content of the clipboard item
    pub text: String,
    /// Unix timestamp when this item was copied
    pub timestamp: u64,
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
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
    /// use uti_core::clipboard::ClipboardItem;
    ///
    /// let item = ClipboardItem::new("Hello".to_string());
    /// assert_eq!(item.text, "Hello");
    /// ```
    pub fn new(text: String) -> Self {
        Self {
            id: new_id(),
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
        Self {
            id: new_id(),
            text,
            timestamp,
        }
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
    fn test_items_with_same_content_are_distinct_entries() {
        let item1 = ClipboardItem::with_timestamp("test".to_string(), 123);
        let item2 = ClipboardItem::with_timestamp("test".to_string(), 123);
        assert_ne!(item1.id, item2.id);
        assert_ne!(item1, item2);
    }
}
