//! Framework-independent logic shared by the uti app and daemon
//!
//! Configuration, clipboard history, snippets and launcher configuration.
//! No GUI toolkit dependencies.

pub mod clipboard;
pub mod config;
pub mod dbus;
#[cfg(feature = "launcher")]
pub mod launcher;
pub mod snippets;
