//! Tauri commands for `uti_core::config`

mod commands;

pub use commands::{
    open_config_folder, open_launcher_config, open_snippets_config, read_config, reload_config,
    save_config,
};
pub use uti_core::config::*;
