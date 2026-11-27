//! Widget construction for the main window (no state logic)

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use crate::state::list::{Key, Tab};
use crate::state::menu::{MenuEntry, MenuItem};

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
        Tab::Launcher => "<b>↵</b> Launch   <b>→</b> Open   <b>F2</b> Edit   <b>Del</b> Delete",
    }
}

/// Footer key hints while a Launcher level is open.
pub const CASCADE_HINTS: &str = "<b>↵</b> Launch   <b>→</b> Open   <b>←</b> Back   <b>F2</b> Edit";

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
        .child(&with_drop_line(&list))
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

/// Shows or hides the "more" buttons of `list`'s rows.
pub fn show_more_buttons(list: &gtk::ListBox, visible: bool) {
    let mut i = 0;
    while let Some(row) = list.row_at_index(i) {
        let mut child = row.child().and_then(|c| c.first_child());
        while let Some(widget) = child {
            if widget.has_css_class("uti-more") {
                widget.set_visible(visible);
            }
            child = widget.next_sibling();
        }
        i += 1;
    }
}

/// Number of shown "more" buttons in `list` (debug state).
pub fn visible_more_buttons(list: &gtk::ListBox) -> usize {
    let mut n = 0;
    let mut i = 0;
    while let Some(row) = list.row_at_index(i) {
        let mut child = row.child().and_then(|c| c.first_child());
        while let Some(widget) = child {
            n += usize::from(widget.has_css_class("uti-more") && widget.is_visible());
            child = widget.next_sibling();
        }
        i += 1;
    }
    n
}

/// Response id of the delete button of [`delete_folder_dialog`].
pub const DELETE_RESPONSE: &str = "delete";

/// Asks before deleting a folder with its items: `title`, `body` and the
/// cancel and delete button labels.
pub fn delete_folder_dialog(
    title: &str,
    body: &str,
    cancel: &str,
    delete: &str,
) -> adw::AlertDialog {
    let dialog = adw::AlertDialog::new(Some(title), Some(body));
    dialog.add_responses(&[("cancel", cancel), (DELETE_RESPONSE, delete)]);
    dialog.set_response_appearance(DELETE_RESPONSE, adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some(DELETE_RESPONSE));
    dialog.set_close_response("cancel");
    dialog
}

/// Action group name of the row context menu actions.
pub const ROW_ACTIONS: &str = "row";

/// Label, action name (in [`ROW_ACTIONS`]) and shown shortcut.
pub fn menu_entry(item: MenuItem) -> (&'static str, &'static str, Option<&'static str>) {
    use MenuItem::*;
    match item {
        AddToSnippets => ("Add to Snippets", "add-snippet", Some("s")),
        DeleteFromHistory => ("Delete from History", "delete", Some("Delete")),
        Edit => ("Edit…", "edit", Some("F2")),
        AddNew => ("New Snippet…", "new", None),
        MoveUp => ("Move Up", "move-up", None),
        MoveDown => ("Move Down", "move-down", None),
        Delete => ("Delete", "delete", Some("Delete")),
        NewFolder => ("Folder…", "new-folder", None),
        NewSeparator => ("Separator", "new-separator", None),
        NewCommand => ("Command…", "new-command", None),
        NewSpecial => ("Special Item…", "new-special", None),
        AddToLauncher => ("Add to Launcher", "add-to-launcher", None),
        OpenFolder => ("Open Folder", "open-folder", None),
        CopyPath => ("Copy Path", "copy-path", None),
        RemoveFromHistory => ("Remove from History", "remove-recent", None),
        FixConfig => ("launcher.json Has Errors…", "fix-config", None),
    }
}

fn menu_item(item: MenuItem, target: &glib::Variant) -> gio::MenuItem {
    let (label, action, accel) = menu_entry(item);
    let entry = gio::MenuItem::new(Some(label), None);
    entry.set_action_and_target_value(Some(&format!("{ROW_ACTIONS}.{action}")), Some(target));
    if let Some(accel) = accel {
        entry.set_attribute_value("accel", Some(&accel.to_variant()));
    }
    entry
}

/// Context menu model for row `index` of Launcher level `depth` (0 for
/// the other tabs); entries are grouped in sections.
pub fn menu_model(sections: &[Vec<MenuEntry>], depth: usize, index: usize) -> gio::Menu {
    let target = (depth as i32, index as i32).to_variant();
    let menu = gio::Menu::new();
    for section in sections {
        let part = gio::Menu::new();
        for entry in section {
            match entry {
                MenuEntry::Item(item) => part.append_item(&menu_item(*item, &target)),
                MenuEntry::New(items) => {
                    let sub = gio::Menu::new();
                    for item in items {
                        sub.append_item(&menu_item(*item, &target));
                    }
                    part.append_submenu(Some("New"), &sub);
                },
            }
        }
        menu.append_section(None, &part);
    }
    menu
}

/// Content of a row drag inside the popup (drops from other apps carry
/// other values).
const DRAG_MARKER: &str = "uti-row";

/// `list` with an insertion line over it, shown while dragging.
pub fn with_drop_line(list: &gtk::ListBox) -> gtk::Overlay {
    let line = gtk::Box::builder()
        .valign(gtk::Align::Start)
        .can_target(false)
        .visible(false)
        .css_classes(["uti-drop-line"])
        .build();
    line.append(
        &gtk::Box::builder()
            .valign(gtk::Align::Center)
            .css_classes(["uti-drop-dot"])
            .build(),
    );
    line.append(
        &gtk::Box::builder()
            .hexpand(true)
            .valign(gtk::Align::Center)
            .css_classes(["uti-drop-bar"])
            .build(),
    );
    let overlay = gtk::Overlay::builder().child(list).build();
    overlay.add_overlay(&line);
    overlay
}

/// Height of the insertion line widget (dot included), for centering it.
const DROP_LINE_HEIGHT: f64 = 6.0;

fn drop_line(list: &gtk::ListBox) -> Option<gtk::Widget> {
    let overlay = list.parent().and_downcast::<gtk::Overlay>()?;
    let mut child = overlay.first_child();
    while let Some(w) = child {
        if w.has_css_class("uti-drop-line") {
            return Some(w);
        }
        child = w.next_sibling();
    }
    None
}

/// Shows the insertion line of `list` at `y` (list coordinates), or hides it.
pub fn show_drop_line(list: &gtk::ListBox, y: Option<f64>) {
    let Some(line) = drop_line(list) else {
        return;
    };
    if let Some(y) = y {
        line.set_margin_top((y - DROP_LINE_HEIGHT / 2.0).max(0.0) as i32);
    }
    line.set_visible(y.is_some());
}

/// The insertion line's top in `list` coordinates, when shown (debug state).
pub fn drop_line_y(list: &gtk::ListBox) -> Option<f64> {
    drop_line(list).filter(|l| l.is_visible()).map(|l| f64::from(l.margin_top()))
}

/// Highlights row `index` of `list` as the folder a drop goes into.
pub fn mark_drop_into(list: &gtk::ListBox, index: Option<usize>) {
    let mut i = 0;
    while let Some(row) = list.row_at_index(i) {
        if index == Some(i as usize) {
            row.add_css_class("uti-drop-into");
        } else {
            row.remove_css_class("uti-drop-into");
        }
        i += 1;
    }
}

/// The row of `list` highlighted by [`mark_drop_into`] (debug state).
pub fn drop_into(list: &gtk::ListBox) -> Option<usize> {
    (0..)
        .map_while(|i| list.row_at_index(i))
        .position(|r| r.has_css_class("uti-drop-into"))
}

/// Top of row `index` of `list` (its bottom for `below`), in list
/// coordinates.
pub fn row_edge(list: &gtk::ListBox, index: usize, below: bool) -> Option<f64> {
    let b = list.row_at_index(index as i32)?.compute_bounds(list)?;
    Some(f64::from(b.y() + if below { b.height() } else { 0.0 }))
}

/// Lets `row` be dragged while `enabled()` holds; the row is dimmed and
/// shown under the pointer while dragged.
pub fn drag_source(
    row: &gtk::ListBoxRow,
    enabled: impl Fn() -> bool + 'static,
    on_begin: impl Fn() + 'static,
    on_end: impl Fn() + 'static,
) {
    let source = gtk::DragSource::builder().actions(gdk::DragAction::MOVE).build();
    let hot = std::rc::Rc::new(std::cell::Cell::new((0, 0)));
    let h = hot.clone();
    source.connect_prepare(move |_, x, y| {
        h.set((x as i32, y as i32));
        enabled().then(|| gdk::ContentProvider::for_value(&DRAG_MARKER.to_value()))
    });
    let r = row.clone();
    source.connect_drag_begin(move |source, _| {
        let image = gtk::WidgetPaintable::new(Some(&r)).current_image();
        let (x, y) = hot.get();
        source.set_icon(Some(&image), x, y);
        r.add_css_class("uti-dragging");
        on_begin();
    });
    let r = row.clone();
    source.connect_drag_end(move |_, _, _| {
        r.remove_css_class("uti-dragging");
        on_end();
    });
    row.add_controller(source);
}

/// The row of `list` at `y` (list coordinates) and `y` within it.
fn hit_at(list: &gtk::ListBox, y: f64) -> Option<crate::state::drop::Hit> {
    let row = list.row_at_y(y as i32)?;
    let b = row.compute_bounds(list)?;
    Some(crate::state::drop::Hit {
        index: row.index() as usize,
        y: y - f64::from(b.y()),
        height: f64::from(b.height()),
    })
}

/// Takes row drags over `list`: reports the row under the pointer while
/// moving and at the drop, and scrolls near the edges of its scrolled
/// window. `accept` says whether the current spot takes the drop.
pub fn drop_target(
    list: &gtk::ListBox,
    accept: std::rc::Rc<std::cell::Cell<bool>>,
    on_motion: impl Fn(Option<crate::state::drop::Hit>) + 'static,
    on_leave: impl Fn() + 'static,
    on_drop: impl Fn(Option<crate::state::drop::Hit>) + 'static,
) {
    use std::cell::Cell;
    use std::rc::Rc;

    let target = gtk::DropTarget::new(glib::Type::STRING, gdk::DragAction::MOVE);
    // Pixels to scroll per tick; a ticker runs while it is not zero.
    let scroll = Rc::new(Cell::new(0.0));
    let ticking = Rc::new(Cell::new(false));
    let (lb, a, s, t) = (list.clone(), accept.clone(), scroll.clone(), ticking);
    target.connect_motion(move |_, _, y| {
        on_motion(hit_at(&lb, y));
        if let Some(pane) = lb
            .ancestor(gtk::ScrolledWindow::static_type())
            .and_downcast::<gtk::ScrolledWindow>()
        {
            let vy = lb.compute_point(&pane, &gtk::graphene::Point::new(0.0, y as f32));
            let vy = vy.map_or(0.0, |p| f64::from(p.y()));
            s.set(crate::state::drop::autoscroll(vy, f64::from(pane.height())));
            if s.get() != 0.0 && !t.get() {
                t.set(true);
                let (s, t) = (s.clone(), t.clone());
                glib::timeout_add_local(std::time::Duration::from_millis(30), move || {
                    let adj = pane.vadjustment();
                    if s.get() == 0.0 {
                        t.set(false);
                        return glib::ControlFlow::Break;
                    }
                    adj.set_value(adj.value() + s.get());
                    glib::ControlFlow::Continue
                });
            }
        }
        if a.get() {
            gdk::DragAction::MOVE
        } else {
            gdk::DragAction::empty()
        }
    });
    let s = scroll.clone();
    target.connect_leave(move |_| {
        s.set(0.0);
        on_leave();
    });
    let lb = list.clone();
    target.connect_drop(move |_, value, _, y| {
        scroll.set(0.0);
        if value.get::<String>().ok().as_deref() != Some(DRAG_MARKER) {
            return false;
        }
        on_drop(hit_at(&lb, y));
        accept.get()
    });
    list.add_controller(target);
}
