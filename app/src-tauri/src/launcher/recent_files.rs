//! Recent files module for reading system file history
//!
//! Reads recent files from:
//! - ~/.local/share/recently-used.xbel (freedesktop standard)
//! - VSCode storage.json (openedPathsList)

use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// Maximum number of files to return per command
const MAX_FILES_PER_COMMAND: usize = 10;

/// Recent file entry returned to frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub timestamp: String,
}

// XBEL XML structures for deserialization

#[derive(Debug, Deserialize)]
struct Xbel {
    #[serde(rename = "bookmark", default)]
    bookmarks: Vec<Bookmark>,
}

#[derive(Debug, Deserialize)]
struct Bookmark {
    #[serde(rename = "@href")]
    href: String,
    #[serde(rename = "@visited")]
    visited: Option<String>,
    #[serde(rename = "@modified")]
    modified: Option<String>,
    #[serde(default)]
    info: Option<BookmarkInfo>,
}

#[derive(Debug, Deserialize)]
struct BookmarkInfo {
    #[serde(default)]
    metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    #[serde(rename = "applications", default)]
    applications: Option<Applications>,
}

#[derive(Debug, Deserialize)]
struct Applications {
    #[serde(rename = "application", default)]
    apps: Vec<Application>,
}

#[derive(Debug, Deserialize)]
struct Application {
    #[serde(rename = "@name")]
    name: String,
}

/// Get the path to the recently-used.xbel file
fn get_xbel_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("recently-used.xbel")
}

/// Decode a file:// URI to a file path
fn decode_file_uri(uri: &str) -> Option<String> {
    if !uri.starts_with("file://") {
        return None;
    }

    // Remove file:// prefix and decode percent-encoded characters
    let path = &uri[7..];
    percent_encoding::percent_decode_str(path)
        .decode_utf8()
        .ok()
        .map(|s| s.to_string())
}

/// Extract filename from path
fn get_filename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}

/// Get recent files from recently-used.xbel
///
/// # Arguments
///
/// * `app_name` - Application name to filter by (optional, e.g., "org.gnome.Nautilus")
///   - Required when using system XBEL (custom_path is None)
///   - Optional when using per-app XBEL (custom_path is Some)
/// * `custom_path` - Optional custom path to XBEL file (supports ~ expansion)
///
/// # Returns
///
/// Vector of recent files, sorted by timestamp (newest first), limited to MAX_FILES_PER_COMMAND
pub fn get_recent_files_from_xbel(
    app_name: Option<&str>,
    custom_path: Option<&str>,
) -> Vec<RecentFile> {
    let xbel_path = match custom_path {
        Some(p) => expand_tilde(p),
        None => get_xbel_path(),
    };

    let content = match fs::read_to_string(&xbel_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read recently-used.xbel: {}", e);
            return Vec::new();
        }
    };

    let xbel: Xbel = match from_str(&content) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Failed to parse recently-used.xbel: {}", e);
            return Vec::new();
        }
    };

    let mut files: Vec<RecentFile> = xbel
        .bookmarks
        .into_iter()
        .filter(|b| {
            // If app_name is provided, filter by it
            // If app_name is None (custom path scenario), include all entries
            match app_name {
                Some(name) => b
                    .info
                    .as_ref()
                    .and_then(|i| i.metadata.as_ref())
                    .and_then(|m| m.applications.as_ref())
                    .map(|apps| apps.apps.iter().any(|a| a.name == name))
                    .unwrap_or(false),
                None => true,
            }
        })
        .filter_map(|b| {
            // Convert to RecentFile
            let path = decode_file_uri(&b.href)?;
            let name = get_filename(&path);
            let timestamp = b.visited.or(b.modified).unwrap_or_default();

            Some(RecentFile {
                path,
                name,
                timestamp,
            })
        })
        .collect();

    // Sort by timestamp (newest first)
    files.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Limit to MAX_FILES_PER_COMMAND
    files.truncate(MAX_FILES_PER_COMMAND);

    files
}

/// Get all available application names from recently-used.xbel
///
/// Useful for debugging and discovering what applications are tracked
#[allow(dead_code)]
pub fn get_available_apps() -> Vec<String> {
    let xbel_path = get_xbel_path();

    let content = match fs::read_to_string(&xbel_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let xbel: Xbel = match from_str(&content) {
        Ok(x) => x,
        Err(_) => return Vec::new(),
    };

    let mut apps: Vec<String> = xbel
        .bookmarks
        .iter()
        .filter_map(|b| {
            b.info
                .as_ref()
                .and_then(|i| i.metadata.as_ref())
                .and_then(|m| m.applications.as_ref())
        })
        .flat_map(|apps| apps.apps.iter().map(|a| a.name.clone()))
        .collect();

    apps.sort();
    apps.dedup();
    apps
}

/// Expand tilde to home directory
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(path)
}

/// Get recent files from VSCode state.vscdb SQLite database
///
/// VSCode stores recent files in `state.vscdb` SQLite database.
/// The key `history.recentlyOpenedPathsList` contains JSON with entries array.
/// Each entry has either `fileUri` (for files), `folderUri` (for folders),
/// or `workspace.configPath` (for workspaces).
///
/// # Arguments
///
/// * `vscdb_path` - Path to state.vscdb (supports ~ expansion)
///
/// # Returns
///
/// Vector of recent files, limited to MAX_FILES_PER_COMMAND
pub fn get_recent_files_from_vscode(vscdb_path: &str) -> Vec<RecentFile> {
    let path = expand_tilde(vscdb_path);

    // Open SQLite database (read-only)
    let conn = match rusqlite::Connection::open_with_flags(
        &path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open VSCode state.vscdb at {:?}: {}", path, e);
            return Vec::new();
        }
    };

    // Query the history.recentlyOpenedPathsList key
    let query_result: Result<String, _> = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = 'history.recentlyOpenedPathsList'",
        [],
        |row| row.get(0),
    );

    let json_str = match query_result {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to query VSCode recent files: {}", e);
            return Vec::new();
        }
    };

    let json: Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse VSCode recent files JSON: {}", e);
            return Vec::new();
        }
    };

    // Navigate to entries array
    let entries = match json.get("entries").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => {
            eprintln!("No entries found in VSCode recent files");
            return Vec::new();
        }
    };

    let mut files: Vec<RecentFile> = entries
        .iter()
        .filter_map(|entry| {
            // Try fileUri first (individual files), then folderUri (folders),
            // then workspace.configPath (workspaces)
            let uri = entry
                .get("fileUri")
                .or_else(|| entry.get("folderUri"))
                .or_else(|| entry.get("workspace").and_then(|w| w.get("configPath")))
                .and_then(|v| v.as_str())?;

            let path = decode_file_uri(uri)?;
            let name = get_filename(&path);

            Some(RecentFile {
                path,
                name,
                timestamp: String::new(), // VSCode doesn't store timestamps
            })
        })
        .collect();

    // Limit to MAX_FILES_PER_COMMAND
    files.truncate(MAX_FILES_PER_COMMAND);

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Run with: cargo test test_real_system -- --ignored --nocapture
    fn test_real_system_apps() {
        let apps = get_available_apps();
        println!("Available apps in recently-used.xbel:");
        for app in &apps {
            println!("  - {}", app);
        }
        assert!(!apps.is_empty(), "Should have at least one app");
    }

    #[test]
    #[ignore] // Run with: cargo test test_real_system -- --ignored --nocapture
    fn test_real_system_nautilus() {
        // System XBEL with app filter
        let files = get_recent_files_from_xbel(Some("org.gnome.Nautilus"), None);
        println!("Recent files for org.gnome.Nautilus:");
        for file in &files {
            println!("  - {} ({})", file.name, file.path);
        }
    }

    #[test]
    #[ignore] // Run with: cargo test test_real_system -- --ignored --nocapture
    fn test_real_system_text_editor() {
        // Per-app XBEL without filter (all entries from custom path)
        let files = get_recent_files_from_xbel(
            None,
            Some("~/.local/share/org.gnome.TextEditor/recently-used.xbel"),
        );
        println!("Recent files for gnome-text-editor:");
        for file in &files {
            println!("  - {} ({})", file.name, file.path);
        }
    }

    #[test]
    fn test_decode_file_uri_simple() {
        let uri = "file:///home/user/test.txt";
        assert_eq!(
            decode_file_uri(uri),
            Some("/home/user/test.txt".to_string())
        );
    }

    #[test]
    fn test_decode_file_uri_with_spaces() {
        let uri = "file:///home/user/My%20Documents/test.txt";
        assert_eq!(
            decode_file_uri(uri),
            Some("/home/user/My Documents/test.txt".to_string())
        );
    }

    #[test]
    fn test_decode_file_uri_with_japanese() {
        let uri = "file:///home/user/%E3%83%86%E3%82%B9%E3%83%88.txt";
        assert_eq!(
            decode_file_uri(uri),
            Some("/home/user/テスト.txt".to_string())
        );
    }

    #[test]
    fn test_decode_file_uri_non_file() {
        let uri = "https://example.com/test.txt";
        assert_eq!(decode_file_uri(uri), None);
    }

    #[test]
    fn test_get_filename() {
        assert_eq!(get_filename("/home/user/test.txt"), "test.txt");
        assert_eq!(get_filename("/home/user/folder/"), "folder");
        assert_eq!(get_filename("test.txt"), "test.txt");
    }

    #[test]
    fn test_get_xbel_path() {
        let path = get_xbel_path();
        assert!(path.ends_with("recently-used.xbel"));
    }

    #[test]
    fn test_parse_xbel_sample() {
        let sample = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbel version="1.0">
  <bookmark href="file:///home/user/test.txt" visited="2025-01-01T12:00:00Z">
    <info>
      <metadata owner="http://freedesktop.org">
        <applications>
          <application name="org.gnome.TextEditor"/>
        </applications>
      </metadata>
    </info>
  </bookmark>
</xbel>"#;

        let xbel: Xbel = from_str(sample).expect("Failed to parse sample XBEL");
        assert_eq!(xbel.bookmarks.len(), 1);
        assert_eq!(xbel.bookmarks[0].href, "file:///home/user/test.txt");
    }

    #[test]
    fn test_expand_tilde() {
        let path = expand_tilde("~/test");
        assert!(path.ends_with("test"));
        assert!(!path.to_string_lossy().contains("~"));
    }

    #[test]
    fn test_expand_tilde_no_tilde() {
        let path = expand_tilde("/home/user/test");
        assert_eq!(path, PathBuf::from("/home/user/test"));
    }

    #[test]
    fn test_parse_vscode_state_db() {
        // Create a temp SQLite database with the same structure as VSCode's state.vscdb
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("state.vscdb");

        let conn = rusqlite::Connection::open(&db_path).expect("Failed to create test db");
        conn.execute(
            "CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT)",
            [],
        )
        .expect("Failed to create table");

        let sample_json = r#"{"entries":[{"folderUri":"file:///home/user/project1"},{"fileUri":"file:///home/user/test.txt"},{"workspace":{"configPath":"file:///home/user/workspace.code-workspace"}}]}"#;
        conn.execute(
            "INSERT INTO ItemTable (key, value) VALUES (?1, ?2)",
            ["history.recentlyOpenedPathsList", sample_json],
        )
        .expect("Failed to insert test data");

        let path = db_path.to_str().unwrap();
        let files = get_recent_files_from_vscode(path);
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].path, "/home/user/project1");
        assert_eq!(files[0].name, "project1");
        assert_eq!(files[1].path, "/home/user/test.txt");
        assert_eq!(files[1].name, "test.txt");
        assert_eq!(files[2].path, "/home/user/workspace.code-workspace");
        assert_eq!(files[2].name, "workspace.code-workspace");
    }

    #[test]
    #[ignore] // Run with: cargo test test_real_vscode -- --ignored --nocapture
    fn test_real_vscode_insiders() {
        let files = get_recent_files_from_vscode(
            "~/.config/Code - Insiders/User/globalStorage/state.vscdb",
        );
        println!("Recent files for VSCode Insiders:");
        for file in &files {
            println!("  - {} ({})", file.name, file.path);
        }
    }
}
