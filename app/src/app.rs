//! Main popup window controller
//!
//! Widgets send [`Msg`] values over a channel; one local task owns the model
//! and handles them in order, so no `RefCell` is needed and signal handlers
//! never re-enter the model. Keyboard focus is decided by [`Nav`]; after
//! every message the GTK focus is moved to match it.

use std::collections::HashMap;
use std::time::Instant;

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use crate::clipboard::ClipboardStore;
use crate::config::AppConfig;
use crate::dbus;
use crate::launcher::entry::{self, Entry};
use crate::launcher::{load_launcher_tree, LauncherNode, NodeKind};
use crate::settings;
use crate::snippets::{load_snippets, save_snippets, SnippetsStore};
use crate::state::launcher::{match_command, rank, SearchText};
use crate::state::list::{self, Key, Tab};
use crate::state::menu::row_menu;
use crate::state::nav::{Effect, Focus, Nav};
use crate::state::scrollbar::Scrollbar;
use crate::state::settings::{SettingsTarget, SnippetTarget};
use crate::state::size::Size;
use crate::state::tree::{Row, TreeAction, HOVER_DELAY};
use crate::state::window::{WindowCommand, WindowState};
use crate::ui;

/// Title the GNOME extension uses to find the main window.
pub const MAIN_WINDOW_TITLE: &str = "uti";
const RELEASES_URL: &str = "https://github.com/noppomario/uti/releases";

/// Messages handled by the main window.
#[derive(Debug)]
pub enum Msg {
    Toggle,
    Show,
    Hide,
    /// The extension has the position request; show the window now.
    Present,
    FocusChanged(bool),
    /// GNOME Shell (desktop background, panel) was clicked.
    ShellPressed,
    /// The daemon saw a left double click somewhere.
    DoubleClicked,
    /// The extension's answer to `PointerOnDesktop`.
    PointerOnDesktop(bool),
    /// The daemon saw a right press while left was held.
    LeftRightClicked,
    /// The daemon or the extension (re)appeared on the bus.
    ServicesAppeared,
    SetPinned(bool),
    /// Opens the settings window, at the General page for `None`.
    OpenSettings(Option<SettingsTarget>),
    CheckUpdates,
    Quit,
    /// A tab button was clicked.
    SelectTab(Tab),
    SearchChanged(String),
    /// A navigation key, routed by [`Nav`] according to the current focus.
    Key(Key),
    /// GTK focus moved (mouse click or Tab key).
    FocusMoved(Focus),
    FocusSearch,
    /// The search bar was revealed (typing in minimal).
    SearchRevealed,
    /// A row was clicked; visible index in the current tab.
    Activate(usize),
    /// Context menu of a visible row, at this point in the list (`None`:
    /// below the selected row). Index 0 on an empty list.
    OpenMenu(usize, Option<ui::MenuAnchor>),
    MenuClosed,
    /// Shortly after the menu closed: hide if focus went to another window.
    MenuSettled,
    /// Visible Clipboard row to add to snippets.
    AddSnippet(usize),
    /// Visible row of the current tab to delete.
    DeleteRow(usize),
    /// Visible Snippets row to open in the settings editor.
    EditSnippet(usize),
    NewSnippet,
    /// Visible Snippets row and how far to move it.
    MoveSnippet(usize, isize),
    /// A Launcher row at this depth (0: the tab's list) was clicked.
    TreeClick(usize, usize),
    /// The pointer moved onto a Launcher row at this depth.
    TreeHover(usize, usize),
    /// The pointer left the rows of this Launcher depth.
    TreeLeave(usize),
    /// The hover delay of this token passed.
    TreeHoverElapsed(u64),
    /// New clipboard text (extension signal or read on focus).
    ClipboardText(String),
    /// Installed applications changed.
    AppsChanged,
    Setting(settings::Event),
    /// Development aid: `UTI_DEBUG_SNAPSHOT` after a `debug-key`.
    DebugSnapshot,
    /// Rebuild the settings window (language changed).
    RepopulateSettings,
    SettingsClosed,
}

/// An open Launcher level below the tab's list.
struct CascadeLevel {
    popover: gtk::Popover,
    list: gtk::ListBox,
    entries: Vec<Entry>,
}

type Sender = async_channel::Sender<Msg>;

fn send(tx: &Sender, msg: Msg) {
    // Unbounded: only fails once the model task has ended (app quitting).
    let _ = tx.try_send(msg);
}

/// Search query and visible rows of one tab.
#[derive(Default)]
struct TabData {
    query: String,
    /// Indices into the backing store for the visible rows.
    visible: Vec<usize>,
}

/// Widgets the model updates directly.
struct Ui {
    root: adw::ApplicationWindow,
    stack: gtk::Stack,
    tab_group: adw::ToggleGroup,
    search: gtk::SearchEntry,
    search_bar: gtk::SearchBar,
    toolbar: adw::ToolbarView,
    lists: HashMap<Tab, gtk::ListBox>,
    pin: gtk::ToggleButton,
    footer: gtk::Label,
    /// Row context menu, parented to `stack`.
    menu: gtk::PopoverMenu,
}

/// Main window model.
struct App {
    tx: Sender,
    ui: Ui,
    window: WindowState,
    nav: Nav,
    conn: Option<gio::DBusConnection>,
    _subscriptions: Vec<gio::SignalSubscription>,
    _watchers: Watchers,
    tabs: HashMap<Tab, TabData>,
    history: ClipboardStore,
    snippets: SnippetsStore,
    /// Window height already trimmed to whole rows.
    rows_fitted: bool,
    launcher: Vec<LauncherNode>,
    /// Installed apps shown in launcher search, loaded once and refreshed
    /// by `AppInfoMonitor` (no rescans per keystroke).
    apps: Vec<gio::AppInfo>,
    /// Command line of each app (field codes removed), for command icons.
    app_argv: Vec<Vec<String>>,
    /// Rows of the Launcher tab's list.
    launcher_rows: Vec<Entry>,
    /// Open Launcher levels, mirroring `nav.tree()` depths 1..
    cascade: Vec<CascadeLevel>,
    config: AppConfig,
    settings: Option<settings::SettingsUi>,
}

/// Installed apps that should appear in launchers.
fn load_apps() -> Vec<gio::AppInfo> {
    gio::AppInfo::all().into_iter().filter(|a| a.should_show()).collect()
}

/// Each app's command line without `%` field codes.
fn app_argv(apps: &[gio::AppInfo]) -> Vec<Vec<String>> {
    apps.iter()
        .map(|a| {
            a.commandline()
                .and_then(|c| glib::shell_parse_argv(c).ok())
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.to_string_lossy().into_owned())
                .filter(|s| !s.starts_with('%'))
                .collect()
        })
        .collect()
}

/// Bus name watchers, unwatched on drop.
struct Watchers(Option<[gio::WatcherId; 2]>);

impl Drop for Watchers {
    fn drop(&mut self) {
        for id in self.0.take().into_iter().flatten() {
            gio::bus_unwatch_name(id);
        }
    }
}

/// Builds the main window and starts the message loop.
pub fn start(app: &adw::Application, minimized: bool) {
    let (tx, rx) = async_channel::unbounded();
    let config = AppConfig::load();
    let ui = build_ui(app, &tx, Size::parse(&config.theme.size));
    register_actions(app, &tx);

    let conn = app.dbus_connection();
    let mut subscriptions = Vec::new();
    if let Some(conn) = &conn {
        let t = tx.clone();
        subscriptions.push(dbus::on_triggered(conn, move || send(&t, Msg::Toggle)));
        let t = tx.clone();
        subscriptions.push(dbus::on_double_clicked(conn, move || send(&t, Msg::DoubleClicked)));
        let t = tx.clone();
        subscriptions
            .push(dbus::on_left_right_clicked(conn, move || send(&t, Msg::LeftRightClicked)));
        let t = tx.clone();
        subscriptions
            .push(dbus::on_clipboard_changed(conn, move |text| send(&t, Msg::ClipboardText(text))));
        let t = tx.clone();
        subscriptions.push(dbus::on_shell_pressed(conn, move || send(&t, Msg::ShellPressed)));
    } else {
        eprintln!("No D-Bus connection: double Ctrl and extension features disabled");
    }

    let mut history = ClipboardStore::load_default();
    history.max_items = config.clipboard_history_limit;

    let mut window = WindowState::new();
    window.set_desktop_double_click(config.mouse.desktop_double_click);
    window.set_left_right_click(config.mouse.left_right_click);
    let watchers = conn.as_ref().map(|conn| {
        let t = tx.clone();
        dbus::watch_services(conn, move || send(&t, Msg::ServicesAppeared))
    });
    let apps = load_apps();
    let mut model = App {
        tx: tx.clone(),
        ui,
        window,
        nav: Nav::new(),
        conn,
        _subscriptions: subscriptions,
        _watchers: Watchers(watchers),
        tabs: Tab::ALL.iter().map(|t| (*t, TabData::default())).collect(),
        history,
        snippets: load_snippets(),
        rows_fitted: false,
        launcher: load_launcher_tree(),
        app_argv: app_argv(&apps),
        apps,
        launcher_rows: Vec::new(),
        cascade: Vec::new(),
        config,
        settings: None,
    };
    let t = tx.clone();
    gio::AppInfoMonitor::get().connect_changed(move |_| send(&t, Msg::AppsChanged));
    model.apply_scrollbar();
    for tab in Tab::ALL {
        model.refresh(tab);
    }
    model.show_tab(model.nav.tab());

    if !minimized {
        send(&tx, Msg::Show);
    }
    glib::spawn_future_local(async move {
        while let Ok(msg) = rx.recv().await {
            model.update(msg);
        }
    });
}

impl App {
    fn update(&mut self, msg: Msg) {
        if debug_keys()
            && !matches!(msg, Msg::ClipboardText(_) | Msg::SearchChanged(_) | Msg::Key(_))
        {
            eprintln!(
                "DEBUG msg={msg:?} active={} nav_focus={:?}",
                self.ui.root.is_active(),
                self.nav.focus()
            );
        }
        let command = match msg {
            Msg::Toggle => self.window.toggle(Instant::now()),
            Msg::Show => self.window.show(Instant::now()),
            Msg::Hide => self.window.hide(),
            Msg::FocusChanged(false) => {
                // Pinned: a click on another window closes the open levels.
                if self.ui.pin.is_active() {
                    self.close_cascade();
                }
                self.window.focus_lost()
            },
            Msg::ShellPressed => self.window.shell_pressed(Instant::now()),
            Msg::DoubleClicked => self.window.double_clicked(),
            Msg::PointerOnDesktop(on) => self.window.pointer_on_desktop(on, Instant::now()),
            Msg::LeftRightClicked => self.window.left_right_clicked(Instant::now()),
            Msg::ServicesAppeared => {
                self.sync_left_right_click();
                None
            },
            Msg::FocusChanged(true) => {
                self.read_clipboard();
                None
            },
            Msg::SetPinned(pinned) => self.window.set_pinned(pinned),
            Msg::Present => {
                self.ui.root.present();
                self.sync_focus();
                if !self.rows_fitted {
                    self.rows_fitted = true;
                    let (root, lists, tab) =
                        (self.ui.root.clone(), self.ui.lists.clone(), self.nav.tab());
                    // After the first layout, so rows have their real size.
                    glib::idle_add_local_once(move || fit_to_rows(&root, &lists, tab));
                }
                debug_snapshot(&self.ui.root, &[]);
                None
            },
            Msg::OpenSettings(target) => {
                self.open_settings(target.unwrap_or_default());
                None
            },
            Msg::Setting(event) => {
                self.apply_setting(event);
                None
            },
            Msg::RepopulateSettings => {
                self.repopulate_settings();
                None
            },
            Msg::DebugSnapshot => {
                let popovers: Vec<gtk::Popover> =
                    self.cascade.iter().map(|l| l.popover.clone()).collect();
                debug_snapshot(&self.ui.root, &popovers);
                None
            },
            Msg::SettingsClosed => {
                self.settings = None;
                None
            },
            Msg::CheckUpdates => {
                open_software_updates(&self.ui.root);
                None
            },
            Msg::Quit => {
                if let Some(app) = self.ui.root.application() {
                    app.quit();
                }
                None
            },
            Msg::SelectTab(tab) => {
                let effect = self.nav.select_tab(tab);
                self.navigate(effect)
            },
            Msg::SearchChanged(query) => {
                let tab = self.nav.tab();
                if let Some(data) = self.tabs.get_mut(&tab) {
                    if data.query != query {
                        data.query = query;
                        self.refresh(tab);
                    }
                }
                self.sync_search_bar();
                None
            },
            Msg::Key(key) => {
                let effect = self.nav.on_key(key);
                let command = self.navigate(effect);
                if debug_keys() {
                    let focused = gtk::prelude::GtkWindowExt::focus(&self.ui.root)
                        .map(|w| w.type_().name().to_string());
                    eprintln!(
                        "DEBUG key={key:?} tab={:?} nav_focus={:?} gtk_focus={focused:?} pinned={}",
                        self.nav.tab(),
                        self.nav.focus(),
                        self.ui.pin.is_active()
                    );
                }
                command
            },
            Msg::FocusMoved(focus) => {
                self.nav.focus_moved(focus);
                self.sync_search_bar();
                None
            },
            Msg::FocusSearch => {
                self.nav.focus_search();
                self.sync_focus();
                None
            },
            Msg::SearchRevealed => {
                // The bar moves focus to the entry itself; grabbing it again
                // would select the typed text.
                self.nav.focus_search();
                None
            },
            Msg::Activate(i) => self.activate(i),
            Msg::OpenMenu(i, point) => {
                self.open_menu(i, point);
                None
            },
            Msg::MenuClosed => {
                self.nav.menu_closed();
                self.sync_focus();
                // Focus comes back to the window only after the popup is gone.
                let t = self.tx.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(150), move || {
                    send(&t, Msg::MenuSettled)
                });
                None
            },
            Msg::MenuSettled if self.ui.menu.is_visible() => None,
            Msg::MenuSettled => self.window.popover_closed(self.ui.root.is_active()),
            Msg::AddSnippet(i) => {
                self.add_snippet(i);
                None
            },
            Msg::DeleteRow(i) => {
                self.delete_row(i);
                None
            },
            Msg::EditSnippet(i) => {
                self.edit_snippet(i);
                None
            },
            Msg::NewSnippet => {
                self.open_settings(SettingsTarget::snippet(SnippetTarget::New));
                None
            },
            Msg::MoveSnippet(i, delta) => {
                self.move_snippet(i, delta);
                None
            },
            Msg::ClipboardText(text) => {
                self.add_history(text);
                None
            },
            Msg::AppsChanged => {
                self.apps = load_apps();
                self.app_argv = app_argv(&self.apps);
                self.refresh(Tab::Launcher);
                None
            },
            Msg::TreeClick(depth, index) => {
                if depth == 0 {
                    self.nav.focus_moved(Focus::List);
                }
                let action = self.nav.tree_mut().click(depth, index);
                self.tree_action(action)
            },
            Msg::TreeHover(depth, index) => {
                if let Some(token) = self.nav.tree_mut().hover(depth, index) {
                    let t = self.tx.clone();
                    glib::timeout_add_local_once(HOVER_DELAY, move || {
                        send(&t, Msg::TreeHoverElapsed(token))
                    });
                }
                self.sync_tree();
                None
            },
            Msg::TreeLeave(depth) => {
                self.nav.tree_mut().leave(depth);
                None
            },
            Msg::TreeHoverElapsed(token) => {
                let action = self.nav.tree_mut().hover_elapsed(token);
                self.tree_action(action)
            },
        };
        if let Some(command) = command {
            if debug_keys() {
                eprintln!("DEBUG command={command:?}");
            }
            self.apply(command);
        }
    }

    /// Applies a navigation effect, then moves GTK focus to [`Nav::focus`].
    fn navigate(&mut self, effect: Option<Effect>) -> Option<WindowCommand> {
        let command = match effect {
            Some(Effect::ShowTab(tab)) => {
                self.show_tab(tab);
                None
            },
            Some(Effect::Activate(i)) => self.activate(i),
            Some(Effect::AddSnippet(i)) => {
                self.add_snippet(i);
                None
            },
            Some(Effect::OpenMenu(i)) => {
                self.open_menu(i, None);
                None
            },
            Some(Effect::Delete(i)) => {
                self.delete_row(i);
                None
            },
            Some(Effect::Edit(i)) => {
                self.edit_snippet(i);
                None
            },
            Some(Effect::ClearQuery) => {
                self.ui.search.set_text("");
                None
            },
            Some(Effect::TogglePin) => self.window.set_pinned(!self.ui.pin.is_active()),
            Some(Effect::TreeOpen {
                depth,
                index,
                enter,
            }) => {
                self.open_level(depth, index, enter);
                None
            },
            Some(Effect::TreeActivate { depth, index }) => self.activate_entry(depth, index),
            // Selections are applied by sync_tree / sync_focus.
            Some(Effect::SelectRow(_) | Effect::TreeChanged) | None => None,
        };
        self.sync_tree();
        self.sync_focus();
        command
    }

    /// Applies a tree action from pointer input, then mirrors the tree.
    fn tree_action(&mut self, action: Option<TreeAction>) -> Option<WindowCommand> {
        let command = match action {
            Some(TreeAction::Open {
                depth,
                index,
                enter,
            }) => {
                self.open_level(depth, index, enter);
                None
            },
            Some(TreeAction::Activate { depth, index }) => self.activate_entry(depth, index),
            Some(TreeAction::Unhandled) | None => None,
        };
        self.sync_tree();
        command
    }

    /// Rows of Launcher level `depth` (0: the tab's list).
    fn level_entries(&self, depth: usize) -> Option<&[Entry]> {
        match depth {
            0 => Some(&self.launcher_rows),
            d => self.cascade.get(d - 1).map(|l| l.entries.as_slice()),
        }
    }

    fn level_list(&self, depth: usize) -> Option<&gtk::ListBox> {
        match depth {
            0 => self.ui.lists.get(&Tab::Launcher),
            d => self.cascade.get(d - 1).map(|l| &l.list),
        }
    }

    /// Opens the children of row `index` at `depth` beside it.
    fn open_level(&mut self, depth: usize, index: usize, enter: bool) {
        let Some(entry) = self.level_entries(depth).and_then(|e| e.get(index)).cloned() else {
            return;
        };
        let Some(parent) = self.level_list(depth).cloned() else {
            return;
        };
        let Some(row) = parent.row_at_index(index as i32) else {
            return;
        };
        self.truncate_cascade(depth);
        let entries = entry::children(&self.launcher, &entry);
        let empty = if entry::folder_children(&self.launcher, &entry).is_some() {
            "Empty"
        } else {
            "No recent files"
        };
        let (popover, list) = ui::cascade_popover(&parent);
        self.fill_level(&list, &entries, false);
        if entries.is_empty() {
            list.append(&ui::empty_row(empty));
        }
        self.style_scrolled(&list);
        let t = self.tx.clone();
        let d = depth + 1;
        list.connect_row_activated(move |_, r| send(&t, Msg::TreeClick(d, r.index() as usize)));
        track_pointer(&list, d, &self.tx);
        if let Some(b) = row.compute_bounds(&parent) {
            popover.set_pointing_to(Some(&gdk::Rectangle::new(
                0,
                b.y() as i32,
                parent.width(),
                b.height() as i32,
            )));
        }
        let rows = entries.iter().map(|e| e.row(&self.launcher)).collect();
        self.nav.tree_mut().push(depth, index, rows, enter);
        popover.popup();
        self.cascade.push(CascadeLevel {
            popover,
            list,
            entries,
        });
    }

    /// Closes the popovers of levels deeper than `depth`.
    fn truncate_cascade(&mut self, depth: usize) {
        while self.cascade.len() > depth {
            if let Some(level) = self.cascade.pop() {
                level.popover.popdown();
                level.popover.unparent();
            }
        }
    }

    fn close_cascade(&mut self) {
        self.nav.tree_mut().close_all();
        self.sync_tree();
    }

    /// Mirrors the Launcher tree: open popovers, selections, open path.
    fn sync_tree(&mut self) {
        let tree = self.nav.tree();
        let depth = tree.depth();
        let active = tree.active();
        let selected: Vec<Option<usize>> = (0..=depth).map(|d| tree.selected(d)).collect();
        let owners: Vec<Option<usize>> = (1..=depth + 1).map(|d| tree.owner(d)).collect();
        self.truncate_cascade(depth);
        for d in 0..=depth {
            let Some(list) = self.level_list(d) else {
                continue;
            };
            if d == active {
                list.remove_css_class("uti-tree-inactive");
            } else {
                list.add_css_class("uti-tree-inactive");
            }
            match selected[d].and_then(|i| list.row_at_index(i as i32)) {
                Some(row) => list.select_row(Some(&row)),
                None => list.unselect_all(),
            }
            let mut i = 0;
            while let Some(row) = list.row_at_index(i) {
                if owners[d] == Some(i as usize) {
                    row.add_css_class("uti-path");
                } else {
                    row.remove_css_class("uti-path");
                }
                i += 1;
            }
        }
        if self.nav.tab() == Tab::Launcher {
            self.ui.footer.set_markup(if depth > 0 {
                ui::CASCADE_HINTS
            } else {
                ui::key_hints(Tab::Launcher)
            });
        }
    }

    /// Puts GTK focus where [`Nav`] says it is.
    fn sync_focus(&self) {
        // Focus is moved by our own key handling, and GTK only draws focus
        // rings after its own keyboard navigation; keep them on (the popup
        // is keyboard-driven).
        self.ui.root.set_focus_visible(true);
        // Before focusing: a hidden search entry cannot take focus.
        self.sync_search_bar();
        match self.nav.focus() {
            // The popover holds focus while it is open.
            Focus::Menu => {},
            Focus::Search => {
                self.ui.search.grab_focus();
            },
            Focus::Pin => {
                self.ui.pin.grab_focus();
            },
            Focus::List => {
                let Some(listbox) = self.ui.lists.get(&self.nav.tab()) else {
                    return;
                };
                match listbox.row_at_index(self.nav.selected() as i32) {
                    Some(row) => {
                        listbox.select_row(Some(&row));
                        row.grab_focus();
                    },
                    // Empty list: focus the list itself. Keys still reach the
                    // window-level handler even if nothing takes focus.
                    None => {
                        if !listbox.grab_focus() {
                            gtk::prelude::GtkWindowExt::set_focus(
                                &self.ui.root,
                                None::<&gtk::Widget>,
                            );
                        }
                    },
                }
            },
        }
    }

    /// Tells the daemon and the extension whether left-held + right click is on.
    fn sync_left_right_click(&self) {
        if let Some(conn) = self.conn.clone() {
            let enabled = self.config.mouse.left_right_click;
            glib::spawn_future_local(
                async move { dbus::set_left_right_click(&conn, enabled).await },
            );
        }
    }

    fn position_and_present(&mut self) {
        match self.conn.clone() {
            Some(conn) => {
                let tx = self.tx.clone();
                glib::spawn_future_local(async move {
                    dbus::position_at_pointer(&conn).await;
                    send(&tx, Msg::Present);
                });
            },
            None => {
                self.ui.root.present();
                self.sync_focus();
            },
        }
    }

    /// Shows or hides the search bar for `theme.size`.
    fn sync_search_bar(&self) {
        let has_query = self.tabs.get(&self.nav.tab()).is_some_and(|d| !d.query.is_empty());
        let show = self.size().show_search(self.nav.focus() == Focus::Search, has_query);
        if self.ui.search_bar.is_search_mode() != show {
            self.ui.search_bar.set_search_mode(show);
        }
    }

    fn size(&self) -> Size {
        Size::parse(&self.config.theme.size)
    }

    fn apply(&mut self, command: WindowCommand) {
        match command {
            WindowCommand::PositionAndPresent(tab) => {
                let effect = self.nav.select_tab(tab);
                self.navigate(effect);
                self.position_and_present();
            },
            WindowCommand::QueryPointerOnDesktop => {
                if let Some(conn) = self.conn.clone() {
                    let tx = self.tx.clone();
                    glib::spawn_future_local(async move {
                        let on = dbus::pointer_on_desktop(&conn).await;
                        send(&tx, Msg::PointerOnDesktop(on));
                    });
                }
            },
            WindowCommand::Hide => {
                self.ui.menu.popdown();
                self.close_cascade();
                self.ui.root.set_visible(false);
            },
            WindowCommand::SetAbove(above) => {
                if self.ui.pin.is_active() != above {
                    self.ui.pin.set_active(above);
                }
                if let Some(conn) = self.conn.clone() {
                    glib::spawn_future_local(async move { dbus::set_above(&conn, above).await });
                }
            },
        }
    }

    /// Copies the chosen item and hides the window.
    fn activate(&mut self, visible_index: usize) -> Option<WindowCommand> {
        let tab = self.nav.tab();
        let source = *self.tabs.get(&tab)?.visible.get(visible_index)?;
        let text = match tab {
            Tab::Clipboard => self.history.items.get(source)?.text.clone(),
            Tab::Snippets => self.snippets.items.get(source)?.value.clone(),
            Tab::Launcher => return self.activate_entry(0, visible_index),
        };
        self.ui.root.clipboard().set_text(&text);
        self.window.hide()
    }

    /// Source index of visible row `visible_index` of `tab`.
    fn source(&self, tab: Tab, visible_index: usize) -> Option<usize> {
        self.tabs.get(&tab)?.visible.get(visible_index).copied()
    }

    /// Shows the context menu of a visible row of the current tab.
    fn open_menu(&mut self, visible_index: usize, anchor: Option<ui::MenuAnchor>) {
        let tab = self.nav.tab();
        let len = self.tabs.get(&tab).map_or(0, |d| d.visible.len());
        let sections = row_menu(tab, visible_index, len);
        let Some(listbox) = self.ui.lists.get(&tab) else {
            return;
        };
        if sections.is_empty() {
            return;
        }
        self.nav.set_selected(tab, visible_index);
        self.sync_focus();
        let row = listbox.row_at_index(visible_index as i32);
        // The given anchor, or below the start of the row (keyboard).
        let anchor = anchor.unwrap_or_else(|| {
            let (x, y) = row
                .as_ref()
                .and_then(|r| r.compute_bounds(listbox))
                .map_or((0.0, 0.0), |b| (f64::from(b.x() + 24.0), f64::from(b.y() + b.height())));
            ui::MenuAnchor { x, y, end: false }
        });
        let Some(at) = listbox.compute_point(
            &self.ui.stack,
            &gtk::graphene::Point::new(anchor.x as f32, anchor.y as f32),
        ) else {
            return;
        };
        // A popover's halign sets which side of `pointing_to` it starts at.
        self.ui.menu.set_halign(if anchor.end {
            gtk::Align::End
        } else {
            gtk::Align::Start
        });
        self.ui.menu.set_menu_model(Some(&ui::menu_model(&sections, visible_index)));
        self.ui.menu.set_pointing_to(Some(&gdk::Rectangle::new(
            at.x() as i32,
            at.y() as i32,
            1,
            1,
        )));
        self.nav.menu_opened();
        self.window.popover_opened();
        self.ui.menu.popup();
        // GtkMenuSectionBox shows its separators from an idle callback, after
        // the popup size is fixed; without a second layout the menu scrolls.
        let menu = self.ui.menu.clone();
        glib::idle_add_local_once(move || {
            menu.queue_resize();
            menu.present();
        });
    }

    /// Adds a visible Clipboard row to snippets; the history item stays.
    fn add_snippet(&mut self, visible_index: usize) {
        let Some(item) = self
            .source(Tab::Clipboard, visible_index)
            .and_then(|i| self.history.items.get(i))
        else {
            return;
        };
        self.snippets.add(item.text.clone(), None);
        self.snippets_changed();
    }

    /// Deletes a visible row of the current tab (history item or snippet).
    fn delete_row(&mut self, visible_index: usize) {
        let tab = self.nav.tab();
        let Some(source) = self.source(tab, visible_index) else {
            return;
        };
        match tab {
            Tab::Clipboard => {
                let Some(id) = self.history.items.get(source).map(|i| i.id.clone()) else {
                    return;
                };
                self.history.remove_by_id(&id);
                self.save_history();
                self.refresh(Tab::Clipboard);
            },
            Tab::Snippets => {
                let Some(id) = self.snippets.items.get(source).map(|i| i.id.clone()) else {
                    return;
                };
                self.snippets.remove(&id);
                self.snippets_changed();
            },
            Tab::Launcher => return,
        }
        self.nav.set_selected(tab, visible_index);
        self.sync_focus();
    }

    /// Opens a visible Snippets row in the settings editor.
    fn edit_snippet(&mut self, visible_index: usize) {
        let Some(id) = self
            .source(Tab::Snippets, visible_index)
            .and_then(|i| self.snippets.items.get(i))
            .map(|i| i.id.clone())
        else {
            return;
        };
        self.open_settings(SettingsTarget::snippet(SnippetTarget::Edit(id)));
    }

    /// Moves a visible Snippets row and keeps it selected.
    fn move_snippet(&mut self, visible_index: usize, delta: isize) {
        let Some(id) = self
            .source(Tab::Snippets, visible_index)
            .and_then(|i| self.snippets.items.get(i))
            .map(|i| i.id.clone())
        else {
            return;
        };
        if !self.snippets.move_by(&id, delta) {
            return;
        }
        self.snippets_changed();
        let moved = self.snippets.items.iter().position(|i| i.id == id);
        let visible = self
            .tabs
            .get(&Tab::Snippets)
            .and_then(|d| d.visible.iter().position(|&i| Some(i) == moved));
        if let Some(v) = visible {
            self.nav.set_selected(Tab::Snippets, v);
            self.sync_focus();
        }
    }

    /// Saves snippets and updates the Snippets tab and the settings window.
    fn snippets_changed(&mut self) {
        if let Err(e) = save_snippets(&self.snippets) {
            eprintln!("Failed to save snippets: {e}");
        }
        self.refresh(Tab::Snippets);
        if let Some(settings) = &self.settings {
            settings.set_snippets(&self.snippets);
        }
    }

    fn add_history(&mut self, text: String) {
        if text.trim().is_empty() || self.history.items.first().is_some_and(|i| i.text == text) {
            return;
        }
        self.history.add(text);
        self.save_history();
        self.refresh(Tab::Clipboard);
    }

    fn save_history(&self) {
        if let Err(e) = self.history.save(&ClipboardStore::get_storage_path()) {
            eprintln!("Failed to save clipboard history: {e}");
        }
    }

    /// Fallback when the extension is absent: read the clipboard on focus.
    fn read_clipboard(&self) {
        let clipboard = self.ui.root.clipboard();
        let tx = self.tx.clone();
        glib::spawn_future_local(async move {
            if let Ok(Some(text)) = clipboard.read_text_future().await {
                send(&tx, Msg::ClipboardText(text.to_string()));
            }
        });
    }

    /// Launches Launcher row `index` at `depth` and hides the window.
    fn activate_entry(&mut self, depth: usize, index: usize) -> Option<WindowCommand> {
        let entry = self.level_entries(depth)?.get(index)?.clone();
        match entry {
            Entry::Node(path) => {
                let item = entry::command_at(&self.launcher, &path)?;
                spawn(&item.command, &item.args, None);
            },
            Entry::Recent { command, file } => {
                let item = entry::command_at(&self.launcher, &command)?;
                spawn(&item.command, &item.args, Some(&file.path));
            },
            Entry::App(a) => {
                let app = self.apps.get(a)?;
                let context = gtk::prelude::WidgetExt::display(&self.ui.root).app_launch_context();
                if dev_sandbox() {
                    for var in SANDBOX_VARS {
                        context.unsetenv(var);
                    }
                }
                if let Err(e) = app.launch(&[], Some(&context)) {
                    eprintln!("Failed to launch {}: {e}", app.name());
                }
            },
        }
        self.close_cascade();
        self.window.hide()
    }

    /// Shows the settings window at `target`, creating it on first use.
    fn open_settings(&mut self, target: SettingsTarget) {
        if self.settings.is_none() {
            let Some(app) = self.ui.root.application().and_downcast::<adw::Application>() else {
                return;
            };
            let window = settings::new_window(&app);
            let t = self.tx.clone();
            window.connect_close_request(move |_| {
                send(&t, Msg::SettingsClosed);
                glib::Propagation::Proceed
            });
            self.settings = Some(self.populate_settings(&window, target.page));
        }
        if let Some(ui) = &self.settings {
            ui.open(&target, &self.snippets);
            ui.window.present();
            debug_snapshot(&ui.window, &[]);
        }
    }

    fn populate_settings(
        &self,
        window: &adw::ApplicationWindow,
        page: crate::state::settings::SettingsPage,
    ) -> settings::SettingsUi {
        let t = self.tx.clone();
        settings::populate(
            window,
            &self.config,
            &self.snippets,
            page,
            std::rc::Rc::new(move |e| send(&t, Msg::Setting(e))),
        )
    }

    /// Rebuilds the settings window (language or reload), on the same page.
    fn repopulate_settings(&mut self) {
        if let Some(ui) = self.settings.take() {
            self.settings = Some(self.populate_settings(&ui.window, ui.page()));
        }
    }

    /// Applies and saves a settings change.
    fn apply_setting(&mut self, event: settings::Event) {
        match event {
            settings::Event::Language(lang) => {
                if lang == self.config.language {
                    return;
                }
                self.config.language = lang;
                // Rebuilt after the combo's handler returns.
                let t = self.tx.clone();
                glib::idle_add_local_once(move || send(&t, Msg::RepopulateSettings));
            },
            settings::Event::Size(size) => {
                self.config.theme.size = size;
                self.apply_size();
            },
            settings::Event::Scrollbar(scrollbar) => {
                self.config.theme.scrollbar = scrollbar;
                self.apply_scrollbar();
            },
            settings::Event::HistoryLimit(limit) => {
                // Trimmed on the next add, like the startup sync.
                self.config.clipboard_history_limit = limit.max(1);
                self.history.max_items = self.config.clipboard_history_limit;
            },
            settings::Event::DesktopDoubleClick(enabled) => {
                self.config.mouse.desktop_double_click = enabled;
                self.window.set_desktop_double_click(enabled);
            },
            settings::Event::LeftRightClick(enabled) => {
                self.config.mouse.left_right_click = enabled;
                self.window.set_left_right_click(enabled);
                self.sync_left_right_click();
            },
            settings::Event::SnippetSave { id, label, value } => {
                match id {
                    Some(id) => {
                        self.snippets.update(&id, label, value);
                    },
                    None => {
                        self.snippets.add(value, label);
                    },
                }
                self.snippets_changed();
                return;
            },
            settings::Event::SnippetDelete(id) => {
                self.snippets.remove(&id);
                self.snippets_changed();
                return;
            },
            settings::Event::SnippetMove(id, delta) => {
                if self.snippets.move_by(&id, delta) {
                    self.snippets_changed();
                }
                return;
            },
            settings::Event::Reload => {
                self.config = AppConfig::load();
                self.window.set_desktop_double_click(self.config.mouse.desktop_double_click);
                self.window.set_left_right_click(self.config.mouse.left_right_click);
                self.sync_left_right_click();
                self.history.max_items = self.config.clipboard_history_limit;
                self.apply_size();
                self.apply_scrollbar();
                self.launcher = load_launcher_tree();
                self.snippets = load_snippets();
                for tab in Tab::ALL {
                    self.refresh(tab);
                }
                self.repopulate_settings();
                return; // Nothing changed on disk.
            },
        }
        if let Err(e) = self.config.save() {
            eprintln!("{e}");
        }
    }

    /// Resizes the main window for `theme.size`.
    fn apply_size(&mut self) {
        let size = self.size();
        for other in Size::ALL {
            self.ui.root.remove_css_class(other.css_class());
        }
        self.ui.root.add_css_class(size.css_class());
        let (width, height) = size.window_size();
        self.ui.root.set_default_size(width, height);
        self.ui.toolbar.set_reveal_bottom_bars(size.show_footer());
        self.sync_search_bar();
        // Trim again on the next show, with the new row height.
        self.rows_fitted = false;
    }

    /// Applies `theme.scrollbar` to every list.
    fn apply_scrollbar(&self) {
        let style = Scrollbar::parse(&self.config.theme.scrollbar);
        for other in Scrollbar::ALL {
            self.ui.root.remove_css_class(other.css_class());
        }
        self.ui.root.add_css_class(style.css_class());
        let lists = self.ui.lists.values().chain(self.cascade.iter().map(|l| &l.list));
        for list in lists {
            self.style_scrolled(list);
        }
    }

    /// Applies `theme.scrollbar` to the scrolled window around `list`.
    fn style_scrolled(&self, list: &gtk::ListBox) {
        let style = Scrollbar::parse(&self.config.theme.scrollbar);
        let Some(pane) = list
            .ancestor(gtk::ScrolledWindow::static_type())
            .and_downcast::<gtk::ScrolledWindow>()
        else {
            return;
        };
        pane.set_vscrollbar_policy(if style.visible() {
            gtk::PolicyType::Automatic
        } else {
            // Scrolls without a scrollbar.
            gtk::PolicyType::External
        });
        pane.set_overlay_scrolling(style.overlays());
    }

    /// Shows `tab`'s page, tab button state, search text and footer.
    fn show_tab(&self, tab: Tab) {
        let name = ui::page_name(tab);
        self.ui.stack.set_visible_child_name(name);
        if self.ui.tab_group.active_name().as_deref() != Some(name) {
            self.ui.tab_group.set_active_name(Some(name));
        }
        self.ui.footer.set_markup(ui::key_hints(tab));
        self.ui.search.set_placeholder_text(Some(ui::search_placeholder(tab)));
        let query = self.tabs.get(&tab).map(|d| d.query.as_str()).unwrap_or_default();
        if self.ui.search.text() != query {
            self.ui.search.set_text(query);
        }
    }

    /// Recomputes the visible rows of `tab` and re-renders its list.
    fn refresh(&mut self, tab: Tab) {
        if tab == Tab::Launcher {
            self.refresh_launcher();
            return;
        }
        let Some(data) = self.tabs.get_mut(&tab) else {
            return;
        };
        let Some(listbox) = self.ui.lists.get(&tab) else {
            return;
        };
        let rows: Vec<(String, String)> = match tab {
            Tab::Clipboard => {
                let texts = self.history.items.iter().map(|i| i.text.as_str());
                data.visible = list::filter(texts, &data.query);
                data.visible
                    .iter()
                    .map(|&i| {
                        let item = &self.history.items[i];
                        (item.text.clone(), item.text.clone())
                    })
                    .collect()
            },
            Tab::Snippets => {
                let haystacks: Vec<String> = self
                    .snippets
                    .items
                    .iter()
                    .map(|s| format!("{}\n{}", s.label.as_deref().unwrap_or_default(), s.value))
                    .collect();
                data.visible = list::filter(haystacks.iter().map(String::as_str), &data.query);
                data.visible
                    .iter()
                    .map(|&i| {
                        let s = &self.snippets.items[i];
                        let label =
                            s.label.as_deref().filter(|l| !l.trim().is_empty()).unwrap_or(&s.value);
                        (label.to_owned(), s.value.clone())
                    })
                    .collect()
            },
            Tab::Launcher => unreachable!("handled above"),
        };
        self.nav.set_len(tab, rows.len());
        listbox.remove_all();
        for (index, (text, tooltip)) in rows.iter().enumerate() {
            let spec = ui::RowSpec {
                index,
                hint: (index < 9).then_some(index + 1),
                icon: None,
                text,
                tooltip,
                branch: false,
                more: true,
            };
            let tx = self.tx.clone();
            listbox
                .append(&ui::build_row(&spec, move |i, point| send(&tx, Msg::OpenMenu(i, point))));
        }
        if self.nav.tab() == tab && self.ui.root.is_visible() {
            self.sync_focus();
        }
    }

    /// Recomputes the Launcher tab's rows: the tree's root, or while
    /// searching, matching commands of the whole tree and installed apps.
    /// Closes the open levels.
    fn refresh_launcher(&mut self) {
        let query = self.tabs.get(&Tab::Launcher).map(|d| d.query.clone()).unwrap_or_default();
        let searching = !query.trim().is_empty();
        self.launcher_rows = if searching {
            let commands = entry::searchable(&self.launcher);
            let apps: Vec<(String, String)> = self
                .apps
                .iter()
                .map(|a| {
                    let desc = a.description().map(|d| d.to_string()).unwrap_or_default();
                    (a.name().to_string(), desc)
                })
                .collect();
            let texts: Vec<SearchText<'_>> = commands
                .iter()
                .map(|c| SearchText {
                    name: &c.command.name,
                    description: &c.command.command,
                })
                .chain(apps.iter().map(|(name, description)| SearchText { name, description }))
                .collect();
            rank(&texts, &query)
                .into_iter()
                .map(|i| match commands.get(i) {
                    Some(c) => Entry::Node(c.path.clone()),
                    None => Entry::App(i - commands.len()),
                })
                .collect()
        } else {
            entry::root_entries(&self.launcher)
        };
        if let Some(data) = self.tabs.get_mut(&Tab::Launcher) {
            data.visible = (0..self.launcher_rows.len()).collect();
        }
        let rows = self.launcher_rows.iter().map(|e| e.row(&self.launcher)).collect();
        self.nav.set_launcher_rows(rows);
        if let Some(list) = self.ui.lists.get(&Tab::Launcher) {
            self.fill_level(list, &self.launcher_rows, searching);
        }
        self.sync_tree();
        if self.nav.tab() == Tab::Launcher && self.ui.root.is_visible() {
            self.sync_focus();
        }
    }

    /// Renders Launcher rows into `list`; `searching` shows each command's
    /// folders as its tooltip.
    fn fill_level(&self, list: &gtk::ListBox, entries: &[Entry], searching: bool) {
        list.remove_all();
        let rows: Vec<Row> = entries.iter().map(|e| e.row(&self.launcher)).collect();
        for (index, (entry, row)) in entries.iter().zip(&rows).enumerate() {
            if !row.selectable {
                list.append(&ui::separator_row());
                continue;
            }
            let (text, tooltip) = self.entry_text(entry, searching);
            let spec = ui::RowSpec {
                index,
                hint: None,
                icon: Some(self.entry_icon(entry)),
                text: &text,
                tooltip: &tooltip,
                branch: row.branch,
                more: false,
            };
            list.append(&ui::build_row(&spec, |_, _| {}));
        }
    }

    /// Icon of a Launcher row: folders and file types by theme name, apps
    /// their own; a command its `icon`, else the icon of the installed app
    /// it starts, else a generic one.
    fn entry_icon(&self, entry: &Entry) -> gio::Icon {
        let themed = |name: &str| gio::ThemedIcon::new(name).upcast::<gio::Icon>();
        match entry {
            Entry::Node(path) => match entry::node_at(&self.launcher, path).map(|n| &n.kind) {
                Some(NodeKind::Folder { .. }) => themed("folder"),
                Some(NodeKind::Command(c)) => {
                    if let Some(icon) = &c.icon {
                        return if icon.starts_with('/') {
                            gio::FileIcon::new(&gio::File::for_path(icon)).upcast()
                        } else {
                            themed(icon)
                        };
                    }
                    match_command(&c.command, &c.args, &self.app_argv)
                        .and_then(|i| self.apps.get(i)?.icon())
                        .unwrap_or_else(|| themed("application-x-executable"))
                },
                _ => themed("application-x-executable"),
            },
            Entry::Recent { file, .. } => {
                let (content_type, _) = gio::content_type_guess(Some(&file.path), None);
                gio::content_type_get_icon(&content_type)
            },
            Entry::App(a) => self
                .apps
                .get(*a)
                .and_then(|app| app.icon())
                .unwrap_or_else(|| themed("application-x-executable")),
        }
    }

    /// Label and tooltip of a Launcher row.
    fn entry_text(&self, entry: &Entry, searching: bool) -> (String, String) {
        match entry {
            Entry::Node(path) => match entry::node_at(&self.launcher, path).map(|n| &n.kind) {
                Some(NodeKind::Folder { name, .. }) => (name.clone(), name.clone()),
                Some(NodeKind::Command(c)) => {
                    let tip = if searching {
                        entry::breadcrumb(&self.launcher, path)
                    } else {
                        c.command.clone()
                    };
                    (c.name.clone(), tip)
                },
                _ => Default::default(),
            },
            Entry::Recent { file, .. } => (file.name.clone(), file.path.clone()),
            Entry::App(a) => self.apps.get(*a).map_or_else(Default::default, |app| {
                let tip = app.description().map(|d| d.to_string()).unwrap_or_default();
                (app.name().to_string(), tip)
            }),
        }
    }
}

/// Sends hover and leave messages for the rows of Launcher level `depth`.
fn track_pointer(list: &gtk::ListBox, depth: usize, tx: &Sender) {
    let motion = gtk::EventControllerMotion::new();
    let last = std::rc::Rc::new(std::cell::Cell::new(-1));
    let (t, l, lb) = (tx.clone(), last.clone(), list.clone());
    let on_move = move |y: f64| {
        let index = lb.row_at_y(y as i32).map_or(-1, |r| r.index());
        if index >= 0 && index != l.get() {
            send(&t, Msg::TreeHover(depth, index as usize));
        }
        l.set(index);
    };
    let enter = on_move.clone();
    motion.connect_enter(move |_, _, y| enter(y));
    motion.connect_motion(move |_, _, y| on_move(y));
    let t = tx.clone();
    motion.connect_leave(move |_| {
        last.set(-1);
        send(&t, Msg::TreeLeave(depth));
    });
    list.add_controller(motion);
}

fn build_ui(app: &adw::Application, tx: &Sender, size: Size) -> Ui {
    let stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .transition_duration(100)
        .build();
    let tab_group = adw::ToggleGroup::builder().homogeneous(true).css_classes(["uti-tabs"]).build();
    let mut lists = HashMap::new();

    // Every tab has the same structure (a list with an empty placeholder), so
    // there is always a focus target.
    for tab in Tab::ALL {
        let listbox = build_list(tab, tx);
        lists.insert(tab, listbox.clone());
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .hexpand(true)
            .child(&listbox)
            .build();
        stack.add_named(&scrolled, Some(ui::page_name(tab)));

        let (title, icon) = ui::tab_label(tab);
        tab_group.add(
            adw::Toggle::builder()
                .name(ui::page_name(tab))
                .icon_name(icon)
                .tooltip(title)
                .build(),
        );
    }
    let t = tx.clone();
    tab_group.connect_active_name_notify(move |g| {
        let name = g.active_name();
        if let Some(tab) = Tab::ALL.into_iter().find(|t| Some(ui::page_name(*t)) == name.as_deref())
        {
            send(&t, Msg::SelectTab(tab));
        }
    });

    let pin = gtk::ToggleButton::builder()
        .icon_name("view-pin-symbolic")
        .tooltip_text("Keep on Top")
        .build();
    let t = tx.clone();
    pin.connect_toggled(move |b| send(&t, Msg::SetPinned(b.is_active())));

    let header = adw::HeaderBar::builder()
        .title_widget(&gtk::Box::new(gtk::Orientation::Horizontal, 0))
        .show_start_title_buttons(false)
        .show_end_title_buttons(false)
        .build();
    header.pack_start(&tab_group);
    header.pack_end(&pin);

    let search = gtk::SearchEntry::builder().hexpand(true).build();
    let t = tx.clone();
    search.connect_search_changed(move |e| send(&t, Msg::SearchChanged(e.text().to_string())));
    // Hidden in minimal until searching; typing anywhere reveals it.
    let search_bar = gtk::SearchBar::builder()
        .child(&search)
        .search_mode_enabled(size.show_search(false, false))
        .build();
    search_bar.connect_entry(&search);
    let t = tx.clone();
    search_bar.connect_search_mode_enabled_notify(move |b| {
        if b.is_search_mode() {
            send(&t, Msg::SearchRevealed);
        }
    });

    let footer = gtk::Label::builder()
        .xalign(0.0)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .css_classes(["dim-label", "caption", "uti-footer"])
        .build();

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.add_top_bar(&search_bar);
    toolbar.set_content(Some(&stack));
    toolbar.add_bottom_bar(&footer);
    // Opaque footer with a separator, so list rows never show through.
    toolbar.set_bottom_bar_style(adw::ToolbarStyle::RaisedBorder);
    toolbar.set_reveal_bottom_bars(size.show_footer());

    let (width, height) = size.window_size();
    let root = adw::ApplicationWindow::builder()
        .application(app)
        .title(MAIN_WINDOW_TITLE)
        .resizable(false)
        .default_width(width)
        .default_height(height)
        // Below AdwWindow's implicit 360x294 minimum, which minimal needs.
        .width_request(200)
        .height_request(200)
        .content(&toolbar)
        .build();
    // add_css_class keeps GTK's own classes (background, csd): replacing them
    // made the window transparent and square.
    root.add_css_class("uti-popup");
    root.add_css_class(size.css_class());
    search_bar.set_key_capture_widget(Some(&root));

    let t = tx.clone();
    root.connect_is_active_notify(move |w| send(&t, Msg::FocusChanged(w.is_active())));
    let t = tx.clone();
    root.connect_close_request(move |_| {
        send(&t, Msg::Hide);
        glib::Propagation::Stop
    });

    // The menu lives in the window's widget tree (parented to the stack), so
    // its focus and keys reach the handlers below; both leave it alone.
    let menu = gtk::PopoverMenu::builder()
        .has_arrow(false)
        .position(gtk::PositionType::Bottom)
        .build();
    menu.set_parent(&stack);
    let t = tx.clone();
    menu.connect_closed(move |_| send(&t, Msg::MenuClosed));
    root.insert_action_group(ui::ROW_ACTIONS, Some(&row_actions(tx)));

    // Keep Nav in step when focus moves by mouse or the Tab key.
    let t = tx.clone();
    let menu_f = menu.clone();
    let (search_f, pin_f, stack_f) = (search.clone(), pin.clone(), stack.clone());
    root.connect_focus_widget_notify(move |w| {
        let Some(widget) = gtk::prelude::GtkWindowExt::focus(w) else {
            return;
        };
        if menu_f.is_visible() {
            return;
        }
        let focus = if widget.is_ancestor(&search_f) || widget == search_f {
            Focus::Search
        } else if widget == pin_f {
            Focus::Pin
        } else if widget.is_ancestor(&stack_f) {
            Focus::List
        } else {
            return;
        };
        send(&t, Msg::FocusMoved(focus));
    });

    // One window-level key handler (capture phase): every navigation key goes
    // to Nav, wherever GTK focus is. Typing in the search entry is left alone.
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let t = tx.clone();
    let (search_k, root_k, menu_k) = (search.clone(), root.clone(), menu.clone());
    keys.connect_key_pressed(move |_, key, _, state| {
        if menu_k.is_visible() {
            return glib::Propagation::Proceed;
        }
        if state.contains(gdk::ModifierType::CONTROL_MASK) && matches!(key, gdk::Key::f) {
            send(&t, Msg::FocusSearch);
            return glib::Propagation::Stop;
        }
        let Some(k) = ui::map_key(key, state) else {
            return glib::Propagation::Proceed;
        };
        let typing = gtk::prelude::GtkWindowExt::focus(&root_k)
            .is_some_and(|w| w.is_ancestor(&search_k) || w == search_k)
            && matches!(
                k,
                Key::Digit(_) | Key::AddSnippet | Key::Up | Key::Delete | Key::Menu | Key::Rename
            );
        if typing {
            return glib::Propagation::Proceed;
        }
        send(&t, Msg::Key(k));
        glib::Propagation::Stop
    });
    root.add_controller(keys);

    Ui {
        root,
        stack,
        tab_group,
        search,
        search_bar,
        toolbar,
        lists,
        pin,
        footer,
        menu,
    }
}

/// A row action name and the message it sends for a visible row index.
type RowActionSpec = (&'static str, fn(usize) -> Msg);

/// Row index actions of the context menu (see [`ui::menu_model`]).
fn row_actions(tx: &Sender) -> gio::SimpleActionGroup {
    let group = gio::SimpleActionGroup::new();
    let indexed: [RowActionSpec; 5] = [
        ("add-snippet", Msg::AddSnippet),
        ("delete", Msg::DeleteRow),
        ("edit", Msg::EditSnippet),
        ("move-up", |i| Msg::MoveSnippet(i, -1)),
        ("move-down", |i| Msg::MoveSnippet(i, 1)),
    ];
    for (name, make) in indexed {
        let action = gio::SimpleAction::new(name, Some(glib::VariantTy::INT32));
        let t = tx.clone();
        action.connect_activate(move |_, v| {
            if let Some(i) = v.and_then(|v| v.get::<i32>()) {
                send(&t, make(i.max(0) as usize));
            }
        });
        group.add_action(&action);
    }
    let action = gio::SimpleAction::new("new", None);
    let t = tx.clone();
    action.connect_activate(move |_, _| send(&t, Msg::NewSnippet));
    group.add_action(&action);
    group
}

fn build_list(tab: Tab, tx: &Sender) -> gtk::ListBox {
    let listbox = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .activate_on_single_click(true)
        .focusable(true)
        .css_classes(["navigation-sidebar"])
        .build();
    listbox.set_placeholder(Some(&ui::empty_page(tab)));
    let t = tx.clone();
    if tab == Tab::Launcher {
        listbox.add_css_class("uti-tree");
        listbox
            .connect_row_activated(move |_, row| send(&t, Msg::TreeClick(0, row.index() as usize)));
        track_pointer(&listbox, 0, tx);
    } else {
        listbox.connect_row_activated(move |_, row| send(&t, Msg::Activate(row.index() as usize)));
    }
    let right_click = gtk::GestureClick::builder().button(gdk::BUTTON_SECONDARY).build();
    let t = tx.clone();
    let lb = listbox.clone();
    right_click.connect_pressed(move |gesture, _, x, y| {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        let index = lb.row_at_y(y as i32).map_or(0, |r| r.index() as usize);
        send(&t, Msg::OpenMenu(index, Some(ui::MenuAnchor { x, y, end: false })));
    });
    listbox.add_controller(right_click);
    listbox
}

/// Starts `command args.. [extra]` without a shell; `false` if it could not
/// start. GSubprocess reaps the child, so no zombies are left behind.
fn spawn(command: &str, args: &[String], extra: Option<&str>) -> bool {
    let mut argv: Vec<&std::ffi::OsStr> = vec![command.as_ref()];
    argv.extend(args.iter().map(std::ffi::OsStr::new));
    argv.extend(extra.map(std::ffi::OsStr::new));
    let launcher = gio::SubprocessLauncher::new(gio::SubprocessFlags::NONE);
    if dev_sandbox() {
        for var in SANDBOX_VARS {
            launcher.unsetenv(var);
        }
    }
    match launcher.spawn(&argv) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Failed to start {command}: {e}");
            false
        },
    }
}

/// Development aid: `UTI_DEBUG_KEYS` logs keys, messages and window commands.
fn debug_keys() -> bool {
    std::env::var_os("UTI_DEBUG_KEYS").is_some()
}

/// XDG variables `app/run-dev.sh` points at its sandbox.
const SANDBOX_VARS: [&str; 2] = ["XDG_CONFIG_HOME", "XDG_DATA_HOME"];

/// Whether the app runs from `app/run-dev.sh`; launched apps then get the
/// session's XDG directories back instead of the sandbox.
fn dev_sandbox() -> bool {
    std::env::var_os("UTI_DEV_SANDBOX").is_some()
}

/// Opens the Updates page of GNOME Software, which installs uti updates from
/// the COPR repository; falls back to the GitHub Releases page without it.
pub fn open_software_updates(parent: &adw::ApplicationWindow) {
    if !spawn("gnome-software", &["--mode=updates".to_string()], None) {
        open_uri(parent, RELEASES_URL);
    }
}

/// Opens `uri` in the default handler.
///
/// The parent is passed only while it is mapped: exporting a hidden surface
/// (xdg-foreign, used for the portal) is a Wayland protocol error that makes
/// Mutter disconnect the app.
fn open_uri(parent: &adw::ApplicationWindow, uri: &'static str) {
    let parent = parent.is_mapped().then_some(parent);
    gtk::UriLauncher::new(uri).launch(parent, None::<&gio::Cancellable>, move |r| {
        if let Err(e) = r {
            eprintln!("Failed to open {uri}: {e}");
        }
    });
}

/// Trims the window height so the list ends on a row boundary instead of
/// showing half of the next row above the footer. Row height depends on the
/// theme and font, so it is measured rather than hard-coded.
fn fit_to_rows(root: &adw::ApplicationWindow, lists: &HashMap<Tab, gtk::ListBox>, tab: Tab) {
    let Some(listbox) = lists.get(&tab) else {
        return;
    };
    let (Some(first), Some(second)) = (listbox.row_at_index(0), listbox.row_at_index(1)) else {
        return; // Need two rows to know the pitch.
    };
    let Some(scrolled) = listbox.ancestor(gtk::ScrolledWindow::static_type()) else {
        return;
    };
    let (Some(a), Some(b)) = (first.compute_bounds(listbox), second.compute_bounds(listbox)) else {
        return;
    };
    let top = f64::from(a.y());
    let pitch = f64::from(b.y() - a.y());
    let gap = pitch - f64::from(a.height());
    let viewport = f64::from(scrolled.height());
    if pitch <= 0.0 || viewport <= 0.0 {
        return;
    }
    // Same padding below the last full row as above the first one.
    let rows = ((viewport - 2.0 * top + gap) / pitch).floor().max(1.0);
    let fitted = 2.0 * top + rows * pitch - gap;
    let excess = (viewport - fitted).round() as i32;
    if excess > 0 {
        root.set_default_size(root.width(), root.height() - excess);
    }
}

/// Development aid: with `UTI_DEBUG_SNAPSHOT=<file.png>`, renders the window
/// and the given popovers (separate surfaces) to one PNG, shortly after.
fn debug_snapshot(root: &adw::ApplicationWindow, popovers: &[gtk::Popover]) {
    let Ok(path) = std::env::var("UTI_DEBUG_SNAPSHOT") else {
        return;
    };
    let root = root.clone();
    let popovers = popovers.to_vec();
    glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
        let snapshot = gtk::Snapshot::new();
        let (x, y) = root.surface_transform();
        let at = |s: &gtk::Snapshot, w: &gtk::Widget, x: f64, y: f64| {
            s.save();
            s.translate(&gtk::graphene::Point::new(x as f32, y as f32));
            gtk::WidgetPaintable::new(Some(w)).snapshot(
                s,
                f64::from(w.width()),
                f64::from(w.height()),
            );
            s.restore();
        };
        at(&snapshot, root.upcast_ref(), x, y);
        for popover in popovers.iter().filter(|p| p.is_visible()) {
            // Popup positions are relative to the parent's surface.
            let (mut px, mut py) = popover.surface_transform();
            let mut native: Option<gtk::Native> = Some(popover.clone().upcast());
            while let Some(n) =
                native.filter(|n| n.upcast_ref::<gtk::Widget>() != root.upcast_ref::<gtk::Widget>())
            {
                if let Some(popup) = n.surface().and_downcast::<gdk::Popup>() {
                    px += f64::from(popup.position_x());
                    py += f64::from(popup.position_y());
                }
                native = n.upcast_ref::<gtk::Widget>().parent().and_then(|w| w.native());
            }
            at(&snapshot, popover.upcast_ref(), px, py);
        }
        if let (Some(node), Some(renderer)) = (snapshot.to_node(), root.renderer()) {
            if let Err(e) = renderer.render_texture(&node, None).save_to_png(&path) {
                eprintln!("Snapshot failed: {e}");
            }
        }
    });
}

/// An action name and the message it sends.
type ActionSpec = (&'static str, fn() -> Msg);

/// Registers the GActions the extension's panel menu invokes via
/// `org.freedesktop.Application.ActivateAction`.
fn register_actions(app: &adw::Application, tx: &Sender) {
    let actions: [ActionSpec; 6] = [
        ("toggle", || Msg::Toggle),
        ("show", || Msg::Show),
        ("hide", || Msg::Hide),
        ("settings", || Msg::OpenSettings(None)),
        ("check-updates", || Msg::CheckUpdates),
        ("quit", || Msg::Quit),
    ];
    if std::env::var_os("UTI_DEBUG_KEYS").is_some() {
        // Development aid: inject navigation keys over D-Bus, e.g.
        // ActivateAction debug-key [<"right">] {}
        let action = gio::SimpleAction::new("debug-key", Some(glib::VariantTy::STRING));
        let t = tx.clone();
        action.connect_activate(move |_, v| {
            let key = match v.and_then(|v| v.get::<String>()).as_deref() {
                Some("up") => Key::Up,
                Some("down") => Key::Down,
                Some("left") => Key::Left,
                Some("right") => Key::Right,
                Some("enter") => Key::Enter,
                Some("escape") => Key::Escape,
                Some("s") => Key::AddSnippet,
                Some("menu") => Key::Menu,
                Some("delete") => Key::Delete,
                Some("f2") => Key::Rename,
                // Anything else: `UTI_DEBUG_SNAPSHOT` with open popovers.
                _ => return send(&t, Msg::DebugSnapshot),
            };
            send(&t, Msg::Key(key));
        });
        app.add_action(&action);
    }
    for (name, make) in actions {
        let action = gio::SimpleAction::new(name, None);
        let t = tx.clone();
        action.connect_activate(move |_, _| send(&t, make()));
        app.add_action(&action);
    }
}
