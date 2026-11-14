//! Tauri commands for `uti_core::clipboard`

mod commands;

pub use commands::{add_clipboard_item, get_clipboard_history, paste_item, remove_clipboard_item};
pub use uti_core::clipboard::*;
