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
use crate::launcher::get_launcher_config_path;
use crate::snippets::{SnippetItem, SnippetsStore};
use crate::state::settings::{SettingsPage, SettingsTarget, SnippetTarget};

/// `theme.size` values, in the order the settings window lists them.
pub const SIZES: [&str; 3] = ["minimal", "normal", "wide"];
/// Clipboard history limit range and step.
pub const HISTORY_LIMIT: (f64, f64, f64) = (10.0, 500.0, 10.0);

const GITHUB_URL: &str = "https://github.com/noppomario/uti";
/// Tag of the snippet list page in the Snippets page's navigation view.
const SNIPPET_LIST_TAG: &str = "list";

/// A settings change the main window applies.
#[derive(Debug)]
pub enum Event {
    Language(String),
    Size(String),
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
}

impl SettingsUi {
    /// Page shown now.
    pub fn page(&self) -> SettingsPage {
        self.stack
            .visible_child_name()
            .and_then(|n| SettingsPage::from_id(&n))
            .unwrap_or_default()
    }

    /// Shows `target`'s page, and its snippet in the editor.
    pub fn open(&self, target: &SettingsTarget, store: &SnippetsStore) {
        let index = SettingsPage::ALL.iter().position(|p| *p == target.page).unwrap_or(0);
        if let Some(row) = self.sidebar.row_at_index(index as i32) {
            self.sidebar.select_row(Some(&row));
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
    nav.pop_to_tag(SNIPPET_LIST_TAG);
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
    add_page(SettingsPage::General, &[general]);

    // Mouse
    let mouse = adw::PreferencesGroup::new();
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
    add_page(SettingsPage::Mouse, &[mouse]);

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
    add_page(SettingsPage::Appearance, &[appearance]);

    // Clipboard
    let clipboard = adw::PreferencesGroup::new();
    let (min, max, step) = HISTORY_LIMIT;
    let limit = adw::SpinRow::with_range(min, max, step);
    limit.set_title(&t("clipboard.historyLimit.label"));
    limit.set_subtitle(&t("clipboard.historyLimit.description"));
    limit.set_value(config.clipboard_history_limit as f64);
    let e = on_event.clone();
    limit.connect_value_notify(move |row| e(Event::HistoryLimit(row.value() as usize)));
    clipboard.add(&limit);
    add_page(SettingsPage::Clipboard, &[clipboard]);

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
            .tag(SNIPPET_LIST_TAG)
            .child(&page_view(&list_prefs))
            .build(),
    );
    stack.add_named(&snippets_nav, Some(SettingsPage::Snippets.id()));

    let launcher = adw::PreferencesGroup::new();
    launcher.add(&link_row(
        &t("launcher.openConfig.label"),
        Some(&t("launcher.openConfig.description")),
        "document-edit-symbolic",
        open_json(window, get_launcher_config_path(), "{\n  \"commands\": []\n}\n"),
    ));
    add_page(SettingsPage::Launcher, &[launcher]);

    // Advanced
    let advanced = adw::PreferencesGroup::new();
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
    add_page(SettingsPage::Advanced, &[advanced]);

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
    };
    ui.set_snippets(snippets);
    ui.open(
        &SettingsTarget {
            page,
            snippet: None,
        },
        snippets,
    );
    ui
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
            for l in LANGUAGES {
                let key = format!("general.language.options.{l}");
                assert_ne!(tr(lang, &key), key);
            }
        }
    }
}
