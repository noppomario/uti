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
        Tab::Clipboard => "<b>↵</b> Copy   <b>1–9</b> Pick   <b>S</b> Add to Snippets",
        Tab::Snippets => "<b>↵</b> Copy   <b>1–9</b> Pick   <b>F2</b> Edit   <b>Del</b> Delete",
        Tab::Launcher => "<b>↵</b> Launch   <b>1–9</b> Pick   <b>→</b> Open",
    }
}

/// Footer key hints while a Launcher level is open.
pub const CASCADE_HINTS: &str = "<b>↵</b> Launch   <b>1–9</b> Pick   <b>→</b> Open   <b>←</b> Back";

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

/// What opens beside a row.
#[derive(Debug, Clone, Copy)]
pub enum Branch {
    Folder,
    /// A command's recent files.
    RecentFiles,
}

/// One row of a list.
pub struct RowSpec<'a> {
    /// Visible index, passed to `on_more`.
    pub index: usize,
    /// Number shortcut shown (1-9).
    pub hint: Option<usize>,
    /// Shown in place of the number (Launcher rows).
    pub icon: Option<gio::Icon>,
    pub text: &'a str,
    pub tooltip: &'a str,
    /// Marker for children that open beside the row.
    pub branch: Option<Branch>,
    /// The row has a context menu, also opened by its "more" button.
    pub more: bool,
}

/// Where a row's context menu opens, in list coordinates.
#[derive(Debug, Clone, Copy)]
pub struct MenuAnchor {
    pub x: f64,
    pub y: f64,
    /// The menu opens to the left of `x` (under a button at the right edge)
    /// instead of to the right, as at the pointer.
    pub end: bool,
}

/// Builds a list row; `on_more` is called with the row's index and the
/// anchor below its "more" button.
pub fn build_row(
    spec: &RowSpec<'_>,
    on_more: impl Fn(usize, Option<MenuAnchor>) + 'static,
) -> gtk::ListBoxRow {
    let first_line = spec.text.lines().find(|l| !l.trim().is_empty());

    // Number shortcut hint for 1-9; kept as an empty slot after that so the
    // text stays aligned.
    let hint = gtk::Label::builder()
        .label(spec.hint.map(|n| n.to_string()).unwrap_or_default())
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
        // Bounds the natural width of Launcher levels, which size to fit.
        .max_width_chars(40)
        .build();

    let content = gtk::Box::builder().spacing(12).css_classes(["uti-row-box"]).build();
    match &spec.icon {
        Some(icon) => content.append(
            &gtk::Image::builder()
                .gicon(icon)
                .pixel_size(16)
                .valign(gtk::Align::Center)
                .css_classes(["uti-row-icon"])
                .build(),
        ),
        None => content.append(&hint),
    }
    content.append(&title);

    if let Some(branch) = spec.branch {
        if matches!(branch, Branch::RecentFiles) {
            // Split look (as Windows 7 jump lists): the name launches, the
            // part after the line lists recent files.
            content.append(
                &gtk::Separator::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .css_classes(["uti-row-split"])
                    .build(),
            );
        }
        // A marker, not a button: hovering or clicking the row opens it.
        content.append(
            &gtk::Image::builder()
                .icon_name("go-next-symbolic")
                .valign(gtk::Align::Center)
                .css_classes(["dim-label", "uti-row-button"])
                .build(),
        );
    }
    if spec.more {
        let more = gtk::Button::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text("More (Menu)")
            .valign(gtk::Align::Center)
            .css_classes(["flat", "uti-row-button", "uti-more"])
            .focusable(false)
            .build();
        let index = spec.index;
        more.connect_clicked(move |b| {
            let point = b.ancestor(gtk::ListBox::static_type()).and_then(|list| {
                b.compute_bounds(&list).map(|r| MenuAnchor {
                    x: f64::from(r.x() + r.width()),
                    y: f64::from(r.y() + r.height()),
                    end: true,
                })
            });
            on_more(index, point);
        });
        content.append(&more);
    }

    gtk::ListBoxRow::builder()
        .child(&content)
        .tooltip_text(glib::markup_escape_text(spec.tooltip))
        .css_classes(["uti-row"])
        .build()
}

/// A Launcher level beside a row of `parent` (ADR-037): a list styled like
/// the main one, in a popover that does not grab the pointer or keyboard.
/// The caller sets `pointing_to`.
pub fn cascade_popover(parent: &gtk::ListBox) -> (gtk::Popover, gtk::ListBox) {
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .activate_on_single_click(true)
        .focusable(false)
        .css_classes(["navigation-sidebar", "uti-tree"])
        .build();
    let max_height = parent
        .native()
        .and_then(|n| n.surface())
        .and_then(|surface| parent.display().monitor_at_surface(&surface))
        .map_or(600, |m| m.geometry().height() - 64);
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .propagate_natural_width(true)
        .propagate_natural_height(true)
        .max_content_height(max_height)
        .child(&list)
        .build();
    let popover = gtk::Popover::builder()
        .has_arrow(false)
        .position(gtk::PositionType::Right)
        // Top edge at the row's top, growing down like classic menus.
        .valign(gtk::Align::Start)
        // No grab: the main window keeps keyboard focus and its clicks.
        .autohide(false)
        .css_classes(["uti-cascade"])
        .child(&scrolled)
        .build();
    popover.set_offset(CASCADE_OVERLAP, -CASCADE_PADDING);
    popover.set_parent(parent);
    (popover, list)
}

/// Pixels a Launcher level overlaps the level it opens from.
const CASCADE_OVERLAP: i32 = -2;
/// Padding above a level's first row (`popover.uti-cascade > contents`), so
/// the row lines up with the row it opened from.
const CASCADE_PADDING: i32 = 4;

/// A dim row saying a level has nothing to show (a list placeholder is not
/// shown in a list that never had rows).
pub fn empty_row(text: &str) -> gtk::ListBoxRow {
    gtk::ListBoxRow::builder()
        .child(
            &gtk::Label::builder()
                .label(text)
                .xalign(0.0)
                .css_classes(["dim-label", "uti-row-box"])
                .build(),
        )
        .selectable(false)
        .activatable(false)
        .focusable(false)
        .build()
}

/// A separator row between list rows (not selectable or activatable).
pub fn separator_row() -> gtk::ListBoxRow {
    gtk::ListBoxRow::builder()
        .child(&gtk::Separator::new(gtk::Orientation::Horizontal))
        .selectable(false)
        .activatable(false)
        .focusable(false)
        .css_classes(["uti-separator"])
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
