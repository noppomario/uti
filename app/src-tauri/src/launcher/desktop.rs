//! Desktop file search functionality
//!
//! Searches .desktop files in standard XDG directories and parses them
//! to provide application search results.

use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A desktop application entry parsed from .desktop file
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopApp {
    /// Unique identifier (desktop file name without extension)
    pub id: String,
    /// Display name
    pub name: String,
    /// Exec command (cleaned of field codes)
    pub exec: String,
    /// Icon name or path
    pub icon: Option<String>,
    /// Short description
    pub comment: Option<String>,
}

/// Search for desktop applications matching the query
///
/// Searches in standard XDG directories:
/// - /usr/share/applications
/// - /usr/local/share/applications
/// - ~/.local/share/applications
///
/// Results are sorted by relevance (exact match > starts with > contains)
pub fn search_desktop_files(query: &str) -> Vec<DesktopApp> {
    let query_lower = query.to_lowercase();
    if query_lower.is_empty() {
        return Vec::new();
    }

    let mut apps: Vec<DesktopApp> = Vec::new();
    let mut seen_ids: HashMap<String, usize> = HashMap::new();

    // Search directories in priority order (user > local > system)
    let dirs = get_desktop_dirs();

    for dir in dirs {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                    continue;
                }

                if let Some(app) = parse_desktop_file(&path) {
                    // Skip hidden or no-display apps
                    let name_lower = app.name.to_lowercase();

                    // Check if name matches query
                    if !name_lower.contains(&query_lower) {
                        // Also check comment
                        if let Some(ref comment) = app.comment {
                            if !comment.to_lowercase().contains(&query_lower) {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }

                    // Deduplicate by id (keep first seen - higher priority dir)
                    if let Some(&existing_idx) = seen_ids.get(&app.id) {
                        // Update if this is a better match
                        let existing = &apps[existing_idx];
                        let existing_name_lower = existing.name.to_lowercase();

                        // Prefer exact name match
                        if name_lower == query_lower && existing_name_lower != query_lower {
                            apps[existing_idx] = app;
                        }
                    } else {
                        seen_ids.insert(app.id.clone(), apps.len());
                        apps.push(app);
                    }
                }
            }
        }
    }

    // Sort by relevance
    apps.sort_by(|a, b| {
        let a_lower = a.name.to_lowercase();
        let b_lower = b.name.to_lowercase();

        // Exact match first
        let a_exact = a_lower == query_lower;
        let b_exact = b_lower == query_lower;
        if a_exact != b_exact {
            return b_exact.cmp(&a_exact);
        }

        // Starts with query
        let a_starts = a_lower.starts_with(&query_lower);
        let b_starts = b_lower.starts_with(&query_lower);
        if a_starts != b_starts {
            return b_starts.cmp(&a_starts);
        }

        // Alphabetical
        a_lower.cmp(&b_lower)
    });

    // Limit results
    apps.truncate(20);

    apps
}

/// Get list of directories to search for .desktop files
fn get_desktop_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User directory (highest priority)
    if let Some(data_dir) = dirs::data_local_dir() {
        dirs.push(data_dir.join("applications"));
    }

    // Flatpak exports
    if let Some(data_dir) = dirs::data_local_dir() {
        dirs.push(data_dir.join("flatpak/exports/share/applications"));
    }

    // Local system directory
    dirs.push(PathBuf::from("/usr/local/share/applications"));

    // System directory
    dirs.push(PathBuf::from("/usr/share/applications"));

    // Flatpak system apps
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));

    dirs
}

/// Parse a .desktop file and return DesktopApp if valid
fn parse_desktop_file(path: &PathBuf) -> Option<DesktopApp> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_desktop_entry = false;
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut icon: Option<String> = None;
    let mut comment: Option<String> = None;
    let mut no_display = false;
    let mut hidden = false;
    let mut app_type: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        // Section headers
        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        // Parse key=value
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Name" => name = Some(value.to_string()),
                "Exec" => exec = Some(clean_exec(value)),
                "Icon" => icon = Some(value.to_string()),
                "Comment" => comment = Some(value.to_string()),
                "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
                "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
                "Type" => app_type = Some(value.to_string()),
                _ => {}
            }
        }
    }

    // Skip if hidden, no-display, or not an Application
    if no_display || hidden {
        return None;
    }

    if app_type.as_deref() != Some("Application") {
        return None;
    }

    // Must have name and exec
    let name = name?;
    let exec = exec?;

    // Get id from filename
    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())?;

    Some(DesktopApp {
        id,
        name,
        exec,
        icon,
        comment,
    })
}

/// Clean exec string by removing field codes (%f, %F, %u, %U, etc.)
fn clean_exec(exec: &str) -> String {
    let mut result = String::new();
    let mut chars = exec.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Skip the field code character
            chars.next();
        } else {
            result.push(c);
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_exec() {
        assert_eq!(clean_exec("firefox %u"), "firefox");
        assert_eq!(clean_exec("nautilus %U"), "nautilus");
        assert_eq!(clean_exec("code --new-window %F"), "code --new-window");
        assert_eq!(clean_exec("gnome-terminal"), "gnome-terminal");
    }

    #[test]
    fn test_search_empty_query() {
        let results = search_desktop_files("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_get_desktop_dirs() {
        let dirs = get_desktop_dirs();
        assert!(!dirs.is_empty());
        assert!(dirs.iter().any(|d| d.ends_with("applications")));
    }
}
