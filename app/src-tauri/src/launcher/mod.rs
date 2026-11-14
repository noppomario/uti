//! Tauri commands for `uti_core::launcher`

mod commands;
pub mod desktop;
pub use desktop::DesktopApp;

pub use commands::{
    execute_command, get_launcher_config, get_recent_files, get_vscode_recent_files,
    search_desktop_files,
};
pub use uti_core::launcher::*;
