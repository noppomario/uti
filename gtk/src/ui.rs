//! Widget construction for the main window (no state logic)

use relm4::gtk::{self, gdk, glib, pango, prelude::*};

use crate::state::list::{Key, Tab};

/// Stack page names, also used as CSS-friendly ids.
pub fn page_name(tab: Tab) -> &'static str {
    match tab {
        Tab::Prompt => "prompt",
        Tab::Clipboard => "clipboard",
        Tab::Snippets => "snippets",
        Tab::Launcher => "launcher",
    }
}

/// Tab button icon and label.
pub fn tab_label(tab: Tab) -> (&'static str, &'static str) {
    match tab {
        Tab::Prompt => ("document-edit-symbolic", "Prompt"),
        Tab::Clipboard => ("edit-paste-symbolic", "Clipboard"),
        Tab::Snippets => ("starred-symbolic", "Snippets"),
        Tab::Launcher => ("system-run-symbolic", "Launcher"),
    }
}

/// Search placeholder for a list tab.
pub fn search_placeholder(tab: Tab) -> &'static str {
    match tab {
        Tab::Clipboard => "Search history...",
        Tab::Snippets => "Search snippets...",
        Tab::Launcher => "Search apps...",
        Tab::Prompt => "",
    }
}

/// Maps a key press to a list key; `None` for keys the lists ignore.
pub fn map_key(key: gdk::Key, state: gdk::ModifierType) -> Option<Key> {
    let plain = !state.intersects(
        gdk::ModifierType::CONTROL_MASK
            | gdk::ModifierType::ALT_MASK
            | gdk::ModifierType::SUPER_MASK,
    );
    if !plain {
        return None;
    }
    Some(match key {
        gdk::Key::Up => Key::Up,
        gdk::Key::Down => Key::Down,
        gdk::Key::Left => Key::Left,
        gdk::Key::Right => Key::Right,
        gdk::Key::Return | gdk::Key::KP_Enter => Key::Enter,
        gdk::Key::Escape => Key::Escape,
        gdk::Key::s | gdk::Key::S => Key::Star,
        k => {
            let digit = k.to_unicode()?.to_digit(10)?;
            if digit == 0 {
                return None;
            }
            Key::Digit(digit as u8)
        },
    })
}

/// One row of a list tab.
pub struct RowSpec<'a> {
    pub index: usize,
    pub text: &'a str,
    pub tooltip: &'a str,
    /// `Some(starred)` shows a star toggle (Clipboard tab).
    pub star: Option<bool>,
}

/// Builds a list row; `on_star` is called with the row's visible index.
pub fn build_row(spec: &RowSpec<'_>, on_star: impl Fn(usize) + 'static) -> gtk::ListBoxRow {
    let line = spec.text.lines().next().unwrap_or_default();
    let label = gtk::Label::builder()
        .label(format!("{}: {line}", spec.index + 1))
        .xalign(0.0)
        .hexpand(true)
        .ellipsize(pango::EllipsizeMode::End)
        .single_line_mode(true)
        .build();

    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row_box.append(&label);

    if let Some(starred) = spec.star {
        let star = gtk::Button::builder()
            .icon_name(if starred {
                "starred-symbolic"
            } else {
                "non-starred-symbolic"
            })
            .tooltip_text("Save as snippet (S)")
            .css_classes(["flat", "uti-star"])
            .focusable(false)
            .build();
        let index = spec.index;
        star.connect_clicked(move |_| on_star(index));
        row_box.append(&star);
    }

    gtk::ListBoxRow::builder()
        .child(&row_box)
        .tooltip_text(glib::markup_escape_text(spec.tooltip))
        .build()
}
