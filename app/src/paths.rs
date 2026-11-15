//! XDG locations for uti files
//!
//! Configuration (edited by users) lives in `$XDG_CONFIG_HOME/uti`; data
//! written by the app, such as clipboard history, in `$XDG_DATA_HOME/uti`.

use std::io;
use std::path::{Path, PathBuf};

/// `$XDG_CONFIG_HOME/uti` (usually `~/.config/uti`).
pub fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("uti")
}

/// `$XDG_DATA_HOME/uti` (usually `~/.local/share/uti`).
pub fn data_dir() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("uti")
}

/// Clipboard history file.
pub fn clipboard_history() -> PathBuf {
    data_dir().join("clipboard.json")
}

/// Clipboard history location used up to v0.2.x.
pub fn legacy_clipboard_history() -> PathBuf {
    config_dir().join("clipboard.json")
}

/// Moves `old` to `new` if `old` exists and `new` does not.
///
/// Falls back to copy + remove when a rename is not possible (e.g. across
/// file systems). The copy is written with [`crate::fs::write_atomic`].
///
/// # Returns
///
/// `true` if a file was moved.
///
/// # Errors
///
/// Returns an error if reading, writing or removing fails; `old` is kept
/// unless `new` was written successfully.
pub fn migrate_file(old: &Path, new: &Path) -> io::Result<bool> {
    if !old.exists() || new.exists() {
        return Ok(false);
    }
    if let Some(parent) = new.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if std::fs::rename(old, new).is_err() {
        crate::fs::write_atomic(new, &std::fs::read(old)?)?;
        std::fs::remove_file(old)?;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn moves_legacy_file_when_new_is_missing() {
        let dir = TempDir::new().unwrap();
        let old = dir.path().join("config/uti/clipboard.json");
        let new = dir.path().join("data/uti/clipboard.json");
        std::fs::create_dir_all(old.parent().unwrap()).unwrap();
        std::fs::write(&old, b"{\"items\":[]}").unwrap();

        assert!(migrate_file(&old, &new).unwrap());

        assert!(!old.exists());
        assert_eq!(std::fs::read(&new).unwrap(), b"{\"items\":[]}");
    }

    #[test]
    fn keeps_new_file_when_both_exist() {
        let dir = TempDir::new().unwrap();
        let old = dir.path().join("old.json");
        let new = dir.path().join("new.json");
        std::fs::write(&old, b"old").unwrap();
        std::fs::write(&new, b"new").unwrap();

        assert!(!migrate_file(&old, &new).unwrap());

        assert_eq!(std::fs::read(&new).unwrap(), b"new");
        assert!(old.exists());
    }

    #[test]
    fn does_nothing_without_legacy_file() {
        let dir = TempDir::new().unwrap();
        let new = dir.path().join("new.json");

        assert!(!migrate_file(&dir.path().join("missing.json"), &new).unwrap());
        assert!(!new.exists());
    }
}
