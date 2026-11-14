//! Main popup window component

use std::collections::{HashMap, HashSet};

use relm4::gtk::{self, gdk, gio, glib, prelude::*};
use relm4::prelude::*;
use uti_core::clipboard::ClipboardStore;
use uti_core::config::AppConfig;
use uti_core::snippets::{load_snippets, save_snippets, SnippetItem, SnippetsStore};

use crate::dbus;
use crate::state::list::{self, Action, Key, ListState, Tab, TabTraits};
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
    /// Internal: the extension has the position request; show the window now.
    Present,
    FocusChanged(bool),
    SetPinned(bool),
    OpenSettings,
    CheckUpdates,
    Quit,
    SelectTab(Tab),
    SearchChanged(String),
    SearchKey(Key),
    ListKey(Key),
    FocusSearch,
    /// A row was clicked; visible index in the current tab.
    Activate(usize),
    ToggleStar(usize),
    /// New clipboard text (extension signal or read on focus).
    ClipboardText(String),
}

/// Startup options parsed from the command line.
#[derive(Debug, Default)]
pub struct Init {
    pub minimized: bool,
}

/// Per-tab list state.
#[derive(Default)]
struct TabData {
    list: ListState,
    query: String,
    /// Indices into the backing store for the visible rows.
    visible: Vec<usize>,
}

/// Widgets the model updates directly.
struct Ui {
    stack: gtk::Stack,
    tab_buttons: Vec<(Tab, gtk::ToggleButton)>,
    search: gtk::SearchEntry,
    lists: HashMap<Tab, gtk::ListBox>,
    pin: gtk::ToggleButton,
}

/// Main window model.
pub struct App {
    root: gtk::ApplicationWindow,
    window: WindowState,
    conn: Option<gio::DBusConnection>,
    _subscriptions: Vec<gio::SignalSubscription>,
    tab: Tab,
    tabs: HashMap<Tab, TabData>,
    history: ClipboardStore,
    snippets: SnippetsStore,
    /// Clipboard item ids starred since the window was shown; removed from
    /// history when it hides (they now live in snippets).
    starred: HashSet<String>,
    ui: Ui,
}

const LIST_TABS: [Tab; 3] = [Tab::Clipboard, Tab::Snippets, Tab::Launcher];

fn traits(tab: Tab) -> TabTraits {
    TabTraits {
        can_star: tab == Tab::Clipboard,
        right_opens_jump_list: tab == Tab::Launcher,
    }
}

impl SimpleComponent for App {
    type Init = Init;
    type Input = Msg;
    type Output = ();
    type Root = gtk::ApplicationWindow;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::ApplicationWindow::builder()
            .title(MAIN_WINDOW_TITLE)
            .decorated(false)
            .resizable(false)
            .default_width(500)
            .default_height(700)
            .build()
    }

    fn init(init: Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let ui = build_ui(&root, &sender);

        let s = sender.clone();
        root.connect_is_active_notify(move |w| s.input(Msg::FocusChanged(w.is_active())));
        let s = sender.clone();
        root.connect_close_request(move |_| {
            s.input(Msg::Hide);
            glib::Propagation::Stop
        });
        let window_keys = gtk::EventControllerKey::new();
        let s = sender.clone();
        window_keys.connect_key_pressed(move |_, key, _, state| {
            if state.contains(gdk::ModifierType::CONTROL_MASK) && matches!(key, gdk::Key::f) {
                s.input(Msg::FocusSearch);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        root.add_controller(window_keys);

        let app = relm4::main_application();
        register_actions(&app, &sender);
        let s = sender.clone();
        app.connect_activate(move |_| s.input(Msg::Show));

        let conn = app.dbus_connection();
        let mut subscriptions = Vec::new();
        if let Some(conn) = &conn {
            let s = sender.clone();
            subscriptions.push(dbus::on_triggered(conn, move || s.input(Msg::Toggle)));
            let s = sender.clone();
            subscriptions.push(dbus::on_clipboard_changed(conn, move |text| {
                s.input(Msg::ClipboardText(text))
            }));
        } else {
            eprintln!("No D-Bus connection: double Ctrl and extension features disabled");
        }

        let config = AppConfig::load();
        let mut history = ClipboardStore::load_default();
        history.max_items = config.clipboard_history_limit;

        let mut model = App {
            root: root.clone(),
            window: WindowState::new(),
            conn,
            _subscriptions: subscriptions,
            tab: Tab::Clipboard,
            tabs: LIST_TABS.iter().map(|t| (*t, TabData::default())).collect(),
            history,
            snippets: load_snippets(),
            starred: HashSet::new(),
            ui,
        };
        model.refresh(Tab::Clipboard);
        model.refresh(Tab::Snippets);
        model.select_tab(Tab::Clipboard);

        if !init.minimized {
            sender.input(Msg::Show);
        }
        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Msg, sender: ComponentSender<Self>) {
        let command = match msg {
            Msg::Toggle => self.window.toggle(),
            Msg::Show => self.window.show(),
            Msg::Hide => self.window.hide(),
            Msg::FocusChanged(false) => self.window.focus_lost(),
            Msg::FocusChanged(true) => {
                self.read_clipboard(&sender);
                None
            },
            Msg::SetPinned(pinned) => self.window.set_pinned(pinned),
            Msg::Present => {
                self.root.present();
                self.focus_list();
                None
            },
            Msg::OpenSettings => {
                eprintln!("Settings window is not implemented yet");
                None
            },
            Msg::CheckUpdates => {
                open_uri(&self.root, RELEASES_URL);
                None
            },
            Msg::Quit => {
                relm4::main_application().quit();
                None
            },
            Msg::SelectTab(tab) => {
                self.select_tab(tab);
                None
            },
            Msg::SearchChanged(query) => {
                if let Some(data) = self.tabs.get_mut(&self.tab) {
                    if data.query != query {
                        data.query = query;
                        self.refresh(self.tab);
                    }
                }
                None
            },
            Msg::SearchKey(key) => {
                let action = self.tabs.get_mut(&self.tab).and_then(|d| d.list.on_search_key(key));
                action.and_then(|a| self.perform(a))
            },
            Msg::ListKey(key) => {
                let tab = self.tab;
                let action =
                    self.tabs.get_mut(&tab).and_then(|d| d.list.on_list_key(key, traits(tab)));
                action.and_then(|a| self.perform(a))
            },
            Msg::FocusSearch => {
                self.ui.search.grab_focus();
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
            self.apply(command, &sender);
        }
    }
}

impl App {
    fn apply(&mut self, command: WindowCommand, sender: &ComponentSender<Self>) {
        match command {
            WindowCommand::PositionAndPresent => match self.conn.clone() {
                Some(conn) => {
                    let s = sender.clone();
                    relm4::spawn_local(async move {
                        dbus::position_at_pointer(&conn).await;
                        s.input(Msg::Present);
                    });
                },
                None => {
                    self.root.present();
                    self.focus_list();
                },
            },
            WindowCommand::Hide => {
                self.root.set_visible(false);
                self.commit_starred();
            },
            WindowCommand::SetAbove(above) => {
                if self.ui.pin.is_active() != above {
                    self.ui.pin.set_active(above);
                }
                if let Some(conn) = self.conn.clone() {
                    relm4::spawn_local(async move { dbus::set_above(&conn, above).await });
                }
            },
        }
    }

    fn perform(&mut self, action: Action) -> Option<WindowCommand> {
        match action {
            Action::Select(i) => self.select_row(i),
            Action::Activate(i) => return self.activate(i),
            Action::FocusSearch => {
                self.ui.search.grab_focus();
            },
            Action::FocusList => self.focus_list(),
            Action::ClearQuery => {
                self.ui.search.set_text("");
                self.focus_list();
            },
            Action::PrevTab => {
                if let Some(t) = self.tab.prev() {
                    self.select_tab(t);
                }
            },
            Action::NextTab => {
                if let Some(t) = self.tab.next() {
                    self.select_tab(t);
                }
            },
            Action::ToggleStar(i) => self.toggle_star(i),
            Action::OpenJumpList(_) => {},
        }
        None
    }

    /// Copies the chosen item and hides the window.
    fn activate(&mut self, visible_index: usize) -> Option<WindowCommand> {
        let source = *self.tabs.get(&self.tab)?.visible.get(visible_index)?;
        let text = match self.tab {
            Tab::Clipboard => self.history.items.get(source)?.text.clone(),
            Tab::Snippets => self.snippets.items.get(source)?.value.clone(),
            Tab::Prompt | Tab::Launcher => return None,
        };
        self.root.clipboard().set_text(&text);
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
        if self.starred.remove(&item.id) {
            // Un-starring only clears the mark; the snippet already exists.
        } else {
            self.starred.insert(item.id.clone());
            self.snippets.items.push(SnippetItem::new(item.text.clone(), None));
            if let Err(e) = save_snippets(&self.snippets) {
                eprintln!("Failed to save snippets: {e}");
            }
            self.refresh(Tab::Snippets);
        }
        self.refresh(Tab::Clipboard);
        if let Some(d) = self.tabs.get_mut(&Tab::Clipboard) {
            d.list.set_selected(visible_index);
        }
        if self.tab == Tab::Clipboard {
            self.select_row(visible_index);
        }
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
    fn read_clipboard(&self, sender: &ComponentSender<Self>) {
        let clipboard = self.root.clipboard();
        let s = sender.clone();
        relm4::spawn_local(async move {
            if let Ok(Some(text)) = clipboard.read_text_future().await {
                s.input(Msg::ClipboardText(text.to_string()));
            }
        });
    }

    fn select_tab(&mut self, tab: Tab) {
        self.tab = tab;
        self.ui.stack.set_visible_child_name(ui::page_name(tab));
        for (t, button) in &self.ui.tab_buttons {
            if button.is_active() != (*t == tab) {
                button.set_active(*t == tab);
            }
        }
        let query = self.tabs.get(&tab).map(|d| d.query.clone()).unwrap_or_default();
        self.ui.search.set_visible(tab != Tab::Prompt);
        self.ui.search.set_placeholder_text(Some(ui::search_placeholder(tab)));
        if self.ui.search.text() != query {
            self.ui.search.set_text(&query);
        }
        self.focus_list();
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
            Tab::Prompt | Tab::Launcher => Vec::new(),
        };
        data.list.set_len(rows.len());

        listbox.remove_all();
        let sender_star = self.ui.stack.clone();
        for (index, (text, tooltip, star)) in rows.iter().enumerate() {
            let spec = ui::RowSpec {
                index,
                text,
                tooltip,
                star: *star,
            };
            let stack = sender_star.clone();
            listbox.append(&ui::build_row(&spec, move |i| {
                let _ = stack.activate_action("win.star", Some(&(i as u32).to_variant()));
            }));
        }
        if let Some(row) = listbox.row_at_index(0) {
            listbox.select_row(Some(&row));
        }
    }

    fn select_row(&self, index: usize) {
        if let Some(listbox) = self.ui.lists.get(&self.tab) {
            if let Some(row) = listbox.row_at_index(index as i32) {
                listbox.select_row(Some(&row));
                row.grab_focus();
            }
        }
    }

    fn focus_list(&self) {
        let selected = self.tabs.get(&self.tab).map_or(0, |d| d.list.selected());
        match self.ui.lists.get(&self.tab) {
            Some(listbox) if listbox.row_at_index(0).is_some() => self.select_row(selected),
            Some(listbox) => {
                listbox.grab_focus();
            },
            None => {},
        }
    }
}

fn build_ui(root: &gtk::ApplicationWindow, sender: &ComponentSender<App>) -> Ui {
    let stack = gtk::Stack::builder().vexpand(true).build();
    let mut lists = HashMap::new();
    let mut tab_buttons = Vec::new();
    let tab_bar = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    tab_bar.add_css_class("linked");

    for tab in Tab::ALL {
        let page: gtk::Widget = if LIST_TABS.contains(&tab) && tab != Tab::Launcher {
            let listbox = gtk::ListBox::builder()
                .selection_mode(gtk::SelectionMode::Single)
                .activate_on_single_click(true)
                .build();
            let s = sender.clone();
            listbox
                .connect_row_activated(move |_, row| s.input(Msg::Activate(row.index() as usize)));
            let keys = gtk::EventControllerKey::new();
            keys.set_propagation_phase(gtk::PropagationPhase::Capture);
            let s = sender.clone();
            keys.connect_key_pressed(move |_, key, _, state| match ui::map_key(key, state) {
                Some(k) => {
                    s.input(Msg::ListKey(k));
                    glib::Propagation::Stop
                },
                None => glib::Propagation::Proceed,
            });
            listbox.add_controller(keys);
            lists.insert(tab, listbox.clone());
            gtk::ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .child(&listbox)
                .build()
                .upcast()
        } else {
            // Prompt (PR 4) and Launcher (PR 3) come later.
            gtk::Label::new(Some(ui::tab_label(tab).1)).upcast()
        };
        stack.add_named(&page, Some(ui::page_name(tab)));

        let (icon, label) = ui::tab_label(tab);
        let button = gtk::ToggleButton::builder()
            .icon_name(icon)
            .tooltip_text(label)
            .focusable(false)
            .build();
        let s = sender.clone();
        button.connect_clicked(move |_| s.input(Msg::SelectTab(tab)));
        tab_bar.append(&button);
        tab_buttons.push((tab, button));
    }

    let pin = gtk::ToggleButton::builder()
        .icon_name("view-pin-symbolic")
        .tooltip_text("Pin window")
        .focusable(false)
        .build();
    let s = sender.clone();
    pin.connect_toggled(move |b| s.input(Msg::SetPinned(b.is_active())));

    let search = gtk::SearchEntry::builder().hexpand(true).build();
    let s = sender.clone();
    search.connect_search_changed(move |e| s.input(Msg::SearchChanged(e.text().to_string())));
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let s = sender.clone();
    keys.connect_key_pressed(move |_, key, _, state| match ui::map_key(key, state) {
        Some(k @ (Key::Down | Key::Enter | Key::Escape | Key::Left | Key::Right)) => {
            s.input(Msg::SearchKey(k));
            glib::Propagation::Stop
        },
        _ => glib::Propagation::Proceed,
    });
    search.add_controller(keys);

    // Star buttons inside rows reach the model through a window action.
    let star = gio::SimpleAction::new("star", Some(glib::VariantTy::UINT32));
    let s = sender.clone();
    star.connect_activate(move |_, v| {
        if let Some(i) = v.and_then(|v| v.get::<u32>()) {
            s.input(Msg::ToggleStar(i as usize));
        }
    });
    root.add_action(&star);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    header.append(&tab_bar);
    header.append(&search);
    header.append(&pin);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.append(&header);
    content.append(&stack);
    root.set_child(Some(&content));

    Ui {
        stack,
        tab_buttons,
        search,
        lists,
        pin,
    }
}

/// Opens `uri` in the default handler.
///
/// The parent is passed only while it is mapped: exporting a hidden surface
/// (xdg-foreign, used for the portal) is a Wayland protocol error that makes
/// Mutter disconnect the app.
fn open_uri(parent: &gtk::ApplicationWindow, uri: &'static str) {
    let parent = parent.is_mapped().then_some(parent);
    gtk::UriLauncher::new(uri).launch(parent, None::<&gio::Cancellable>, move |r| {
        if let Err(e) = r {
            eprintln!("Failed to open {uri}: {e}");
        }
    });
}

/// An action name and the message it sends.
type ActionSpec = (&'static str, fn() -> Msg);

/// Registers the GActions the extension's panel menu invokes via
/// `org.freedesktop.Application.ActivateAction`.
fn register_actions(app: &impl IsA<gio::ActionMap>, sender: &ComponentSender<App>) {
    let actions: [ActionSpec; 6] = [
        ("toggle", || Msg::Toggle),
        ("show", || Msg::Show),
        ("hide", || Msg::Hide),
        ("settings", || Msg::OpenSettings),
        ("check-updates", || Msg::CheckUpdates),
        ("quit", || Msg::Quit),
    ];
    for (name, make) in actions {
        let action = gio::SimpleAction::new(name, None);
        let s = sender.clone();
        action.connect_activate(move |_, _| s.input(make()));
        app.add_action(&action);
    }
}
