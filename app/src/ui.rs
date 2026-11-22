//! Widget construction for the main window (no state logic)

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use crate::state::list::{Key, Tab};
use crate::state::menu::MenuItem;

/// View stack page names.
pub fn page_name(tab: Tab) -> &'static str {
    match tab {
        Tab::Clipboard => "clipboard",
        Tab::Snippets => "snippets",
        Tab::Launcher => "launcher",
    }
}

/// Tab title and icon.
pub fn tab_label(tab: Tab) -> (&'static str, &'static str) {
    match tab {
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
    }
}

/// Footer key hints for a tab (Pango markup).
pub fn key_hints(tab: Tab) -> &'static str {
    match tab {
        Tab::Clipboard => {
            "<b>↵</b> Copy   <b>1–9</b> Pick   <b>S</b> Add to Snippets   <b>Ctrl+F</b> Search"
        },
        Tab::Snippets => "<b>↵</b> Copy   <b>1–9</b> Pick   <b>F2</b> Edit   <b>Ctrl+F</b> Search",
        Tab::Launcher => "<b>↵</b> Launch   <b>1–9</b> Pick   <b>→</b> Recent Files",
    }
}

/// Footer key hints while the Launcher jump list is open.
pub const JUMP_LIST_HINTS: &str = "<b>↵</b> Open   <b>1–9</b> Pick   <b>←</b> Back";

/// Empty-state page shown as the list placeholder.
pub fn empty_page(tab: Tab) -> adw::StatusPage {
    let (icon, title, description) = match tab {
        Tab::Clipboard => {
            ("edit-paste-symbolic", "No Clipboard History", "Copied text appears here")
        },
        Tab::Snippets => (
            "starred-symbolic",
            "No Snippets",
            "Right-click a clipboard item or press S to add it",
        ),
        Tab::Launcher => ("view-app-grid-symbolic", "No Apps", ""),
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
    if key == gdk::Key::F10 {
        return state.contains(gdk::ModifierType::SHIFT_MASK).then_some(Key::Menu);
    }
    Some(match key {
        gdk::Key::Up => Key::Up,
        gdk::Key::Down => Key::Down,
        gdk::Key::Left => Key::Left,
        gdk::Key::Right => Key::Right,
        gdk::Key::Return | gdk::Key::KP_Enter => Key::Enter,
        gdk::Key::Escape => Key::Escape,
        gdk::Key::s | gdk::Key::S => Key::AddSnippet,
        gdk::Key::Menu => Key::Menu,
        gdk::Key::Delete | gdk::Key::KP_Delete => Key::Delete,
        gdk::Key::F2 => Key::Rename,
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
    /// The row has a jump list (recent files, opened with Right).
    pub jump: bool,
}

/// Builds a list row; `on_jump` is called with the row's visible index when
/// its jump list button is clicked.
pub fn build_row(spec: &RowSpec<'_>, on_jump: impl Fn(usize) + 'static) -> gtk::ListBoxRow {
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

    if spec.jump {
        // A chevron, as on GNOME rows that lead to a sub-page.
        let jump = gtk::Button::builder()
            .icon_name("go-next-symbolic")
            .tooltip_text("Recent Files (→)")
            .valign(gtk::Align::Center)
            .css_classes(["flat", "uti-row-button"])
            .focusable(false)
            .build();
        let index = spec.index;
        jump.connect_clicked(move |_| on_jump(index));
        content.append(&jump);
    }
    gtk::ListBoxRow::builder()
        .child(&content)
        .tooltip_text(glib::markup_escape_text(spec.tooltip))
        .css_classes(["uti-row"])
        .build()
}

/// Action group name of the row context menu actions.
pub const ROW_ACTIONS: &str = "row";

/// Label, action name (in [`ROW_ACTIONS`]), shown shortcut and whether the
/// action takes the row index.
fn menu_entry(item: MenuItem) -> (&'static str, &'static str, Option<&'static str>, bool) {
    match item {
        MenuItem::AddToSnippets => ("Add to Snippets", "add-snippet", Some("s"), true),
        MenuItem::DeleteFromHistory => ("Delete from History", "delete", Some("Delete"), true),
        MenuItem::Edit => ("Edit…", "edit", Some("F2"), true),
        MenuItem::AddNew => ("New Snippet…", "new", None, false),
        MenuItem::MoveUp => ("Move Up", "move-up", None, true),
        MenuItem::MoveDown => ("Move Down", "move-down", None, true),
        MenuItem::Delete => ("Delete", "delete", Some("Delete"), true),
    }
}

/// Context menu model for row `index`; entries are grouped in sections.
pub fn menu_model(sections: &[Vec<MenuItem>], index: usize) -> gio::Menu {
    let menu = gio::Menu::new();
    for section in sections {
        let part = gio::Menu::new();
        for item in section {
            let (label, action, accel, indexed) = menu_entry(*item);
            let entry = gio::MenuItem::new(Some(label), None);
            let name = format!("{ROW_ACTIONS}.{action}");
            if indexed {
                entry.set_action_and_target_value(Some(&name), Some(&(index as i32).to_variant()));
            } else {
                entry.set_action_and_target_value(Some(&name), None);
            }
            if let Some(accel) = accel {
                entry.set_attribute_value("accel", Some(&accel.to_variant()));
            }
            part.append_item(&entry);
        }
        menu.append_section(None, &part);
    }
    menu
}
