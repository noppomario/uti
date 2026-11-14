//! Launch at login via `$XDG_CONFIG_HOME/autostart/uti.desktop`
//!
//! Same file name as the Tauri app wrote, so an existing setting carries over.

use std::io;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "uti.desktop";

const ENTRY: &str = "[Desktop Entry]
Type=Application
Name=uti
Exec=uti --minimized
X-GNOME-Autostart-enabled=true
NoDisplay=true
";

/// `$XDG_CONFIG_HOME/autostart`.
pub fn default_dir() -> PathBuf {
    gtk::glib::user_config_dir().join("autostart")
}

/// Whether the autostart entry exists in `dir`.
pub fn is_enabled(dir: &Path) -> bool {
    dir.join(FILE_NAME).exists()
}

/// Creates or removes the autostart entry in `dir`.
///
/// # Errors
///
/// When the directory or file cannot be written or removed.
pub fn set_enabled(dir: &Path, enabled: bool) -> io::Result<()> {
    let path = dir.join(FILE_NAME);
    if enabled {
        std::fs::create_dir_all(dir)?;
        std::fs::write(path, ENTRY)
    } else {
        match std::fs::remove_file(path) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("uti-autostart-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn enable_then_disable() {
        let dir = temp_dir("toggle");
        assert!(!is_enabled(&dir));
        set_enabled(&dir, true).unwrap();
        assert!(is_enabled(&dir));
        let entry = std::fs::read_to_string(dir.join(FILE_NAME)).unwrap();
        assert!(entry.contains("Exec=uti --minimized"));
        set_enabled(&dir, false).unwrap();
        assert!(!is_enabled(&dir));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn disabling_when_absent_is_ok() {
        let dir = temp_dir("absent");
        set_enabled(&dir, false).unwrap();
    }
}
