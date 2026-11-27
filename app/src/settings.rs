//! Settings window: a sidebar of pages, applied instantly
//!
//! Changes that affect the main window are reported as [`Event`]s; the rest
//! (autostart, opening files and links) is handled here. Snippets are owned
//! by the main window, which re-renders them with [`SettingsUi::set_snippets`].

use std::path::{Path, PathBuf};
use std::rc::Rc;

use adw::prelude::*;
use gtk::gio;

use crate::autostart;
use crate::config::AppConfig;
use crate::i18n::{tr, LANGUAGES};
use crate::launcher::apps::{AppMeta, Category};
use crate::launcher::form::{self, History};
use crate::launcher::power::PowerAction;
use crate::launcher::{edit, get_launcher_config_path, CommandItem, LauncherFile, NodeKind};
use crate::snippets::{SnippetItem, SnippetsStore};
use crate::state::scrollbar::Scrollbar;
use crate::state::settings::{
    LauncherTarget, NewNode, SettingsPage, SettingsTarget, SnippetTarget,
};

/// `theme.size` values, in the order the settings window lists them.
pub const SIZES: [&str; 3] = ["minimal", "normal", "wide"];
/// Clipboard history limit range and step.
pub const HISTORY_LIMIT: (f64, f64, f64) = (10.0, 500.0, 10.0);

const GITHUB_URL: &str = "https://github.com/noppomario/uti";
/// Tag of the list page in the Snippets and Launcher navigation views.
const LIST_TAG: &str = "list";

/// A settings change the main window applies.
#[derive(Debug)]
pub enum Event {
    Language(String),
    Size(String),
    Scrollbar(String),
    HistoryLimit(usize),
    DesktopDoubleClick(bool),
    LeftRightClick(bool),
    /// Re-read config.json, launcher.json and snippets.json.
    Reload,
    /// Save a snippet; a new one when `id` is `None`.
    SnippetSave {
        id: Option<String>,
        label: Option<String>,
        value: String,
    },
    SnippetDelete(String),
    /// Move a snippet by this many positions.
    SnippetMove(String, isize),
    /// Save a launcher item from the item editor.
    LauncherSave {
        target: LauncherTarget,
        kind: NodeKind,
    },
    LauncherDelete(String),
}

type OnEvent = Rc<dyn Fn(Event)>;

/// Position of `value` in `options`, or 0 when absent.
pub fn index_of(options: &[&str], value: &str) -> u32 {
    options.iter().position(|o| *o == value).unwrap_or(0) as u32
}

/// Creates the settings window (not yet shown).
pub fn new_window(app: &adw::Application) -> adw::ApplicationWindow {
    adw::ApplicationWindow::builder()
        .application(app)
        .default_width(780)
        .default_height(620)
        .build()
}

/// Widgets of a populated settings window.
pub struct SettingsUi {
    pub window: adw::ApplicationWindow,
    lang: String,
    on_event: OnEvent,
    sidebar: gtk::ListBox,
    stack: gtk::Stack,
    snippets_nav: adw::NavigationView,
    snippets_list: gtk::ListBox,
    launcher_nav: adw::NavigationView,
    /// Says why launcher.json cannot be edited.
    launcher_invalid: adw::ActionRow,
}

impl SettingsUi {
    /// Page shown now.
    pub fn page(&self) -> SettingsPage {
        self.stack
            .visible_child_name()
            .and_then(|n| SettingsPage::from_id(&n))
            .unwrap_or_default()
    }

    /// Shows `target`'s page, and its snippet or launcher item in the
    /// editor.
    pub fn open(
        &self,
        target: &SettingsTarget,
        store: &SnippetsStore,
        launcher: &LauncherFile,
        apps: &[AppMeta],
    ) {
        let index = SettingsPage::ALL.iter().position(|p| *p == target.page).unwrap_or(0);
        if let Some(row) = self.sidebar.row_at_index(index as i32) {
            self.sidebar.select_row(Some(&row));
        }
        if let Some(t) = &target.launcher {
            self.open_launcher_editor(t, launcher, apps);
        }
        let item = match &target.snippet {
            None => return,
            Some(SnippetTarget::New) => None,
            Some(SnippetTarget::Edit(id)) => match store.items.iter().find(|i| i.id == *id) {
                Some(item) => Some(item),
                None => return,
            },
        };
        push_editor(&self.snippets_nav, &self.lang, &self.on_event, item);
    }

    /// Title of the open launcher item editor, if the window shows one.
    pub fn editor_title(&self) -> Option<String> {
        let page = self.launcher_nav.visible_page()?;
        (self.window.is_visible()
            && self.page() == SettingsPage::Launcher
            && page.tag().as_deref() != Some(LIST_TAG))
        .then(|| page.title().to_string())
    }

    /// Shows whether launcher.json can be edited.
    pub fn set_launcher(&self, launcher: &LauncherFile) {
        self.launcher_invalid.set_visible(!launcher.editable());
        self.launcher_invalid
            .set_subtitle(launcher.error.as_deref().unwrap_or_default());
    }

    fn open_launcher_editor(
        &self,
        target: &LauncherTarget,
        launcher: &LauncherFile,
        apps: &[AppMeta],
    ) {
        if !launcher.editable() {
            self.launcher_nav.pop_to_tag(LIST_TAG);
            return;
        }
        let (kind, children) = match target {
            LauncherTarget::Edit(id) => match edit::find(&launcher.nodes, id) {
                Some(node) => (node.kind.clone(), edit::descendants(&launcher.nodes, id)),
                None => return,
            },
            LauncherTarget::New(NewNode::Folder, _) => (
                NodeKind::Folder {
                    name: String::new(),
                    children: Vec::new(),
                },
                0,
            ),
            LauncherTarget::New(NewNode::Command, _) => (
                NodeKind::Command(CommandItem {
                    name: String::new(),
                    command: String::new(),
                    args: Vec::new(),
                    history_source: None,
                    icon: None,
                }),
                0,
            ),
        };
        let app = match &kind {
            NodeKind::App { desktop_id, .. } => apps.iter().find(|a| a.id == *desktop_id),
            _ => None,
        };
        let editor = LauncherEditor {
            nav: &self.launcher_nav,
            window: &self.window,
            lang: &self.lang,
            on_event: &self.on_event,
        };
        editor.push(target, kind, children, app);
    }

    /// Re-renders the snippet list.
    pub fn set_snippets(&self, store: &SnippetsStore) {
        let list = &self.snippets_list;
        list.remove_all();
        let t = |key: &str| tr(&self.lang, key);
        let last = store.items.len().saturating_sub(1);
        for (index, item) in store.items.iter().enumerate() {
            let first_line = |s: &str| {
                s.lines().find(|l| !l.trim().is_empty()).unwrap_or_default().trim().to_owned()
            };
            let label = item.label.as_deref().filter(|l| !l.trim().is_empty());
            let row = adw::ActionRow::builder()
                .title(label.map_or_else(|| first_line(&item.value), str::to_owned))
                .subtitle(label.map(|_| first_line(&item.value)).unwrap_or_default())
                .use_markup(false)
                .title_lines(1)
                .subtitle_lines(1)
                .activatable(true)
                .build();
            let buttons: [(&str, String, isize, bool); 2] = [
                ("go-up-symbolic", t("snippets.manager.moveUp"), -1, index > 0),
                ("go-down-symbolic", t("snippets.manager.moveDown"), 1, index < last),
            ];
            for (icon, tooltip, delta, sensitive) in buttons {
                let button = icon_button(icon, &tooltip);
                button.set_sensitive(sensitive);
                let (e, id) = (self.on_event.clone(), item.id.clone());
                button.connect_clicked(move |_| e(Event::SnippetMove(id.clone(), delta)));
                row.add_suffix(&button);
            }
            let delete = icon_button("user-trash-symbolic", &t("snippets.manager.delete"));
            let (e, id) = (self.on_event.clone(), item.id.clone());
            delete.connect_clicked(move |_| e(Event::SnippetDelete(id.clone())));
            row.add_suffix(&delete);
            row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));
            let (nav, lang, e, item) = (
                self.snippets_nav.clone(),
                self.lang.clone(),
                self.on_event.clone(),
                item.clone(),
            );
            row.connect_activated(move |_| push_editor(&nav, &lang, &e, Some(&item)));
            list.append(&row);
        }
    }
}

fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(tooltip)
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build()
}

/// Opens the snippet editor for `item` (a new snippet for `None`) on top of
/// the snippet list.
fn push_editor(
    nav: &adw::NavigationView,
    lang: &str,
    on_event: &OnEvent,
    item: Option<&SnippetItem>,
) {
    let t = |key: &str| tr(lang, key);
    let label = adw::EntryRow::builder()
        .title(t("snippets.manager.label"))
        .text(item.and_then(|i| i.label.as_deref()).unwrap_or_default())
        .build();
    let buffer = gtk::TextBuffer::new(None);
    buffer.set_text(item.map_or("", |i| i.value.as_str()));
    let view = gtk::TextView::builder()
        .buffer(&buffer)
        .wrap_mode(gtk::WrapMode::WordChar)
        .accepts_tab(false)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
        .build();
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_height(240)
        .child(&view)
        .build();
    let frame = gtk::Frame::builder().child(&scrolled).css_classes(["view"]).build();

    let page = adw::PreferencesPage::new();
    let label_group = adw::PreferencesGroup::new();
    label_group.add(&label);
    page.add(&label_group);
    let value_group = group(&t("snippets.manager.value"));
    value_group.add(&frame);
    page.add(&value_group);

    let id = item.map(|i| i.id.clone());
    let save = gtk::Button::builder()
        .label(t("snippets.manager.save"))
        .css_classes(["suggested-action"])
        .sensitive(item.is_some_and(|i| !i.value.trim().is_empty()))
        .build();
    let s = save.clone();
    buffer.connect_changed(move |b| s.set_sensitive(!buffer_text(b).trim().is_empty()));
    let (e, n, b, l, save_id) =
        (on_event.clone(), nav.clone(), buffer.clone(), label.clone(), id.clone());
    save.connect_clicked(move |_| {
        let label = l.text().trim().to_owned();
        e(Event::SnippetSave {
            id: save_id.clone(),
            label: (!label.is_empty()).then_some(label),
            value: buffer_text(&b),
        });
        n.pop();
    });
    if let Some(id) = id {
        let delete = gtk::Button::builder()
            .label(t("snippets.manager.delete"))
            .halign(gtk::Align::Start)
            .css_classes(["destructive-action"])
            .build();
        let (e, n) = (on_event.clone(), nav.clone());
        delete.connect_clicked(move |_| {
            e(Event::SnippetDelete(id.clone()));
            n.pop();
        });
        let delete_group = adw::PreferencesGroup::new();
        delete_group.add(&delete);
        page.add(&delete_group);
    }

    let header = adw::HeaderBar::new();
    header.pack_end(&save);
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&page));
    let title = if item.is_some() {
        t("snippets.manager.editTitle")
    } else {
        t("snippets.manager.newTitle")
    };
    let editor = adw::NavigationPage::builder().title(title).child(&toolbar).build();
    editor.connect_shown(move |_| {
        view.grab_focus();
    });
    nav.pop_to_tag(LIST_TAG);
    nav.push(&editor);
}

fn buffer_text(buffer: &gtk::TextBuffer) -> String {
    buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string()
}

/// A header bar over a preferences page.
fn page_view(page: &adw::PreferencesPage) -> adw::ToolbarView {
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());
    toolbar.set_content(Some(page));
    toolbar
}

/// Fills `window` for `config`, in its language, showing `page`.
pub fn populate(
    window: &adw::ApplicationWindow,
    config: &AppConfig,
    snippets: &SnippetsStore,
    launcher: &LauncherFile,
    page: SettingsPage,
    on_event: OnEvent,
) -> SettingsUi {
    let lang = config.language.as_str();
    let t = |key: &str| tr(lang, key);
    // Differs from the main window's "uti", which the extension looks for.
    window.set_title(Some(&t("windowTitle")));

    let stack = gtk::Stack::new();
    let add_page = |p: SettingsPage, groups: &[adw::PreferencesGroup]| {
        let prefs = adw::PreferencesPage::new();
        for g in groups {
            prefs.add(g);
        }
        stack.add_named(&page_view(&prefs), Some(p.id()));
    };

    // General
    let general = adw::PreferencesGroup::new();
    let autostart_row = adw::SwitchRow::builder()
        .title(t("general.autoStart.label"))
        .subtitle(t("general.autoStart.description"))
        .active(autostart::is_enabled(&autostart::default_dir()))
        .build();
    autostart_row.connect_active_notify(|row| {
        if let Err(e) = autostart::set_enabled(&autostart::default_dir(), row.is_active()) {
            eprintln!("Failed to update autostart: {e}");
        }
    });
    general.add(&autostart_row);
    let language_labels: Vec<String> =
        LANGUAGES.iter().map(|l| t(&format!("general.language.options.{l}"))).collect();
    let language =
        combo(&t("general.language.label"), &language_labels, index_of(&LANGUAGES, lang));
    let e = on_event.clone();
    language.connect_selected_notify(move |row| {
        if let Some(l) = LANGUAGES.get(row.selected() as usize) {
            e(Event::Language((*l).to_string()));
        }
    });
    general.add(&language);

    // Mouse
    let mouse = group(&t("sections.mouse"));
    let double_click = adw::SwitchRow::builder()
        .title(t("mouse.desktopDoubleClick.label"))
        .subtitle(t("mouse.desktopDoubleClick.description"))
        .active(config.mouse.desktop_double_click)
        .build();
    let e = on_event.clone();
    double_click.connect_active_notify(move |row| e(Event::DesktopDoubleClick(row.is_active())));
    mouse.add(&double_click);
    let left_right = adw::SwitchRow::builder()
        .title(t("mouse.leftRightClick.label"))
        .subtitle(t("mouse.leftRightClick.description"))
        .active(config.mouse.left_right_click)
        .build();
    let e = on_event.clone();
    left_right.connect_active_notify(move |row| e(Event::LeftRightClick(row.is_active())));
    mouse.add(&left_right);

    // Appearance
    let appearance = adw::PreferencesGroup::new();
    let size_labels: Vec<String> =
        SIZES.iter().map(|s| t(&format!("appearance.theme.size.options.{s}"))).collect();
    let size = combo(
        &t("appearance.theme.size.label"),
        &size_labels,
        index_of(&SIZES, &config.theme.size),
    );
    let e = on_event.clone();
    size.connect_selected_notify(move |row| {
        if let Some(s) = SIZES.get(row.selected() as usize) {
            e(Event::Size((*s).to_string()));
        }
    });
    appearance.add(&size);
    let scrollbar_labels: Vec<String> = Scrollbar::ALL
        .iter()
        .map(|s| t(&format!("appearance.theme.scrollbar.options.{}", s.id())))
        .collect();
    let current = Scrollbar::parse(&config.theme.scrollbar);
    let scrollbar = combo(
        &t("appearance.theme.scrollbar.label"),
        &scrollbar_labels,
        Scrollbar::ALL.iter().position(|s| *s == current).unwrap_or(0) as u32,
    );
    let e = on_event.clone();
    scrollbar.connect_selected_notify(move |row| {
        if let Some(s) = Scrollbar::ALL.get(row.selected() as usize) {
            e(Event::Scrollbar(s.id().to_string()));
        }
    });
    appearance.add(&scrollbar);
    add_page(SettingsPage::Appearance, &[appearance]);

    // Clipboard
    let clipboard = group(&t("sections.clipboard"));
    let (min, max, step) = HISTORY_LIMIT;
    let limit = adw::SpinRow::with_range(min, max, step);
    limit.set_title(&t("clipboard.historyLimit.label"));
    limit.set_subtitle(&t("clipboard.historyLimit.description"));
    limit.set_value(config.clipboard_history_limit as f64);
    let e = on_event.clone();
    limit.connect_value_notify(move |row| e(Event::HistoryLimit(row.value() as usize)));
    clipboard.add(&limit);

    // Snippets: list -> editor
    let snippets_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    snippets_list.set_placeholder(Some(
        &gtk::Label::builder()
            .label(t("snippets.manager.empty"))
            .margin_top(12)
            .margin_bottom(12)
            .css_classes(["dim-label"])
            .build(),
    ));
    let snippets_nav = adw::NavigationView::new();
    let list_group = adw::PreferencesGroup::new();
    let add = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(t("snippets.manager.add"))
        .css_classes(["flat"])
        .build();
    let (nav, l, e) = (snippets_nav.clone(), lang.to_owned(), on_event.clone());
    add.connect_clicked(move |_| push_editor(&nav, &l, &e, None));
    list_group.set_header_suffix(Some(&add));
    list_group.add(&snippets_list);
    let snippets_advanced = group(&t("sections.advanced"));
    snippets_advanced.add(&link_row(
        &t("snippets.openConfig.label"),
        Some(&t("snippets.openConfig.description")),
        "document-edit-symbolic",
        open_json(window, SnippetsStore::get_storage_path(), "{\n  \"items\": []\n}\n"),
    ));
    let list_prefs = adw::PreferencesPage::new();
    list_prefs.add(&list_group);
    list_prefs.add(&snippets_advanced);
    snippets_nav.add(
        &adw::NavigationPage::builder()
            .title(t("sections.snippets"))
            .tag(LIST_TAG)
            .child(&page_view(&list_prefs))
            .build(),
    );
    stack.add_named(&snippets_nav, Some(SettingsPage::Snippets.id()));

    // Launcher: how to edit -> item editor
    let launcher_group = adw::PreferencesGroup::new();
    let launcher_invalid = adw::ActionRow::builder()
        .title(t("launcher.editor.invalid"))
        .css_classes(["error"])
        .visible(false)
        .build();
    launcher_invalid.add_prefix(&gtk::Image::from_icon_name("dialog-warning-symbolic"));
    launcher_group.add(&launcher_invalid);
    launcher_group.add(
        &adw::ActionRow::builder()
            .title(t("launcher.editor.hint"))
            .css_classes(["dim-label"])
            .build(),
    );
    let launcher_advanced = group(&t("sections.advanced"));
    launcher_advanced.add(&link_row(
        &t("launcher.openConfig.label"),
        Some(&t("launcher.openConfig.description")),
        "document-edit-symbolic",
        open_json(window, get_launcher_config_path(), "{\n  \"items\": []\n}\n"),
    ));
    let launcher_prefs = adw::PreferencesPage::new();
    launcher_prefs.add(&launcher_group);
    launcher_prefs.add(&launcher_advanced);
    let launcher_nav = adw::NavigationView::new();
    launcher_nav.add(
        &adw::NavigationPage::builder()
            .title(t("sections.launcher"))
            .tag(LIST_TAG)
            .child(&page_view(&launcher_prefs))
            .build(),
    );
    stack.add_named(&launcher_nav, Some(SettingsPage::Launcher.id()));

    // Advanced
    let advanced = group(&t("sections.advanced"));
    let w = window.clone();
    advanced.add(&link_row(
        &t("advanced.openConfigFolder"),
        None,
        "folder-open-symbolic",
        move || {
            let dir = AppConfig::get_config_dir();
            if let Err(e) = std::fs::create_dir_all(&dir) {
                eprintln!("Failed to create {}: {e}", dir.display());
            }
            open_path(&w, &dir);
        },
    ));
    let e = on_event.clone();
    advanced.add(&link_row(
        &t("advanced.reloadConfig"),
        None,
        "view-refresh-symbolic",
        move || e(Event::Reload),
    ));
    add_page(SettingsPage::General, &[general, mouse, clipboard, advanced]);

    // About
    let about = adw::PreferencesGroup::new();
    let version = adw::ActionRow::builder().title(t("about.version.label")).build();
    version.add_suffix(
        &gtk::Label::builder()
            .label(env!("CARGO_PKG_VERSION"))
            .css_classes(["dim-label", "numeric"])
            .selectable(true)
            .build(),
    );
    about.add(&version);
    let w = window.clone();
    about.add(&link_row(
        &t("about.checkForUpdates.label"),
        None,
        "adw-external-link-symbolic",
        move || crate::app::open_software_updates(&w),
    ));
    let w = window.clone();
    about.add(&link_row(
        &t("about.openGitHub.label"),
        Some(&t("about.openGitHub.description")),
        "adw-external-link-symbolic",
        move || open_uri(&w, GITHUB_URL),
    ));
    add_page(SettingsPage::About, &[about]);

    // Sidebar
    let sidebar = gtk::ListBox::builder().css_classes(["navigation-sidebar"]).build();
    for p in SettingsPage::ALL {
        sidebar.append(
            &gtk::Label::builder()
                .label(t(&format!("sections.{}", p.id())))
                .xalign(0.0)
                .build(),
        );
    }
    let split = adw::NavigationSplitView::new();
    let content = adw::NavigationPage::builder().child(&stack).build();
    let (st, sp, c, l) = (stack.clone(), split.clone(), content.clone(), lang.to_owned());
    sidebar.connect_row_selected(move |_, row| {
        let Some(p) = row.and_then(|r| SettingsPage::ALL.get(r.index() as usize)) else {
            return;
        };
        st.set_visible_child_name(p.id());
        c.set_title(&tr(&l, &format!("sections.{}", p.id())));
        sp.set_show_content(true);
    });
    let sidebar_view = adw::ToolbarView::new();
    sidebar_view.add_top_bar(&adw::HeaderBar::new());
    sidebar_view.set_content(Some(
        &gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&sidebar)
            .build(),
    ));
    split.set_sidebar(Some(
        &adw::NavigationPage::builder()
            .title(t("windowTitle"))
            .child(&sidebar_view)
            .build(),
    ));
    split.set_content(Some(&content));
    window.set_content(Some(&split));

    let ui = SettingsUi {
        window: window.clone(),
        lang: lang.to_owned(),
        on_event,
        sidebar,
        stack,
        snippets_nav,
        snippets_list,
        launcher_nav,
        launcher_invalid,
    };
    ui.set_snippets(snippets);
    ui.set_launcher(launcher);
    ui.open(
        &SettingsTarget {
            page,
            ..Default::default()
        },
        snippets,
        launcher,
        &[],
    );
    ui
}

/// Builds and pushes the launcher item editor (one page per item kind).
struct LauncherEditor<'a> {
    nav: &'a adw::NavigationView,
    window: &'a adw::ApplicationWindow,
    lang: &'a str,
    on_event: &'a OnEvent,
}

/// Reads the editor's fields into a node kind; `None` while invalid.
type ReadForm = Rc<dyn Fn() -> Option<NodeKind>>;

impl LauncherEditor<'_> {
    fn t(&self, key: &str) -> String {
        tr(self.lang, &format!("launcher.editor.{key}"))
    }

    fn entry(&self, key: &str, text: &str) -> adw::EntryRow {
        adw::EntryRow::builder().title(self.t(key)).text(text).build()
    }

    fn combo(&self, key: &str, labels: &[String], selected: u32) -> adw::ComboRow {
        combo(&self.t(key), labels, selected)
    }

    /// Opens the editor of `kind` for `target`; `children` is the number
    /// of items in a folder, `app` the app of an `app` node.
    fn push(
        &self,
        target: &LauncherTarget,
        kind: NodeKind,
        children: usize,
        app: Option<&AppMeta>,
    ) {
        let group = adw::PreferencesGroup::new();
        let mut entries: Vec<adw::EntryRow> = Vec::new();
        let mut combos: Vec<adw::ComboRow> = Vec::new();
        let (title, read): (&str, ReadForm) = match kind {
            NodeKind::Folder { name, .. } => {
                let name = self.entry("name", &name);
                entries.push(name.clone());
                let read: ReadForm = Rc::new(move || {
                    Some(NodeKind::Folder {
                        name: form::optional(&name.text())?,
                        children: Vec::new(),
                    })
                });
                let title = if matches!(target, LauncherTarget::New(..)) {
                    "newFolder"
                } else {
                    "editFolder"
                };
                (title, read)
            },
            NodeKind::Command(c) => {
                let name = self.entry("name", &c.name);
                let command = self.entry("command", &c.command);
                let args = self.entry("args", &form::format_args(&c.args));
                let icon = self.entry("icon", c.icon.as_deref().unwrap_or_default());
                let (choice, value) = History::of(c.history_source.as_ref());
                let labels: Vec<String> = ["none", "appName", "xbelFile", "vscode"]
                    .iter()
                    .map(|k| self.t(&format!("historyOptions.{k}")))
                    .collect();
                let history = self.combo(
                    "history",
                    &labels,
                    History::ALL.iter().position(|h| *h == choice).unwrap_or(0) as u32,
                );
                let history_value = self.entry("historyValue.appName", &value);
                let value_keys = ["", "appName", "xbelFile", "vscode"];
                let (l, hv) = (self.lang.to_owned(), history_value.clone());
                let show_value = move |selected: u32| {
                    let key = value_keys.get(selected as usize).copied().unwrap_or_default();
                    hv.set_visible(!key.is_empty());
                    if !key.is_empty() {
                        hv.set_title(&tr(&l, &format!("launcher.editor.historyValue.{key}")));
                    }
                };
                show_value(history.selected());
                history.connect_selected_notify(move |row| show_value(row.selected()));
                entries.extend([
                    name.clone(),
                    command.clone(),
                    args.clone(),
                    icon.clone(),
                    history_value.clone(),
                ]);
                combos.push(history.clone());
                let original = c.history_source.clone();
                let read: ReadForm = Rc::new(move || {
                    let choice = *History::ALL.get(history.selected() as usize)?;
                    let text = history_value.text();
                    // Unchanged: keep the source as written (it may set
                    // fields the editor does not show).
                    let history_source =
                        if History::of(original.as_ref()) == (choice, text.to_string()) {
                            original.clone()
                        } else {
                            choice.source(&text).ok()?
                        };
                    Some(NodeKind::Command(CommandItem {
                        name: form::optional(&name.text())?,
                        command: form::optional(&command.text())?,
                        args: form::parse_args(&args.text())?,
                        history_source,
                        icon: form::optional(&icon.text()),
                    }))
                });
                ("editCommand", read)
            },
            NodeKind::App {
                desktop_id,
                name,
                icon,
            } => {
                let app_row = adw::ActionRow::builder()
                    .title(self.t("app"))
                    .subtitle(
                        app.map_or(desktop_id.clone(), |a| format!("{} ({desktop_id})", a.name)),
                    )
                    .build();
                group.add(&app_row);
                group.set_description(Some(&self.t("defaults")));
                let name = self.entry("nameOptional", name.as_deref().unwrap_or_default());
                let icon = self.entry("icon", icon.as_deref().unwrap_or_default());
                entries.extend([name.clone(), icon.clone()]);
                let read: ReadForm = Rc::new(move || {
                    Some(NodeKind::App {
                        desktop_id: desktop_id.clone(),
                        name: form::optional(&name.text()),
                        icon: form::optional(&icon.text()),
                    })
                });
                ("editApp", read)
            },
            NodeKind::Apps { name, category } => {
                let name = self.entry("nameOptional", name.as_deref().unwrap_or_default());
                let labels: Vec<String> = std::iter::once(self.t("allCategories"))
                    .chain(Category::ALL.iter().map(|c| c.label().to_owned()))
                    .collect();
                let selected = category.and_then(|c| Category::ALL.iter().position(|x| *x == c));
                let category =
                    self.combo("category", &labels, selected.map_or(0, |i| i as u32 + 1));
                entries.push(name.clone());
                combos.push(category.clone());
                let read: ReadForm = Rc::new(move || {
                    Some(NodeKind::Apps {
                        name: form::optional(&name.text()),
                        category: (category.selected() as usize)
                            .checked_sub(1)
                            .and_then(|i| Category::ALL.get(i).copied()),
                    })
                });
                ("editApps", read)
            },
            NodeKind::Power { name, action } => {
                let name = self.entry("nameOptional", name.as_deref().unwrap_or_default());
                let labels: Vec<String> = std::iter::once(self.t("allActions"))
                    .chain(PowerAction::ALL.iter().map(|a| a.label().to_owned()))
                    .collect();
                let selected = action.and_then(|a| PowerAction::ALL.iter().position(|x| *x == a));
                let action = self.combo("action", &labels, selected.map_or(0, |i| i as u32 + 1));
                entries.push(name.clone());
                combos.push(action.clone());
                let read: ReadForm = Rc::new(move || {
                    Some(NodeKind::Power {
                        name: form::optional(&name.text()),
                        action: (action.selected() as usize)
                            .checked_sub(1)
                            .and_then(|i| PowerAction::ALL.get(i).copied()),
                    })
                });
                ("editPower", read)
            },
            NodeKind::Separator | NodeKind::Raw(_) => return,
        };
        let title = if matches!(target, LauncherTarget::New(NewNode::Command, _)) {
            "newCommand"
        } else {
            title
        };
        for e in &entries {
            group.add(e);
        }
        for c in &combos {
            group.add(c);
        }
        let page = adw::PreferencesPage::new();
        page.add(&group);

        let save = gtk::Button::builder()
            .label(self.t("save"))
            .css_classes(["suggested-action"])
            .sensitive(read().is_some())
            .build();
        let check = {
            let (save, read) = (save.clone(), read.clone());
            move || save.set_sensitive(read().is_some())
        };
        let do_save = {
            let (e, n, read, target) =
                (self.on_event.clone(), self.nav.clone(), read.clone(), target.clone());
            move || {
                if let Some(kind) = read() {
                    e(Event::LauncherSave {
                        target: target.clone(),
                        kind,
                    });
                    n.pop();
                }
            }
        };
        for entry in &entries {
            let check = check.clone();
            entry.connect_changed(move |_| check());
            let do_save = do_save.clone();
            entry.connect_entry_activated(move |_| do_save());
        }
        for c in &combos {
            let check = check.clone();
            c.connect_selected_notify(move |_| check());
        }
        save.connect_clicked(move |_| do_save());

        if let LauncherTarget::Edit(id) = target {
            let delete = gtk::Button::builder()
                .label(self.t("delete"))
                .halign(gtk::Align::Start)
                .css_classes(["destructive-action"])
                .build();
            let (e, n, w, id) =
                (self.on_event.clone(), self.nav.clone(), self.window.clone(), id.clone());
            let dialog = (children > 0).then(|| {
                let name = entries.first().map(|e| e.text().to_string()).unwrap_or_default();
                crate::ui::delete_folder_dialog(
                    &self.t("deleteFolder").replace("{name}", &name),
                    &self.t("deleteFolderBody").replace("{count}", &children.to_string()),
                    &self.t("cancel"),
                    &self.t("delete"),
                )
            });
            delete.connect_clicked(move |_| {
                let (e, n, id) = (e.clone(), n.clone(), id.clone());
                let run = move || {
                    e(Event::LauncherDelete(id.clone()));
                    n.pop();
                };
                match &dialog {
                    Some(dialog) => {
                        dialog
                            .connect_response(Some(crate::ui::DELETE_RESPONSE), move |_, _| run());
                        dialog.present(Some(&w));
                    },
                    None => run(),
                }
            });
            let delete_group = adw::PreferencesGroup::new();
            delete_group.add(&delete);
            page.add(&delete_group);
        }

        let header = adw::HeaderBar::new();
        header.pack_end(&save);
        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&page));
        let editor = adw::NavigationPage::builder().title(self.t(title)).child(&toolbar).build();
        let first = entries.first().cloned();
        editor.connect_shown(move |_| {
            if let Some(f) = &first {
                f.grab_focus();
            }
        });
        self.nav.pop_to_tag(LIST_TAG);
        self.nav.push(&editor);
    }
}

fn group(title: &str) -> adw::PreferencesGroup {
    adw::PreferencesGroup::builder().title(title).build()
}

fn combo(title: &str, labels: &[String], selected: u32) -> adw::ComboRow {
    let model = gtk::StringList::new(&labels.iter().map(String::as_str).collect::<Vec<_>>());
    adw::ComboRow::builder().title(title).model(&model).selected(selected).build()
}

/// An activatable row with a trailing icon that runs `on_activate`.
fn link_row(
    title: &str,
    subtitle: Option<&str>,
    icon: &str,
    on_activate: impl Fn() + 'static,
) -> adw::ActionRow {
    let row = adw::ActionRow::builder().title(title).activatable(true).build();
    if let Some(subtitle) = subtitle {
        row.set_subtitle(subtitle);
    }
    row.add_suffix(&gtk::Image::from_icon_name(icon));
    row.connect_activated(move |_| on_activate());
    row
}

/// Returns a handler that creates `path` with `default` if missing, then
/// opens it in the default editor.
fn open_json(
    window: &adw::ApplicationWindow,
    path: PathBuf,
    default: &'static str,
) -> impl Fn() + 'static {
    let window = window.clone();
    move || {
        if !path.exists() {
            let created = path
                .parent()
                .map_or(Ok(()), std::fs::create_dir_all)
                .and_then(|()| crate::fs::write_atomic(&path, default.as_bytes()));
            if let Err(e) = created {
                eprintln!("Failed to create {}: {e}", path.display());
                return;
            }
        }
        open_path(&window, &path);
    }
}

fn open_path(window: &adw::ApplicationWindow, path: &Path) {
    let file = gio::File::for_path(path);
    let shown = path.display().to_string();
    gtk::FileLauncher::new(Some(&file)).launch(Some(window), None::<&gio::Cancellable>, move |r| {
        if let Err(e) = r {
            eprintln!("Failed to open {shown}: {e}");
        }
    });
}

fn open_uri(window: &adw::ApplicationWindow, uri: &'static str) {
    gtk::UriLauncher::new(uri).launch(Some(window), None::<&gio::Cancellable>, move |r| {
        if let Err(e) = r {
            eprintln!("Failed to open {uri}: {e}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_of_falls_back_to_first() {
        assert_eq!(index_of(&SIZES, "wide"), 2);
        assert_eq!(index_of(&SIZES, "huge"), 0);
        assert_eq!(index_of(&LANGUAGES, "ja"), 1);
    }

    #[test]
    fn every_option_is_translated() {
        for lang in LANGUAGES {
            for s in SIZES {
                let key = format!("appearance.theme.size.options.{s}");
                assert_ne!(tr(lang, &key), key);
            }
            for s in Scrollbar::ALL {
                let key = format!("appearance.theme.scrollbar.options.{}", s.id());
                assert_ne!(tr(lang, &key), key);
            }
            for l in LANGUAGES {
                let key = format!("general.language.options.{l}");
                assert_ne!(tr(lang, &key), key);
            }
        }
    }
}
