//! Widget construction for the main window (no state logic)

use adw::prelude::*;
use gtk::{gdk, glib};

use crate::state::list::{Key, Tab};

/// View stack page names.
pub fn page_name(tab: Tab) -> &'static str {
    match tab {
        Tab::Prompt => "prompt",
        Tab::Clipboard => "clipboard",
        Tab::Snippets => "snippets",
        Tab::Launcher => "launcher",
    }
}

/// Tab title and icon.
pub fn tab_label(tab: Tab) -> (&'static str, &'static str) {
    match tab {
        Tab::Prompt => ("Prompt", "document-edit-symbolic"),
        Tab::Clipboard => ("Clipboard", "edit-paste-symbolic"),
        Tab::Snippets => ("Snippets", "starred-symbolic"),
        Tab::Launcher => ("Launcher", "view-app-grid-symbolic"),
    }
}

/// Search placeholder for a list tab.
pub fn search_placeholder(tab: Tab) -> &'static str {
    match tab {
        Tab::Clipboard => "Search history",
        Tab::Snippets => "Search snippets",
        Tab::Launcher => "Search apps",
        Tab::Prompt => "",
    }
}

/// Footer key hints for a tab (Pango markup).
pub fn key_hints(tab: Tab) -> &'static str {
    match tab {
        Tab::Clipboard => {
            "<b>↵</b> Copy   <b>1–9</b> Pick   <b>S</b> Keep as Snippet   <b>Ctrl+F</b> Search"
        },
        Tab::Snippets => "<b>↵</b> Copy   <b>1–9</b> Pick   <b>Ctrl+F</b> Search",
        Tab::Launcher => "<b>↵</b> Launch   <b>1–9</b> Pick   <b>→</b> Recent Files",
        Tab::Prompt => "",
    }
}

/// Empty-state page shown as the list placeholder.
pub fn empty_page(tab: Tab) -> adw::StatusPage {
    let (icon, title, description) = match tab {
        Tab::Clipboard => {
            ("edit-paste-symbolic", "No Clipboard History", "Copied text appears here")
        },
        Tab::Snippets => {
            ("starred-symbolic", "No Snippets", "Press S on a clipboard item to keep it")
        },
        Tab::Launcher => ("view-app-grid-symbolic", "No Apps", ""),
        Tab::Prompt => ("document-edit-symbolic", "", ""),
    };
    adw::StatusPage::builder()
        .icon_name(icon)
        .title(title)
        .description(description)
        .css_classes(["compact"])
        .build()
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
    let first_line = spec.text.lines().find(|l| !l.trim().is_empty());

    // Number shortcut hint for 1-9; kept as an empty slot after that so the
    // text stays aligned.
    let hint = gtk::Label::builder()
        .label(if spec.index < 9 {
            (spec.index + 1).to_string()
        } else {
            String::new()
        })
        .xalign(0.5)
        .width_chars(2)
        .valign(gtk::Align::Center)
        .css_classes(["dim-label", "numeric", "uti-index"])
        .build();
    let title = gtk::Label::builder()
        .label(first_line.unwrap_or_default().trim())
        .xalign(0.0)
        .hexpand(true)
        .valign(gtk::Align::Center)
        .lines(1)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();

    let content = gtk::Box::builder().spacing(12).css_classes(["uti-row-box"]).build();
    content.append(&hint);
    content.append(&title);

    if let Some(starred) = spec.star {
        let mut classes = vec!["flat", "circular", "uti-star"];
        if starred {
            classes.push("starred");
        }
        let star = gtk::Button::builder()
            .icon_name(if starred {
                "starred-symbolic"
            } else {
                "non-starred-symbolic"
            })
            .tooltip_text("Keep as Snippet (S)")
            .valign(gtk::Align::Center)
            .css_classes(classes)
            .focusable(false)
            .build();
        let index = spec.index;
        star.connect_clicked(move |_| on_star(index));
        content.append(&star);
    }

    gtk::ListBoxRow::builder()
        .child(&content)
        .tooltip_text(glib::markup_escape_text(spec.tooltip))
        .css_classes(["uti-row"])
        .build()
}
