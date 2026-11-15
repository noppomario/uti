//! Settings window: one libadwaita preferences page, applied instantly
//!
//! Changes that affect the main window are reported as [`Event`]s; the rest
//! (autostart, opening files and links) is handled here.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use adw::prelude::*;
use gtk::gio;

use crate::autostart;
use crate::config::AppConfig;
use crate::i18n::{tr, LANGUAGES};
use crate::launcher::get_launcher_config_path;
use crate::snippets::SnippetsStore;

/// `theme.size` values, in the order the settings window lists them.
pub const SIZES: [&str; 3] = ["minimal", "normal", "wide"];
/// Clipboard history limit range and step.
pub const HISTORY_LIMIT: (f64, f64, f64) = (10.0, 500.0, 10.0);

const RELEASES_URL: &str = "https://github.com/noppomario/uti/releases";
const GITHUB_URL: &str = "https://github.com/noppomario/uti";

/// A settings change the main window applies.
#[derive(Debug)]
pub enum Event {
    Language(String),
    Size(String),
    HistoryLimit(usize),
    /// Re-read config.json, launcher.json and snippets.json.
    Reload,
}

/// Position of `value` in `options`, or 0 when absent.
pub fn index_of(options: &[&str], value: &str) -> u32 {
    options.iter().position(|o| *o == value).unwrap_or(0) as u32
}

/// Creates the settings window (not yet shown).
pub fn new_window(app: &adw::Application) -> adw::ApplicationWindow {
    adw::ApplicationWindow::builder()
        .application(app)
        .default_width(560)
        .default_height(680)
        .build()
}

/// Fills `window` with the page for `config`, in its language.
pub fn populate(window: &adw::ApplicationWindow, config: &AppConfig, on_event: Rc<dyn Fn(Event)>) {
    let lang = config.language.as_str();
    let t = |key: &str| tr(lang, key);
    // Differs from the main window's "uti", which the extension looks for.
    window.set_title(Some(&t("windowTitle")));

    let page = adw::PreferencesPage::new();

    // General
    let general = group(&t("sections.general"));
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
    page.add(&general);

    // Appearance
    let appearance = group(&t("sections.appearance"));
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
    page.add(&appearance);

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
    page.add(&clipboard);

    // Snippets and Launcher
    let snippets = group(&t("sections.snippets"));
    snippets.add(&link_row(
        &t("snippets.openConfig.label"),
        Some(&t("snippets.openConfig.description")),
        "document-edit-symbolic",
        open_json(window, SnippetsStore::get_storage_path(), "{\n  \"items\": []\n}\n"),
    ));
    page.add(&snippets);
    let launcher = group(&t("sections.launcher"));
    launcher.add(&link_row(
        &t("launcher.openConfig.label"),
        Some(&t("launcher.openConfig.description")),
        "document-edit-symbolic",
        open_json(window, get_launcher_config_path(), "{\n  \"commands\": []\n}\n"),
    ));
    page.add(&launcher);

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
    page.add(&advanced);

    // About
    let about = group(&t("sections.about"));
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
        move || open_uri(&w, RELEASES_URL),
    ));
    let w = window.clone();
    about.add(&link_row(
        &t("about.openGitHub.label"),
        Some(&t("about.openGitHub.description")),
        "adw-external-link-symbolic",
        move || open_uri(&w, GITHUB_URL),
    ));
    page.add(&about);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());
    toolbar.set_content(Some(&page));
    window.set_content(Some(&toolbar));
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
