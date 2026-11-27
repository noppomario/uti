//! Main popup window controller
//!
//! Widgets send [`Msg`] values over a channel; one local task owns the model
//! and handles them in order, so no `RefCell` is needed and signal handlers
//! never re-enter the model. Keyboard focus is decided by [`Nav`]; after
//! every message the GTK focus is moved to match it.

use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Instant;

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use crate::clipboard::ClipboardStore;
use crate::config::AppConfig;
use crate::dbus;
use crate::launcher::apps::{AppMeta, Category};
use crate::launcher::edit::{self, Place};
use crate::launcher::entry::{self, Entry, Opener, Sources};
use crate::launcher::power::{CallArgs, PowerAction};
use crate::launcher::recent_files;
use crate::launcher::{get_launcher_config_path, LauncherFile, LauncherNode, NodeKind};
use crate::settings;
use crate::snippets::{load_snippets, save_snippets, SnippetsStore};
use crate::state::drop::{self, Hit, Spot};
use crate::state::launcher::{match_command, rank, SearchText};
use crate::state::list::{self, Key, Tab};
use crate::state::menu::{launcher_menu, row_menu, MenuEntry, MenuItem, Sections};
use crate::state::nav::{Effect, Focus, Nav};
use crate::state::scrollbar::Scrollbar;
use crate::state::settings::{LauncherTarget, NewNode, SettingsTarget, SnippetTarget};
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
    /// Context menu of visible row `index` of Launcher level `depth` (0
    /// for the other tabs), at this point in its list (`None`: below the
    /// row). An index past the rows is the empty space below them.
    OpenMenu(usize, usize, Option<ui::MenuAnchor>),
    MenuClosed,
    /// Shortly after the menu closed: hide if focus went to another window.
    MenuSettled,
    /// A context menu entry chosen for row `index` of level `depth` (as in
    /// [`Msg::OpenMenu`]).
    RowAction(MenuItem, usize, usize),
    /// Re-render the Launcher keeping its open levels (recent files
    /// changed).
    LauncherRefresh,
    /// The folder delete confirmation closed; the folder id when confirmed.
    DeleteConfirmed(Option<String>),
    /// A Launcher row at this depth (0: the tab's list) was clicked.
    TreeClick(usize, usize),
    /// The pointer moved onto a Launcher row at this depth.
    TreeHover(usize, usize),
    /// The pointer left the rows of this Launcher depth.
    TreeLeave(usize),
    /// The hover delay of this token passed.
    TreeHoverElapsed(u64),
    /// A row drag started from visible row `index` of a list.
    DragBegin(DropList, usize),
    /// A row drag moved over a list (`None`: below its rows).
    DragMotion(DropList, Option<Hit>),
    DragLeave(DropList),
    /// A row was dropped on a list; applied once the drag has ended.
    Drop(DropList, Option<Hit>),
    /// The row drag ended (dropped or cancelled).
    DragEnd,
    /// New clipboard text (extension signal or read on focus).
    ClipboardText(String),
    /// Installed applications changed.
    AppsChanged,
    Setting(settings::Event),
    /// Development aid: `UTI_DEBUG_SNAPSHOT` after a `debug-key`.
    DebugSnapshot,
    /// Development aid: write the Launcher state and row positions as JSON.
    DebugState(String),
    /// Rebuild the settings window (language changed).
    RepopulateSettings,
    SettingsClosed,
}

/// A row an open Launcher level belongs to, found again after re-rendering.
enum OpenStep {
    Node(String),
    Index(usize),
}

/// A list rows are dragged within.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropList {
    /// Launcher level `depth` (0: the tab's list).
    Launcher(usize),
    Snippets,
}

/// What a row drag carries.
enum Payload {
    Launcher(entry::Drag),
    /// A snippet, by id.
    Snippet(String),
}

/// A row drag in progress.
struct DragState {
    payload: Payload,
    /// List and visible row it started from.
    from: (DropList, usize),
    /// Where it was dropped, applied at the drag's end.
    dropped: Option<DropPlace>,
    /// Last row hovered, for spring-loading: (depth, index, opens).
    hovered: Option<(usize, usize, bool)>,
}

/// Where a drop puts the dragged item.
#[derive(Debug, Clone)]
enum DropPlace {
    Launcher(Place),
    /// Before this snippet (`None`: at the end).
    Snippet(Option<String>),
}

/// What shows the drop position in a list.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Indicator {
    /// The insertion line at this height (list coordinates).
    Line(f64),
    /// The folder row (or empty-level row) dropped into.
    Into(usize),
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
    /// Whether the drop position under the pointer takes the dragged row
    /// (read by drop targets, which must answer at once).
    drop_ok: Rc<Cell<bool>>,
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
    launcher: LauncherFile,
    /// Installed apps shown in launcher search, loaded once and refreshed
    /// by `AppInfoMonitor` (no rescans per keystroke).
    apps: Vec<gio::AppInfo>,
    /// Command line of each app (field codes removed), for command icons.
    app_argv: Vec<Vec<String>>,
    /// What the launcher tree needs of each app in `apps`.
    app_meta: Vec<AppMeta>,
    /// Application names with recent files, read when Launcher rows are
    /// built.
    recent_apps: HashSet<String>,
    /// Rows of the Launcher tab's list.
    launcher_rows: Vec<Entry>,
    /// Open Launcher levels, mirroring `nav.tree()` depths 1..
    cascade: Vec<CascadeLevel>,
    config: AppConfig,
    settings: Option<settings::SettingsUi>,
    drag: Option<DragState>,
    /// Levels closed during a drag, unparented once it ends (the dragged
    /// row may be in one).
    parked: Vec<gtk::Popover>,
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

/// Launcher metadata of each app; `argv` from [`app_argv`].
fn app_meta(apps: &[gio::AppInfo], argv: &[Vec<String>]) -> Vec<AppMeta> {
    apps.iter()
        .zip(argv)
        .map(|(a, argv)| AppMeta {
            id: a.id().map(|i| i.to_string()).unwrap_or_default(),
            name: a.name().to_string(),
            description: a.description().map(|d| d.to_string()).unwrap_or_default(),
            exec: argv.first().cloned(),
            categories: a
                .downcast_ref::<gio_unix::DesktopAppInfo>()
                .and_then(|d| d.categories())
                .map(|c| c.split(';').filter(|c| !c.is_empty()).map(String::from).collect())
                .unwrap_or_default(),
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
    let argv = app_argv(&apps);
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
        launcher: LauncherFile::load(&get_launcher_config_path()),
        app_meta: app_meta(&apps, &argv),
        app_argv: argv,
        apps,
        recent_apps: HashSet::new(),
        launcher_rows: Vec::new(),
        cascade: Vec::new(),
        config,
        settings: None,
        drag: None,
        parked: Vec::new(),
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
                // Pinned: a click on another window closes the open levels
                // (not a menu or dialog opened over them: ADR-036).
                // Not while dragging either (the drag takes focus).
                if self.ui.pin.is_active() && self.nav.focus() != Focus::Menu && self.drag.is_none()
                {
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
            Msg::SetPinned(pinned) => {
                self.sync_more();
                self.window.set_pinned(pinned)
            },
            Msg::LauncherRefresh => {
                self.rerender_launcher(None);
                None
            },
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
                let mut popovers: Vec<gtk::Popover> =
                    self.cascade.iter().map(|l| l.popover.clone()).collect();
                if self.ui.menu.is_visible() {
                    popovers.push(self.ui.menu.clone().upcast());
                }
                debug_snapshot(&self.ui.root, &popovers);
                None
            },
            Msg::DebugState(path) => {
                // Written whole, then renamed: readers poll for the file.
                let tmp = format!("{path}.tmp");
                let written = std::fs::write(&tmp, self.debug_state().to_string())
                    .and_then(|()| std::fs::rename(&tmp, &path));
                if let Err(e) = written {
                    eprintln!("Debug state failed: {e}");
                }
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
            Msg::OpenMenu(depth, i, point) => {
                self.open_menu(depth, i, point);
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
            Msg::RowAction(item, depth, i) => {
                self.row_action(item, depth, i);
                None
            },
            Msg::DeleteConfirmed(id) => {
                if let Some(id) = id {
                    self.edit_launcher(None, |nodes| edit::remove(nodes, &id).is_some());
                }
                self.nav.menu_closed();
                self.sync_focus();
                let t = self.tx.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(150), move || {
                    send(&t, Msg::MenuSettled)
                });
                None
            },
            Msg::ClipboardText(text) => {
                self.add_history(text);
                None
            },
            Msg::AppsChanged => {
                self.apps = load_apps();
                self.app_argv = app_argv(&self.apps);
                self.app_meta = app_meta(&self.apps, &self.app_argv);
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
                let Some(tree) = self.nav.pointer_tree() else {
                    return;
                };
                if let Some(token) = tree.hover(depth, index) {
                    let t = self.tx.clone();
                    glib::timeout_add_local_once(HOVER_DELAY, move || {
                        send(&t, Msg::TreeHoverElapsed(token))
                    });
                }
                self.sync_tree();
                None
            },
            Msg::TreeLeave(depth) => {
                if let Some(tree) = self.nav.pointer_tree() {
                    tree.leave(depth);
                }
                None
            },
            Msg::TreeHoverElapsed(token) => {
                let action = self.nav.pointer_tree().and_then(|t| t.hover_elapsed(token));
                self.tree_action(action)
            },
            Msg::DragBegin(list, index) => {
                self.drag_begin(list, index);
                None
            },
            Msg::DragMotion(list, hit) => {
                self.drag_motion(list, hit);
                None
            },
            Msg::DragLeave(list) => {
                if self.drop_shown().is_some_and(|(l, _)| l == list) {
                    self.show_drop(None);
                }
                if let (DropList::Launcher(d), Some(tree)) = (list, self.nav.pointer_tree()) {
                    tree.leave(d);
                }
                None
            },
            Msg::Drop(list, hit) => {
                let at = self.drop_at(list, hit).map(|(place, _)| place);
                if let Some(drag) = &mut self.drag {
                    drag.dropped = at;
                }
                self.show_drop(None);
                // In case the source row is gone and never reports the end.
                let t = self.tx.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                    send(&t, Msg::DragEnd)
                });
                None
            },
            Msg::DragEnd => {
                self.drag_end();
                None
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
                self.open_menu(0, i, None);
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
            Some(Effect::TreeMenu { depth, index }) => {
                self.open_menu(depth, index, None);
                None
            },
            Some(Effect::TreeEdit { depth, index }) => {
                self.launcher_key(&[MenuItem::Edit], depth, index);
                None
            },
            Some(Effect::TreeDelete { depth, index }) => {
                self.launcher_key(&[MenuItem::Delete, MenuItem::RemoveFromHistory], depth, index);
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
            // Keys only; pointer input does not produce these.
            Some(TreeAction::Menu { .. } | TreeAction::Edit { .. } | TreeAction::Delete { .. }) => {
                None
            },
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
        self.read_recent_apps();
        let entries = self.sources().children(&entry);
        let empty = if self.sources().menu_children(&entry).is_some() {
            "Empty"
        } else {
            "No recent files"
        };
        let (popover, list) = ui::cascade_popover(&parent);
        self.fill_level(&list, depth + 1, &entries, false);
        if entries.is_empty() {
            list.append(&ui::empty_row(empty));
        }
        self.style_scrolled(&list);
        let t = self.tx.clone();
        let d = depth + 1;
        list.connect_row_activated(move |_, r| send(&t, Msg::TreeClick(d, r.index() as usize)));
        track_pointer(&list, d, &self.tx);
        drop_target(&list, DropList::Launcher(d), &self.tx, &self.ui.drop_ok);
        right_click_menu(&list, d, &self.tx);
        if let Some(b) = row.compute_bounds(&parent) {
            popover.set_pointing_to(Some(&gdk::Rectangle::new(
                0,
                b.y() as i32,
                parent.width(),
                b.height() as i32,
            )));
        }
        let rows = entries.iter().map(|e| self.sources().row(e)).collect();
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
                if self.drag.is_some() {
                    self.parked.push(level.popover);
                } else {
                    level.popover.unparent();
                }
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

    /// Menu sections of row `index` of level `depth` of the current tab.
    fn menu_sections(&self, depth: usize, index: usize) -> Sections {
        let tab = self.nav.tab();
        if tab != Tab::Launcher {
            let len = self.tabs.get(&tab).map_or(0, |d| d.visible.len());
            return row_menu(tab, index, len);
        }
        let Some(level) = self.level_entries(depth) else {
            return Vec::new();
        };
        let row = self.sources().menu_row(level, index, depth == 0 && self.searching());
        launcher_menu(row, self.launcher.editable())
    }

    fn searching(&self) -> bool {
        self.tabs.get(&Tab::Launcher).is_some_and(|d| !d.query.trim().is_empty())
    }

    /// Shows the context menu of row `index` of level `depth` of the
    /// current tab, at `anchor` in that level's list.
    fn open_menu(&mut self, depth: usize, index: usize, anchor: Option<ui::MenuAnchor>) {
        let tab = self.nav.tab();
        // Edited by hand (or broken) since: show and offer what is there now.
        if tab == Tab::Launcher && self.launcher.reload_if_changed(&get_launcher_config_path()) {
            self.rerender_launcher(None);
            if let Some(settings) = &self.settings {
                settings.set_launcher(&self.launcher);
            }
        }
        let sections = self.menu_sections(depth, index);
        let list = if tab == Tab::Launcher {
            self.level_list(depth).cloned()
        } else {
            self.ui.lists.get(&tab).cloned()
        };
        let Some(listbox) = list else {
            return;
        };
        if sections.is_empty() {
            return;
        }
        if tab == Tab::Launcher {
            self.nav.tree_mut().select(depth, index);
            self.sync_tree();
        } else {
            self.nav.set_selected(tab, index);
        }
        self.sync_focus();
        let row = listbox.row_at_index(index as i32);
        // The given anchor, or below the start of the row (keyboard).
        let anchor = anchor.unwrap_or_else(|| {
            let (x, y) = row
                .as_ref()
                .and_then(|r| r.compute_bounds(&listbox))
                .map_or((0.0, 0.0), |b| (f64::from(b.x() + 24.0), f64::from(b.y() + b.height())));
            ui::MenuAnchor { x, y, end: false }
        });
        // The menu stays a child of the main window even for a Launcher
        // level: a grabbing popup on a non-grabbing one (the level) is
        // dismissed at once (ADR-038). Only its position is the level's.
        let Some(at) = self.menu_point(depth, &listbox, anchor.x, anchor.y) else {
            return;
        };
        let menu = &self.ui.menu;
        // A popover's halign sets which side of `pointing_to` it starts at.
        menu.set_halign(if anchor.end {
            gtk::Align::End
        } else {
            gtk::Align::Start
        });
        menu.set_menu_model(Some(&ui::menu_model(&sections, depth, index)));
        menu.set_pointing_to(Some(&gdk::Rectangle::new(at.0 as i32, at.1 as i32, 1, 1)));
        self.nav.menu_opened();
        self.window.popover_opened();
        menu.popup();
        // GtkMenuSectionBox shows its separators from an idle callback, after
        // the popup size is fixed; without a second layout the menu scrolls.
        let menu = menu.clone();
        glib::idle_add_local_once(move || {
            menu.queue_resize();
            menu.present();
        });
    }

    /// Point (x, y) of `list` (level `depth`) in the coordinates of the
    /// menu's parent, the main window's stack.
    fn menu_point(&self, depth: usize, list: &gtk::ListBox, x: f64, y: f64) -> Option<(f64, f64)> {
        let point = gtk::graphene::Point::new(x as f32, y as f32);
        let Some(level) = depth.checked_sub(1).and_then(|d| self.cascade.get(d)) else {
            let p = list.compute_point(&self.ui.stack, &point)?;
            return Some((f64::from(p.x()), f64::from(p.y())));
        };
        let in_popover = list.compute_point(&level.popover, &point)?;
        let (ox, oy) = popover_origin(&self.ui.root, &level.popover);
        let window = gtk::graphene::Point::new(
            (f64::from(in_popover.x()) + ox) as f32,
            (f64::from(in_popover.y()) + oy) as f32,
        );
        let p = self.ui.root.compute_point(&self.ui.stack, &window)?;
        // A popover cannot point outside its parent: the menu opens from
        // the window's edge, at the row's height.
        let stack = &self.ui.stack;
        Some((
            f64::from(p.x()).clamp(0.0, f64::from(stack.width() - 1)),
            f64::from(p.y()).clamp(0.0, f64::from(stack.height() - 1)),
        ))
    }

    /// A menu entry chosen for row `index` of level `depth`.
    fn row_action(&mut self, item: MenuItem, depth: usize, index: usize) {
        if self.nav.tab() == Tab::Launcher {
            self.launcher_action(item, depth, index);
            return;
        }
        match item {
            MenuItem::AddToSnippets => self.add_snippet(index),
            MenuItem::Delete | MenuItem::DeleteFromHistory => self.delete_row(index),
            MenuItem::Edit => self.edit_snippet(index),
            MenuItem::AddNew => self.open_settings(SettingsTarget::snippet(SnippetTarget::New)),
            MenuItem::MoveUp => self.move_snippet(index, -1),
            MenuItem::MoveDown => self.move_snippet(index, 1),
            _ => {},
        }
    }

    /// F2 / Delete on a Launcher row: the first of `items` its menu has.
    fn launcher_key(&mut self, items: &[MenuItem], depth: usize, index: usize) {
        let sections = self.menu_sections(depth, index);
        let has = |item: &MenuItem| sections.iter().flatten().any(|e| *e == MenuEntry::Item(*item));
        if let Some(item) = items.iter().find(|i| has(i)) {
            self.launcher_action(*item, depth, index);
        }
    }

    /// The rows the open levels belong to, from the root to level `depth`.
    fn owners(&self, depth: usize) -> Vec<&Entry> {
        let tree = self.nav.tree();
        (1..=depth)
            .filter_map(|d| self.level_entries(d - 1)?.get(tree.owner(d)?))
            .collect()
    }

    /// Runs a Launcher menu entry on row `index` of level `depth`.
    fn launcher_action(&mut self, item: MenuItem, depth: usize, index: usize) {
        let Some(level) = self.level_entries(depth) else {
            return;
        };
        let entry = level.get(index).cloned();
        let node_id = match &entry {
            Some(Entry::Node(path)) => {
                entry::node_at(&self.launcher.nodes, path).map(|n| n.id.clone())
            },
            _ => None,
        };
        let new_place = || self.sources().new_place(&self.owners(depth), entry.as_ref());
        match item {
            MenuItem::Edit => {
                if let Some(id) = node_id {
                    self.open_settings(SettingsTarget::launcher(Some(LauncherTarget::Edit(id))));
                }
            },
            MenuItem::NewFolder | MenuItem::NewCommand => {
                let kind = if item == MenuItem::NewFolder {
                    NewNode::Folder
                } else {
                    NewNode::Command
                };
                if let Some(place) = new_place() {
                    let target = LauncherTarget::New(kind, place);
                    self.open_settings(SettingsTarget::launcher(Some(target)));
                }
            },
            MenuItem::NewSeparator => {
                if let Some(place) = new_place() {
                    let node = LauncherNode::new(NodeKind::Separator);
                    let id = node.id.clone();
                    self.edit_launcher(Some(id), |nodes| edit::insert(nodes, &place, node));
                }
            },
            MenuItem::MoveUp | MenuItem::MoveDown => {
                let Some(id) = node_id else {
                    return;
                };
                let delta = if item == MenuItem::MoveUp { -1 } else { 1 };
                let apps = self.app_meta.clone();
                let shown = move |n: &LauncherNode| entry::node_shown(&apps, n);
                self.edit_launcher(Some(id.clone()), |nodes| {
                    edit::move_by(nodes, &id, delta, shown)
                });
            },
            MenuItem::Delete => {
                let Some(id) = node_id else {
                    return;
                };
                if edit::descendants(&self.launcher.nodes, &id) > 0 {
                    self.confirm_delete(&id, depth, index);
                } else {
                    self.edit_launcher(None, |nodes| edit::remove(nodes, &id).is_some());
                }
            },
            MenuItem::AddToLauncher => {
                let Some(kind) = entry.as_ref().and_then(|e| self.sources().node_for(e)) else {
                    return;
                };
                let place = self.sources().add_place(&self.owners(depth));
                let node = LauncherNode::new(kind);
                self.edit_launcher(None, |nodes| edit::insert(nodes, &place, node));
            },
            MenuItem::OpenFolder | MenuItem::CopyPath | MenuItem::RemoveFromHistory => {
                let Some(Entry::Recent { file, .. }) = entry else {
                    return;
                };
                self.recent_action(item, &file.path);
            },
            MenuItem::FixConfig => self.open_settings(SettingsTarget::launcher(None)),
            _ => {},
        }
    }

    /// Open Folder, Copy Path or Remove from History for a recent file.
    fn recent_action(&mut self, item: MenuItem, path: &str) {
        let file = gio::File::for_path(path);
        match item {
            MenuItem::OpenFolder => {
                gtk::FileLauncher::new(Some(&file)).open_containing_folder(
                    Some(&self.ui.root),
                    None::<&gio::Cancellable>,
                    |r| {
                        if let Err(e) = r {
                            eprintln!("Failed to open the folder: {e}");
                        }
                    },
                );
                if debug_keys() {
                    eprintln!("DEBUG open folder of {path}");
                }
            },
            MenuItem::CopyPath => self.ui.root.clipboard().set_text(path),
            MenuItem::RemoveFromHistory => {
                if let Err(e) = gtk::RecentManager::default().remove_item(&file.uri()) {
                    eprintln!("Failed to remove {path} from recent files: {e}");
                }
                // The manager writes the file from an idle callback.
                let t = self.tx.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(200), move || {
                    send(&t, Msg::LauncherRefresh)
                });
            },
            _ => {},
        }
    }

    /// Asks before deleting folder `id` (row `index` of level `depth`) with
    /// its items.
    fn confirm_delete(&mut self, id: &str, depth: usize, index: usize) {
        let Some(node) = edit::find(&self.launcher.nodes, id) else {
            return;
        };
        let name = match &node.kind {
            NodeKind::Folder { name, .. } => name.clone(),
            _ => return,
        };
        let count = edit::descendants(&self.launcher.nodes, id);
        let dialog = ui::delete_folder_dialog(
            &format!("Delete “{name}”?"),
            &format!("The {count} items in it are deleted too."),
            "Cancel",
            "Delete",
        );
        let (t, id) = (self.tx.clone(), id.to_owned());
        dialog.connect_response(None, move |_, response| {
            let confirmed = (response == ui::DELETE_RESPONSE).then(|| id.clone());
            send(&t, Msg::DeleteConfirmed(confirmed));
        });
        self.nav.tree_mut().select(depth, index);
        self.nav.menu_opened();
        self.window.popover_opened();
        dialog.present(Some(&self.ui.root));
    }

    /// Applies `edit` to the launcher tree and saves it, then re-renders
    /// the Launcher keeping the open levels and selecting node `select`.
    fn edit_launcher(
        &mut self,
        select: Option<String>,
        edit: impl FnOnce(&mut Vec<LauncherNode>) -> bool,
    ) {
        let open = self.open_path();
        let tree = self.nav.tree();
        let at = (tree.active(), tree.selected(tree.active()).unwrap_or(0));
        match self.launcher.edit(&get_launcher_config_path(), edit) {
            Ok(true) => {},
            Ok(false) => return,
            Err(e) => eprintln!("{e}"),
        }
        self.refresh_launcher();
        self.reopen(&open, select.as_deref(), at);
        if let Some(settings) = &self.settings {
            settings.set_launcher(&self.launcher);
        }
    }

    /// Re-renders the Launcher, keeping its open levels and selection (or
    /// selecting node `select`).
    fn rerender_launcher(&mut self, select: Option<&str>) {
        let open = self.open_path();
        let tree = self.nav.tree();
        let at = (tree.active(), tree.selected(tree.active()).unwrap_or(0));
        self.refresh_launcher();
        self.reopen(&open, select, at);
    }

    /// A row drag started: remember what it carries.
    fn drag_begin(&mut self, list: DropList, index: usize) {
        let payload = match list {
            DropList::Launcher(d) => self
                .level_entries(d)
                .and_then(|l| l.get(index))
                .and_then(|e| self.sources().drag(e))
                .map(Payload::Launcher),
            DropList::Snippets => self
                .source(Tab::Snippets, index)
                .and_then(|i| self.snippets.items.get(i))
                .map(|s| Payload::Snippet(s.id.clone())),
        };
        self.drag = payload.map(|payload| DragState {
            payload,
            from: (list, index),
            dropped: None,
            hovered: None,
        });
        self.nav.tree_mut().cancel_pending();
    }

    /// The drag moved over `list`: shows where it would land and
    /// spring-loads folders.
    fn drag_motion(&mut self, list: DropList, hit: Option<Hit>) {
        let at = self.drop_at(list, hit).map(|(_, indicator)| indicator);
        self.ui.drop_ok.set(at.is_some());
        self.show_drop(at.map(|indicator| (list, indicator)));
        let DropList::Launcher(d) = list else {
            return;
        };
        let len = self.level_entries(d).map_or(0, <[Entry]>::len);
        let Some(index) = hit.map(|h| h.index).filter(|&i| i < len) else {
            return;
        };
        let opens = matches!(at, Some(Indicator::Into(i)) if i == index);
        let Some(drag) = &mut self.drag else {
            return;
        };
        if drag.hovered == Some((d, index, opens)) {
            return;
        }
        drag.hovered = Some((d, index, opens));
        let Some(tree) = self.nav.pointer_tree() else {
            return;
        };
        if let Some(token) = tree.drag_hover(d, index, opens) {
            let t = self.tx.clone();
            glib::timeout_add_local_once(HOVER_DELAY, move || {
                send(&t, Msg::TreeHoverElapsed(token))
            });
        }
        self.sync_tree();
    }

    /// Where the dragged row would land over `hit` in `list`, and how that
    /// is shown; `None` where it cannot go (or would not move).
    fn drop_at(&self, list: DropList, hit: Option<Hit>) -> Option<(DropPlace, Indicator)> {
        let drag = self.drag.as_ref()?;
        match (list, &drag.payload) {
            (DropList::Launcher(d), Payload::Launcher(payload)) => {
                if self.searching() {
                    return None;
                }
                let level = self.level_entries(d)?;
                let src = self.sources();
                let spot = drop::spot(level.len(), |i| src.is_folder(&level[i]), hit);
                let place = src.drop_place(&self.owners(d), level, spot)?;
                if let entry::Drag::Move(id) = payload {
                    let nodes = &self.launcher.nodes;
                    if edit::check_move(nodes, id, &place).is_err()
                        || edit::is_noop(nodes, id, &place)
                    {
                        return None;
                    }
                }
                let indicator = indicator(self.level_list(d)?, spot, level.len())?;
                Some((DropPlace::Launcher(place), indicator))
            },
            (DropList::Snippets, Payload::Snippet(_)) => {
                let visible = &self.tabs.get(&Tab::Snippets)?.visible;
                let spot = drop::spot(visible.len(), |_| false, hit);
                let from = drag.from.1;
                let noop = match spot {
                    Spot::Before(v) => v == from || v == from + 1,
                    _ => from + 1 == visible.len(),
                };
                if noop {
                    return None;
                }
                let before = match spot {
                    Spot::Before(v) => Some(self.snippets.items.get(*visible.get(v)?)?.id.clone()),
                    _ => None,
                };
                let indicator = indicator(self.ui.lists.get(&Tab::Snippets)?, spot, visible.len())?;
                Some((DropPlace::Snippet(before), indicator))
            },
            _ => None,
        }
    }

    /// Lists that take row drops: the Launcher levels and Snippets.
    fn drop_lists(&self) -> Vec<(DropList, &gtk::ListBox)> {
        let mut lists: Vec<(DropList, &gtk::ListBox)> = (0..=self.cascade.len())
            .filter_map(|d| Some((DropList::Launcher(d), self.level_list(d)?)))
            .collect();
        lists.extend(self.ui.lists.get(&Tab::Snippets).map(|l| (DropList::Snippets, l)));
        lists
    }

    /// Shows the drop position in one list (none for `None`).
    fn show_drop(&self, at: Option<(DropList, Indicator)>) {
        for (list, widget) in self.drop_lists() {
            let shown = at.filter(|(l, _)| *l == list).map(|(_, i)| i);
            let line = match shown {
                Some(Indicator::Line(y)) => Some(y),
                _ => None,
            };
            let into = match shown {
                Some(Indicator::Into(i)) => Some(i),
                _ => None,
            };
            ui::show_drop_line(widget, line);
            ui::mark_drop_into(widget, into);
        }
    }

    /// The list showing a drop position, with its line height or folder row.
    fn drop_shown(&self) -> Option<(DropList, Indicator)> {
        self.drop_lists().into_iter().find_map(|(list, widget)| {
            ui::drop_line_y(widget)
                .map(Indicator::Line)
                .or_else(|| ui::drop_into(widget).map(Indicator::Into))
                .map(|i| (list, i))
        })
    }

    /// The drag ended: applies its drop, if any.
    fn drag_end(&mut self) {
        self.ui.drop_ok.set(false);
        self.show_drop(None);
        let Some(drag) = self.drag.take() else {
            return;
        };
        self.unpark();
        match (drag.payload, drag.dropped) {
            (Payload::Launcher(entry::Drag::Move(id)), Some(DropPlace::Launcher(place))) => {
                self.edit_launcher(Some(id.clone()), |nodes| {
                    edit::move_to(nodes, &id, &place).is_ok()
                });
            },
            (Payload::Launcher(entry::Drag::Copy(kind)), Some(DropPlace::Launcher(place))) => {
                let node = LauncherNode::new(kind);
                let id = node.id.clone();
                self.edit_launcher(Some(id), |nodes| edit::insert(nodes, &place, node));
            },
            (Payload::Snippet(id), Some(DropPlace::Snippet(before)))
                if self.snippets.move_before(&id, before.as_deref()) =>
            {
                self.snippets_changed();
                self.select_snippet(&id);
            },
            _ => {},
        }
    }

    /// Unparents the levels closed during a drag.
    fn unpark(&mut self) {
        for popover in self.parked.drain(..) {
            popover.unparent();
        }
    }

    /// Shows the rows' "more" buttons while pinned (organize mode).
    fn sync_more(&self) {
        let pinned = self.ui.pin.is_active();
        let lists = self.ui.lists.values().chain(self.cascade.iter().map(|l| &l.list));
        for list in lists {
            ui::show_more_buttons(list, pinned);
        }
    }

    /// The open levels as the rows they belong to: a tree node by id, a
    /// generated row by index.
    fn open_path(&self) -> Vec<OpenStep> {
        let tree = self.nav.tree();
        (1..=tree.depth())
            .filter_map(|d| {
                let index = tree.owner(d)?;
                Some(match self.level_entries(d - 1)?.get(index)? {
                    Entry::Node(path) => {
                        OpenStep::Node(entry::node_at(&self.launcher.nodes, path)?.id.clone())
                    },
                    _ => OpenStep::Index(index),
                })
            })
            .collect()
    }

    /// Row of `level` that `step` names.
    fn find_step(&self, level: &[Entry], step: &OpenStep) -> Option<usize> {
        match step {
            OpenStep::Index(i) => (*i < level.len()).then_some(*i),
            OpenStep::Node(id) => level.iter().position(|e| match e {
                Entry::Node(path) => {
                    entry::node_at(&self.launcher.nodes, path).is_some_and(|n| n.id == *id)
                },
                _ => false,
            }),
        }
    }

    /// Opens the levels of `open` again (after re-rendering) and selects
    /// node `select` where it is shown, else the row at `at` (depth, index).
    fn reopen(&mut self, open: &[OpenStep], select: Option<&str>, at: (usize, usize)) {
        let mut depth = 0;
        for step in open {
            let Some(index) = self.level_entries(depth).and_then(|l| self.find_step(l, step))
            else {
                break;
            };
            self.nav.tree_mut().select(depth, index);
            self.open_level(depth, index, false);
            depth += 1;
        }
        let active = select
            .and_then(|id| {
                (0..=depth).rev().find_map(|d| {
                    let i =
                        self.find_step(self.level_entries(d)?, &OpenStep::Node(id.to_owned()))?;
                    Some((d, i))
                })
            })
            .unwrap_or_else(|| {
                let d = at.0.min(depth);
                let len = self.level_entries(d).map_or(0, <[Entry]>::len);
                (d, at.1.min(len.saturating_sub(1)))
            });
        self.nav.tree_mut().select(active.0, active.1);
        self.sync_tree();
        self.sync_more();
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
        if self.snippets.move_by(&id, delta) {
            self.snippets_changed();
            self.select_snippet(&id);
        }
    }

    /// Selects the visible Snippets row of snippet `id`.
    fn select_snippet(&mut self, id: &str) {
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
                let src = self.sources();
                if let Some(a) = src.app_at(&path) {
                    self.launch_app(a, None);
                } else if let Some(action) = src.power_at(&path) {
                    self.power(action);
                } else {
                    let item = entry::command_at(&self.launcher.nodes, &path)?;
                    spawn(&item.command, &item.args, None);
                }
            },
            Entry::Recent {
                opener: Opener::Command(command),
                file,
            } => {
                let item = entry::command_at(&self.launcher.nodes, &command)?;
                spawn(&item.command, &item.args, Some(&file.path));
            },
            Entry::Recent {
                opener: Opener::App(a),
                file,
            } => self.launch_app(a, Some(&file.path)),
            Entry::App(a) => self.launch_app(a, None),
            Entry::Power(action) => self.power(action),
            Entry::Category(_) => return None,
        }
        self.close_cascade();
        self.window.hide()
    }

    /// Launches installed app `index`, with `file` when given.
    fn launch_app(&self, index: usize, file: Option<&str>) {
        let Some(app) = self.apps.get(index) else {
            return;
        };
        let context = gtk::prelude::WidgetExt::display(&self.ui.root).app_launch_context();
        if dev_sandbox() {
            for var in SANDBOX_VARS {
                context.unsetenv(var);
            }
        }
        let files: Vec<gio::File> = file.into_iter().map(gio::File::for_path).collect();
        if debug_keys() {
            eprintln!("DEBUG launch {:?} {file:?}", app.id());
        }
        if let Err(e) = app.launch(&files, Some(&context)) {
            eprintln!("Failed to launch {}: {e}", app.name());
        }
    }

    /// Calls the D-Bus method behind `action`; only logs it with
    /// `UTI_DRY_POWER` (e2e checks).
    fn power(&self, action: PowerAction) {
        let call = action.call();
        eprintln!("power: {} via {}", call.method, call.dest);
        if std::env::var_os("UTI_DRY_POWER").is_some() {
            return;
        }
        let args = match call.args {
            CallArgs::None => None,
            CallArgs::Bool(b) => Some((b,).to_variant()),
            CallArgs::U32(u) => Some((u,).to_variant()),
        };
        let bus = if call.system {
            gio::BusType::System
        } else {
            gio::BusType::Session
        };
        glib::spawn_future_local(async move {
            let result = match gio::bus_get_future(bus).await {
                Ok(conn) => {
                    conn.call_future(
                        Some(call.dest),
                        call.path,
                        call.interface,
                        call.method,
                        args.as_ref(),
                        None,
                        gio::DBusCallFlags::ALLOW_INTERACTIVE_AUTHORIZATION,
                        -1,
                    )
                    .await
                },
                Err(e) => Err(e),
            };
            if let Err(e) = result {
                eprintln!("power: {} failed: {e}", call.method);
            }
        });
    }

    /// Rows come from the tree, installed apps and their recent files.
    fn sources(&self) -> Sources<'_> {
        Sources {
            nodes: &self.launcher.nodes,
            apps: &self.app_meta,
            recent_apps: &self.recent_apps,
        }
    }

    /// Reads which apps have recent files (once per level built).
    fn read_recent_apps(&mut self) {
        self.recent_apps = recent_files::get_available_apps().into_iter().collect();
    }

    /// Shows the settings window at `target`, creating it on first use.
    fn open_settings(&mut self, target: SettingsTarget) {
        // GNOME does not raise an existing window from the popup (focus
        // stealing prevention), but focuses a new one: open it anew.
        if let Some(ui) = self.settings.take_if(|ui| !ui.window.is_active()) {
            ui.window.destroy();
        }
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
            ui.open(&target, &self.snippets, &self.launcher, &self.app_meta);
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
            &self.launcher,
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
            settings::Event::LauncherSave { target, kind } => {
                match target {
                    LauncherTarget::Edit(id) => {
                        self.edit_launcher(Some(id.clone()), |nodes| {
                            edit::update(nodes, &id, kind)
                        });
                    },
                    LauncherTarget::New(_, place) => {
                        let node = LauncherNode::new(kind);
                        let id = node.id.clone();
                        self.edit_launcher(Some(id), |nodes| edit::insert(nodes, &place, node));
                    },
                }
                return;
            },
            settings::Event::LauncherDelete(id) => {
                self.edit_launcher(None, |nodes| edit::remove(nodes, &id).is_some());
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
                self.launcher = LauncherFile::load(&get_launcher_config_path());
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
                branch: None,
                more: true,
            };
            let tx = self.tx.clone();
            let row = ui::build_row(&spec, move |i, point| send(&tx, Msg::OpenMenu(0, i, point)));
            if tab == Tab::Snippets {
                drag_source(&row, DropList::Snippets, index, &self.tx, &self.ui.pin);
            }
            listbox.append(&row);
        }
        ui::show_more_buttons(listbox, self.ui.pin.is_active());
        if self.nav.tab() == tab && self.ui.root.is_visible() {
            self.sync_focus();
        }
    }

    /// Recomputes the Launcher tab's rows: the tree's root, or while
    /// searching, matching commands of the whole tree and installed apps.
    /// Closes the open levels.
    fn refresh_launcher(&mut self) {
        // Levels are popovers parented to the rows' lists, which
        // `remove_all` would try (and fail) to remove.
        self.truncate_cascade(0);
        self.unpark();
        let query = self.tabs.get(&Tab::Launcher).map(|d| d.query.clone()).unwrap_or_default();
        let searching = !query.trim().is_empty();
        self.read_recent_apps();
        self.launcher_rows = if searching {
            let sources = self.sources();
            let candidates = sources.searchable();
            let texts: Vec<SearchText<'_>> = candidates
                .iter()
                .map(|c| SearchText {
                    name: c.name,
                    description: c.description,
                })
                .collect();
            rank(&texts, &query).into_iter().map(|i| candidates[i].entry.clone()).collect()
        } else {
            self.sources().root_entries()
        };
        if let Some(data) = self.tabs.get_mut(&Tab::Launcher) {
            data.visible = (0..self.launcher_rows.len()).collect();
        }
        let rows = self.launcher_rows.iter().map(|e| self.sources().row(e)).collect();
        self.nav.set_launcher_rows(rows);
        if let Some(list) = self.ui.lists.get(&Tab::Launcher) {
            self.fill_level(list, 0, &self.launcher_rows, searching);
        }
        self.sync_tree();
        if self.nav.tab() == Tab::Launcher && self.ui.root.is_visible() {
            self.sync_focus();
        }
    }

    /// Renders Launcher rows into `list`; `searching` shows each command's
    /// folders as its tooltip.
    fn fill_level(&self, list: &gtk::ListBox, depth: usize, entries: &[Entry], searching: bool) {
        list.remove_all();
        let rows: Vec<Row> = entries.iter().map(|e| self.sources().row(e)).collect();
        for (index, (entry, row)) in entries.iter().zip(&rows).enumerate() {
            if !row.selectable {
                let r = ui::separator_row();
                if !searching && self.sources().drag(entry).is_some() {
                    drag_source(&r, DropList::Launcher(depth), index, &self.tx, &self.ui.pin);
                }
                list.append(&r);
                continue;
            }
            let (text, tooltip) = self.entry_text(entry, searching);
            let spec = ui::RowSpec {
                index,
                hint: None,
                icon: Some(self.entry_icon(entry)),
                text: &text,
                tooltip: &tooltip,
                // Folders open their items; launchable rows their recent files.
                branch: row.branch.then_some(if row.activates {
                    ui::Branch::RecentFiles
                } else {
                    ui::Branch::Folder
                }),
                more: true,
            };
            let tx = self.tx.clone();
            let r = ui::build_row(&spec, move |i, point| send(&tx, Msg::OpenMenu(depth, i, point)));
            if !searching && self.sources().drag(entry).is_some() {
                drag_source(&r, DropList::Launcher(depth), index, &self.tx, &self.ui.pin);
            }
            list.append(&r);
        }
        ui::show_more_buttons(list, self.ui.pin.is_active());
    }

    /// Icon of a Launcher row: folders and file types by theme name, apps
    /// their own, categories and power actions their symbolic icons; a
    /// command its `icon`, else the icon of the installed app it starts,
    /// else a generic one.
    fn entry_icon(&self, entry: &Entry) -> gio::Icon {
        let themed = |names: &[&str]| gio::ThemedIcon::from_names(names).upcast::<gio::Icon>();
        let generic = || themed(&["application-x-executable"]);
        let app_icon =
            |a: usize| self.apps.get(a).and_then(|app| app.icon()).unwrap_or_else(generic);
        match entry {
            Entry::Node(path) => {
                match entry::node_at(&self.launcher.nodes, path).map(|n| &n.kind) {
                    Some(NodeKind::Folder { .. }) => themed(&["folder"]),
                    Some(NodeKind::Apps { category, .. }) => {
                        themed(category.map_or(&["view-app-grid-symbolic"], Category::icons))
                    },
                    Some(NodeKind::Power { action, .. }) => {
                        themed(action.map_or(&["system-shutdown-symbolic"], PowerAction::icons))
                    },
                    Some(NodeKind::App {
                        icon: Some(icon), ..
                    }) => named_icon(icon),
                    Some(NodeKind::App { .. }) => {
                        self.sources().app_at(path).map_or_else(generic, app_icon)
                    },
                    Some(NodeKind::Command(c)) => {
                        if let Some(icon) = &c.icon {
                            return named_icon(icon);
                        }
                        match_command(&c.command, &c.args, &self.app_argv)
                            .and_then(|i| self.apps.get(i)?.icon())
                            .unwrap_or_else(generic)
                    },
                    _ => generic(),
                }
            },
            Entry::Category(c) => themed(c.icons()),
            Entry::Power(p) => themed(p.icons()),
            Entry::Recent { file, .. } => {
                let (content_type, _) = gio::content_type_guess(Some(&file.path), None);
                gio::content_type_get_icon(&content_type)
            },
            Entry::App(a) => app_icon(*a),
        }
    }

    /// Launcher tree state and each level's row rectangles (main window
    /// coordinates; its top-left is at the pointer when shown), for e2e tests.
    /// Label and rectangle (window coordinates) of each shown menu entry.
    fn debug_menu_items(&self) -> Vec<serde_json::Value> {
        fn walk(w: &gtk::Widget, out: &mut Vec<gtk::Widget>) {
            if w.type_().name() == "GtkModelButton" {
                out.push(w.clone());
                return;
            }
            let mut child = w.first_child();
            while let Some(c) = child {
                walk(&c, out);
                child = c.next_sibling();
            }
        }
        let menu = &self.ui.menu;
        if !menu.is_visible() {
            return Vec::new();
        }
        let mut buttons = Vec::new();
        walk(menu.upcast_ref(), &mut buttons);
        let (ox, oy) = popover_origin(&self.ui.root, menu.upcast_ref());
        buttons
            .iter()
            .filter(|b| b.is_mapped())
            .filter_map(|b| {
                let r = b.compute_bounds(menu)?;
                Some(serde_json::json!({
                    "label": b.property::<Option<String>>("text").unwrap_or_default(),
                    "sensitive": b.is_sensitive(),
                    "rect": [f64::from(r.x()) + ox, f64::from(r.y()) + oy, r.width(), r.height()],
                }))
            })
            .collect()
    }

    fn debug_state(&self) -> serde_json::Value {
        let tree = self.nav.tree();
        let root = &self.ui.root;
        let levels: Vec<serde_json::Value> = (0..=tree.depth())
            .map(|d| {
                let (list, origin) = match d {
                    0 => (self.ui.lists.get(&Tab::Launcher), (0.0, 0.0)),
                    d => match self.cascade.get(d - 1) {
                        Some(l) => (Some(&l.list), popover_origin(root, &l.popover)),
                        None => (None, (0.0, 0.0)),
                    },
                };
                let base: &gtk::Widget = match d {
                    0 => root.upcast_ref(),
                    d => self.cascade[d - 1].popover.upcast_ref(),
                };
                let mut rows = Vec::new();
                let mut i = 0;
                while let Some(row) = list.and_then(|l| l.row_at_index(i)) {
                    let b = row.compute_bounds(base).unwrap_or_else(gtk::graphene::Rect::zero);
                    rows.push(serde_json::json!([
                        f64::from(b.x()) + origin.0,
                        f64::from(b.y()) + origin.1,
                        b.width(),
                        b.height()
                    ]));
                    i += 1;
                }
                let texts: Vec<String> = self
                    .level_entries(d)
                    .unwrap_or_default()
                    .iter()
                    .map(|e| self.entry_text(e, false).0)
                    .collect();
                let more = list.map_or(0, ui::visible_more_buttons);
                serde_json::json!({
                    "texts": texts,
                    "more": more,
                    "selected": tree.selected(d),
                    "owner": tree.owner(d),
                    "rows": rows,
                })
            })
            .collect();
        serde_json::json!({
            "visible": root.is_visible(),
            "pinned": self.ui.pin.is_active(),
            "tab": ui::page_name(self.nav.tab()),
            "active": tree.active(),
            "levels": levels,
            "menu": self.ui.menu.is_visible(),
            "menuItems": self.debug_menu_items(),
            "editor": self.settings.as_ref().and_then(|s| s.editor_title()),
            "drag": self.drag.is_some(),
            "drop": self.drop_shown().map(|(list, indicator)| {
                let list = match list {
                    DropList::Launcher(d) => serde_json::json!(d),
                    DropList::Snippets => serde_json::json!("snippets"),
                };
                match indicator {
                    Indicator::Line(y) => serde_json::json!({"list": list, "line": y}),
                    Indicator::Into(i) => serde_json::json!({"list": list, "into": i}),
                }
            }),
            "snippets": self.debug_snippets(),
        })
    }

    /// Snippets tab rows: texts and rectangles (window coordinates).
    fn debug_snippets(&self) -> serde_json::Value {
        let Some(list) = self.ui.lists.get(&Tab::Snippets) else {
            return serde_json::Value::Null;
        };
        let visible = self.tabs.get(&Tab::Snippets).map(|d| d.visible.clone()).unwrap_or_default();
        let texts: Vec<&str> = visible
            .iter()
            .filter_map(|&i| self.snippets.items.get(i))
            .map(|s| s.value.as_str())
            .collect();
        let rows: Vec<serde_json::Value> = (0..)
            .map_while(|i| list.row_at_index(i))
            .map(|row| {
                let b = row.compute_bounds(&self.ui.root).unwrap_or_else(gtk::graphene::Rect::zero);
                serde_json::json!([b.x(), b.y(), b.width(), b.height()])
            })
            .collect();
        serde_json::json!({"texts": texts, "rows": rows, "selected": self.nav.selected()})
    }

    /// Label and tooltip of a Launcher row.
    fn entry_text(&self, entry: &Entry, searching: bool) -> (String, String) {
        let src = self.sources();
        let app_text = |a: usize| {
            self.app_meta
                .get(a)
                .map_or_else(Default::default, |app| (app.name.clone(), app.description.clone()))
        };
        match entry {
            Entry::Node(path) => {
                let name = src.node_name(path).unwrap_or_default().to_string();
                let tip = match entry::node_at(&self.launcher.nodes, path).map(|n| &n.kind) {
                    _ if searching => src.breadcrumb(path),
                    Some(NodeKind::Command(c)) => c.command.clone(),
                    Some(NodeKind::App { .. }) => {
                        src.app_at(path).map(|a| app_text(a).1).unwrap_or_default()
                    },
                    _ => name.clone(),
                };
                (name, tip)
            },
            Entry::Category(c) => (c.label().to_string(), c.label().to_string()),
            Entry::Power(p) => (p.label().to_string(), p.label().to_string()),
            Entry::Recent { file, .. } => (file.name.clone(), file.path.clone()),
            Entry::App(a) => app_text(*a),
        }
    }
}

/// An icon by theme name, or from a file for an absolute path.
fn named_icon(icon: &str) -> gio::Icon {
    if icon.starts_with('/') {
        gio::FileIcon::new(&gio::File::for_path(icon)).upcast()
    } else {
        gio::ThemedIcon::new(icon).upcast()
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
    let drop_ok = Rc::new(Cell::new(false));
    for tab in Tab::ALL {
        let listbox = build_list(tab, tx, &drop_ok);
        lists.insert(tab, listbox.clone());
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .hexpand(true)
            .child(&ui::with_drop_line(&listbox))
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
    // The popup keeps the size of the page it opened with; a larger
    // submenu ("New") would scroll inside it. Resized once the page's slide
    // (and size interpolation) is over.
    menu.connect_notify_local(Some("visible-submenu"), |menu, _| {
        let menu = menu.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
            menu.queue_resize();
            menu.present();
        });
    });
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
        drop_ok,
    }
}

/// Context menu actions (see [`ui::menu_model`]); each takes the row's
/// `(depth, index)`.
fn row_actions(tx: &Sender) -> gio::SimpleActionGroup {
    let group = gio::SimpleActionGroup::new();
    // Both deletes share one action, which runs the tab's delete.
    for item in MenuItem::ALL.into_iter().filter(|i| *i != MenuItem::DeleteFromHistory) {
        let (_, name, _) = ui::menu_entry(item);
        let action = gio::SimpleAction::new(name, Some(&<(i32, i32)>::static_variant_type()));
        // The special item palette comes with P5.
        action.set_enabled(item != MenuItem::NewSpecial);
        let t = tx.clone();
        action.connect_activate(move |_, v| {
            if let Some((depth, index)) = v.and_then(|v| v.get::<(i32, i32)>()) {
                send(&t, Msg::RowAction(item, depth.max(0) as usize, index.max(0) as usize));
            }
        });
        group.add_action(&action);
    }
    group
}

fn build_list(tab: Tab, tx: &Sender, drop_ok: &Rc<Cell<bool>>) -> gtk::ListBox {
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
        drop_target(&listbox, DropList::Launcher(0), tx, drop_ok);
    } else {
        listbox.connect_row_activated(move |_, row| send(&t, Msg::Activate(row.index() as usize)));
        if tab == Tab::Snippets {
            drop_target(&listbox, DropList::Snippets, tx, drop_ok);
        }
    }
    right_click_menu(&listbox, 0, tx);
    listbox
}

/// How `spot` is shown in `list` of `len` rows: a line between rows, or
/// the folder row (the empty-level row for an empty level).
fn indicator(list: &gtk::ListBox, spot: Spot, len: usize) -> Option<Indicator> {
    match (spot, spot.line(len)) {
        (Spot::Into(i), _) => Some(Indicator::Into(i)),
        (_, Some(line)) => ui::row_edge(list, line.row, line.below).map(Indicator::Line),
        (_, None) => Some(Indicator::Into(0)),
    }
}

/// Lets row `index` of `list` be dragged while pinned (organize mode).
fn drag_source(
    row: &gtk::ListBoxRow,
    list: DropList,
    index: usize,
    tx: &Sender,
    pin: &gtk::ToggleButton,
) {
    let (t, e, pin) = (tx.clone(), tx.clone(), pin.clone());
    ui::drag_source(
        row,
        move || pin.is_active(),
        move || send(&t, Msg::DragBegin(list, index)),
        move || send(&e, Msg::DragEnd),
    );
}

/// Takes row drags over `list`.
fn drop_target(list_box: &gtk::ListBox, list: DropList, tx: &Sender, ok: &Rc<Cell<bool>>) {
    let (m, l, d) = (tx.clone(), tx.clone(), tx.clone());
    ui::drop_target(
        list_box,
        ok.clone(),
        move |hit| send(&m, Msg::DragMotion(list, hit)),
        move || send(&l, Msg::DragLeave(list)),
        move |hit| send(&d, Msg::Drop(list, hit)),
    );
}

/// Opens the context menu of the row under a right click on `list` (Launcher
/// level `depth`); below the rows, of the space after them.
fn right_click_menu(list: &gtk::ListBox, depth: usize, tx: &Sender) {
    let right_click = gtk::GestureClick::builder().button(gdk::BUTTON_SECONDARY).build();
    let t = tx.clone();
    let lb = list.clone();
    right_click.connect_pressed(move |gesture, _, x, y| {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        let index = lb.row_at_y(y as i32).map_or_else(
            || (0..).take_while(|&i| lb.row_at_index(i).is_some()).count(),
            |r| r.index() as usize,
        );
        send(&t, Msg::OpenMenu(depth, index, Some(ui::MenuAnchor { x, y, end: false })));
    });
    list.add_controller(right_click);
}

/// Starts `command args.. [extra]` without a shell; `false` if it could not
/// start. GSubprocess reaps the child, so no zombies are left behind.
fn spawn(command: &str, args: &[String], extra: Option<&str>) -> bool {
    let mut argv: Vec<&std::ffi::OsStr> = vec![command.as_ref()];
    argv.extend(args.iter().map(std::ffi::OsStr::new));
    argv.extend(extra.map(std::ffi::OsStr::new));
    if debug_keys() {
        eprintln!("DEBUG spawn {argv:?}");
    }
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

/// Origin of `popover` in `root` widget coordinates (popup positions are
/// relative to the parent's surface).
fn popover_origin(root: &adw::ApplicationWindow, popover: &gtk::Popover) -> (f64, f64) {
    let (mut x, mut y) = popover.surface_transform();
    let mut native: Option<gtk::Native> = Some(popover.clone().upcast());
    while let Some(n) =
        native.filter(|n| n.upcast_ref::<gtk::Widget>() != root.upcast_ref::<gtk::Widget>())
    {
        if let Some(popup) = n.surface().and_downcast::<gdk::Popup>() {
            x += f64::from(popup.position_x());
            y += f64::from(popup.position_y());
        }
        native = n.upcast_ref::<gtk::Widget>().parent().and_then(|w| w.native());
    }
    let (rx, ry) = root.surface_transform();
    (x - rx, y - ry)
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
        let at = |w: &gtk::Widget, (x, y): (f64, f64)| {
            snapshot.save();
            snapshot.translate(&gtk::graphene::Point::new(x as f32, y as f32));
            gtk::WidgetPaintable::new(Some(w)).snapshot(
                &snapshot,
                f64::from(w.width()),
                f64::from(w.height()),
            );
            snapshot.restore();
        };
        at(root.upcast_ref(), (0.0, 0.0));
        for popover in popovers.iter().filter(|p| p.is_visible()) {
            at(popover.upcast_ref(), popover_origin(&root, popover));
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
        let action = gio::SimpleAction::new("debug-state", Some(glib::VariantTy::STRING));
        let t = tx.clone();
        action.connect_activate(move |_, v| {
            if let Some(path) = v.and_then(|v| v.get::<String>()) {
                send(&t, Msg::DebugState(path));
            }
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
