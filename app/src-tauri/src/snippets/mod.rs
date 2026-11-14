//! Tauri commands for `uti_core::snippets`

mod commands;

pub use commands::{add_snippet, get_snippets};
pub use uti_core::snippets::*;
