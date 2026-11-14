//! Main popup window controller
//!
//! Widgets send [`Msg`] values over a channel; one local task owns the model
//! and handles them in order, so no `RefCell` is needed and signal handlers
//! never re-enter the model. Keyboard focus is decided by [`Nav`]; after
//! every message the GTK focus is moved to match it.

use std::collections::{HashMap, HashSet};

use adw::prelude::*;
use gtk::{gdk, gio, glib};
use uti_core::clipboard::ClipboardStore;
use uti_core::config::AppConfig;
use uti_core::snippets::{load_snippets, save_snippets, SnippetItem, SnippetsStore};

use crate::dbus;
use crate::state::list::{self, Key, Tab};
use crate::state::nav::{Effect, Focus, Nav};
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
    SetPinned(bool),
    OpenSettings,
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
    /// A row was clicked; visible index in the current tab.
    Activate(usize),
    ToggleStar(usize),
    /// New clipboard text (extension signal or read on focus).
    ClipboardText(String),
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
    lists: HashMap<Tab, gtk::ListBox>,
    pin: gtk::ToggleButton,
    footer: gtk::Label,
}

/// Main window model.
struct App {
    tx: Sender,
    ui: Ui,
    window: WindowState,
    nav: Nav,
    conn: Option<gio::DBusConnection>,
    _subscriptions: Vec<gio::SignalSubscription>,
    tabs: HashMap<Tab, TabData>,
    history: ClipboardStore,
    snippets: SnippetsStore,
    /// Clipboard item ids starred since the window was shown; removed from
    /// history when it hides (they now live in snippets).
    starred: HashSet<String>,
    /// Window height already trimmed to whole rows.
    rows_fitted: bool,
}

/// Window size and CSS class for `theme.size`.
fn density(size: &str) -> (i32, i32, &'static str) {
    match size {
        "minimal" => (320, 600, "density-minimal"),
        "wide" => (750, 800, "density-wide"),
        _ => (500, 700, "density-normal"),
    }
}

/// Builds the main window and starts the message loop.
pub fn start(app: &adw::Application, minimized: bool) {
    let (tx, rx) = async_channel::unbounded();
    let config = AppConfig::load();
    let ui = build_ui(app, &tx, &config.theme.size);
    register_actions(app, &tx);

    let conn = app.dbus_connection();
    let mut subscriptions = Vec::new();
    if let Some(conn) = &conn {
        let t = tx.clone();
        subscriptions.push(dbus::on_triggered(conn, move || send(&t, Msg::Toggle)));
        let t = tx.clone();
        subscriptions
            .push(dbus::on_clipboard_changed(conn, move |text| send(&t, Msg::ClipboardText(text))));
    } else {
        eprintln!("No D-Bus connection: double Ctrl and extension features disabled");
    }

    let mut history = ClipboardStore::load_default();
    history.max_items = config.clipboard_history_limit;

    let mut model = App {
        tx: tx.clone(),
        ui,
        window: WindowState::new(),
        nav: Nav::new(),
        conn,
        _subscriptions: subscriptions,
        tabs: Tab::ALL.iter().map(|t| (*t, TabData::default())).collect(),
        history,
        snippets: load_snippets(),
        starred: HashSet::new(),
        rows_fitted: false,
    };
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
        let command = match msg {
            Msg::Toggle => self.window.toggle(),
            Msg::Show => self.window.show(),
            Msg::Hide => self.window.hide(),
            Msg::FocusChanged(false) => self.window.focus_lost(),
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
                debug_snapshot(&self.ui.root);
                None
            },
            Msg::OpenSettings => {
                eprintln!("Settings window is not implemented yet");
                None
            },
            Msg::CheckUpdates => {
                open_uri(&self.ui.root, RELEASES_URL);
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
                None
            },
            Msg::Key(key) => {
                let effect = self.nav.on_key(key);
                let command = self.navigate(effect);
                if std::env::var_os("UTI_DEBUG_KEYS").is_some() {
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
                None
            },
            Msg::FocusSearch => {
                self.nav.focus_search();
                self.sync_focus();
                None
            },
            Msg::Activate(i) => self.activate(i),
            Msg::ToggleStar(i) => {
                self.toggle_star(i);
                None
            },
            Msg::ClipboardText(text) => {
                self.add_history(text);
                None
            },
        };
        if let Some(command) = command {
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
            Some(Effect::ToggleStar(i)) => {
                self.toggle_star(i);
                None
            },
            Some(Effect::ClearQuery) => {
                self.ui.search.set_text("");
                None
            },
            Some(Effect::TogglePin) => self.window.set_pinned(!self.ui.pin.is_active()),
            // Selection is applied by sync_focus; jump lists come with PR 3.
            Some(Effect::SelectRow(_) | Effect::OpenJumpList(_)) | None => None,
        };
        self.sync_focus();
        command
    }

    /// Puts GTK focus where [`Nav`] says it is.
    fn sync_focus(&self) {
        // Focus is moved by our own key handling, and GTK only draws focus
        // rings after its own keyboard navigation; keep them on (the popup
        // is keyboard-driven).
        self.ui.root.set_focus_visible(true);
        match self.nav.focus() {
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

    fn apply(&mut self, command: WindowCommand) {
        match command {
            WindowCommand::PositionAndPresent => match self.conn.clone() {
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
            },
            WindowCommand::Hide => {
                self.ui.root.set_visible(false);
                self.commit_starred();
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
            Tab::Prompt | Tab::Launcher => return None,
        };
        self.ui.root.clipboard().set_text(&text);
        self.window.hide()
    }

    fn toggle_star(&mut self, visible_index: usize) {
        let Some(&source) =
            self.tabs.get(&Tab::Clipboard).and_then(|d| d.visible.get(visible_index))
        else {
            return;
        };
        let Some(item) = self.history.items.get(source) else {
            return;
        };
        // Un-starring only clears the mark; the snippet already exists.
        if !self.starred.remove(&item.id) {
            self.starred.insert(item.id.clone());
            self.snippets.items.push(SnippetItem::new(item.text.clone(), None));
            if let Err(e) = save_snippets(&self.snippets) {
                eprintln!("Failed to save snippets: {e}");
            }
            self.refresh(Tab::Snippets);
        }
        self.refresh(Tab::Clipboard);
        self.nav.set_selected(Tab::Clipboard, visible_index);
        self.sync_focus();
    }

    /// Removes items starred into snippets from history (by id).
    fn commit_starred(&mut self) {
        if self.starred.is_empty() {
            return;
        }
        for id in self.starred.drain() {
            self.history.remove_by_id(&id);
        }
        self.save_history();
        self.refresh(Tab::Clipboard);
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
        let Some(data) = self.tabs.get_mut(&tab) else {
            return;
        };
        let Some(listbox) = self.ui.lists.get(&tab) else {
            return;
        };
        let rows: Vec<(String, String, Option<bool>)> = match tab {
            Tab::Clipboard => {
                let texts = self.history.items.iter().map(|i| i.text.as_str());
                data.visible = list::filter(texts, &data.query);
                data.visible
                    .iter()
                    .map(|&i| {
                        let item = &self.history.items[i];
                        let starred = self.starred.contains(&item.id);
                        (item.text.clone(), item.text.clone(), Some(starred))
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
                        (label.to_owned(), s.value.clone(), None)
                    })
                    .collect()
            },
            // Launcher items come with PR 3.
            Tab::Prompt | Tab::Launcher => {
                data.visible.clear();
                Vec::new()
            },
        };
        self.nav.set_len(tab, rows.len());

        listbox.remove_all();
        for (index, (text, tooltip, star)) in rows.iter().enumerate() {
            let spec = ui::RowSpec {
                index,
                text,
                tooltip,
                star: *star,
            };
            let tx = self.tx.clone();
            listbox.append(&ui::build_row(&spec, move |i| send(&tx, Msg::ToggleStar(i))));
        }
        if self.nav.tab() == tab && self.ui.root.is_visible() {
            self.sync_focus();
        }
    }
}

fn build_ui(app: &adw::Application, tx: &Sender, size: &str) -> Ui {
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
        let page = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .child(&listbox)
            .build();
        stack.add_named(&page, Some(ui::page_name(tab)));

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
    let search_bar = gtk::Box::builder()
        .margin_start(6)
        .margin_end(6)
        .margin_top(6)
        .margin_bottom(6)
        .build();
    search_bar.append(&search);

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

    let (width, height, density_class) = density(size);
    let root = adw::ApplicationWindow::builder()
        .application(app)
        .title(MAIN_WINDOW_TITLE)
        .resizable(false)
        .default_width(width)
        .default_height(height)
        .content(&toolbar)
        .build();
    // add_css_class keeps GTK's own classes (background, csd): replacing them
    // made the window transparent and square.
    root.add_css_class("uti-popup");
    root.add_css_class(density_class);

    let t = tx.clone();
    root.connect_is_active_notify(move |w| send(&t, Msg::FocusChanged(w.is_active())));
    let t = tx.clone();
    root.connect_close_request(move |_| {
        send(&t, Msg::Hide);
        glib::Propagation::Stop
    });

    // Keep Nav in step when focus moves by mouse or the Tab key.
    let t = tx.clone();
    let (search_f, pin_f, stack_f) = (search.clone(), pin.clone(), stack.clone());
    root.connect_focus_widget_notify(move |w| {
        let Some(widget) = gtk::prelude::GtkWindowExt::focus(w) else {
            return;
        };
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
    let (search_k, root_k) = (search.clone(), root.clone());
    keys.connect_key_pressed(move |_, key, _, state| {
        if state.contains(gdk::ModifierType::CONTROL_MASK) && matches!(key, gdk::Key::f) {
            send(&t, Msg::FocusSearch);
            return glib::Propagation::Stop;
        }
        let Some(k) = ui::map_key(key, state) else {
            return glib::Propagation::Proceed;
        };
        let typing = gtk::prelude::GtkWindowExt::focus(&root_k)
            .is_some_and(|w| w.is_ancestor(&search_k) || w == search_k)
            && matches!(k, Key::Digit(_) | Key::Star | Key::Up);
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
        lists,
        pin,
        footer,
    }
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
    listbox.connect_row_activated(move |_, row| send(&t, Msg::Activate(row.index() as usize)));
    listbox
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
/// to a PNG shortly after it is shown.
fn debug_snapshot(root: &adw::ApplicationWindow) {
    let Ok(path) = std::env::var("UTI_DEBUG_SNAPSHOT") else {
        return;
    };
    let root = root.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
        let (w, h) = (f64::from(root.width()), f64::from(root.height()));
        let snapshot = gtk::Snapshot::new();
        gtk::WidgetPaintable::new(Some(&root)).snapshot(&snapshot, w, h);
        if let (Some(node), Some(renderer)) =
            (snapshot.to_node(), root.native().and_then(|n| n.renderer()))
        {
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
        ("settings", || Msg::OpenSettings),
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
                _ => return,
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
