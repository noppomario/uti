//! Snippets module
//!
//! Provides persistent snippet storage for quick text access.
//! Snippets are saved text entries that users can quickly copy to clipboard.

mod store;

pub use store::{load_snippets, save_snippets, SnippetsStore};

use serde::{Deserialize, Serialize};

/// A single snippet entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SnippetItem {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// Display label (optional, uses value if empty/missing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The actual text content
    pub value: String,
}

impl SnippetItem {
    /// Creates a new snippet with generated UUID
    pub fn new(value: String, label: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            label,
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_snippet_with_uuid() {
        let snippet = SnippetItem::new("test value".to_string(), None);
        assert!(!snippet.id.is_empty());
        assert_eq!(snippet.value, "test value");
        assert!(snippet.label.is_none());
    }

    #[test]
    fn test_new_with_label() {
        let snippet = SnippetItem::new("value".to_string(), Some("Label".to_string()));
        assert_eq!(snippet.label, Some("Label".to_string()));
    }

    #[test]
    fn test_deserialize_snippet() {
        let json = r#"{
            "id": "abc-123",
            "label": "My Snippet",
            "value": "Hello World"
        }"#;
        let snippet: SnippetItem = serde_json::from_str(json).unwrap();
        assert_eq!(snippet.id, "abc-123");
        assert_eq!(snippet.label, Some("My Snippet".to_string()));
        assert_eq!(snippet.value, "Hello World");
    }

    #[test]
    fn test_deserialize_snippet_without_label() {
        let json = r#"{
            "id": "abc-123",
            "value": "Hello World"
        }"#;
        let snippet: SnippetItem = serde_json::from_str(json).unwrap();
        assert!(snippet.label.is_none());
    }

    #[test]
    fn test_serialize_skips_none_label() {
        let snippet = SnippetItem::new("value".to_string(), None);
        let json = serde_json::to_string(&snippet).unwrap();
        assert!(!json.contains("label"));
    }
}
