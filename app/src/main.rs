//! uti: clipboard history, snippets and launcher popup for GNOME.

mod app;
mod autostart;
mod clipboard;
mod config;
mod dbus;
mod fs;
mod i18n;
mod launcher;
mod paths;
mod settings;
mod snippets;
mod state;
mod ui;

use adw::prelude::*;
use dbus::APP_BUS_NAME;

fn main() -> gtk::glib::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version" || a == "-V" || a == "-v") {
        println!("uti {}", env!("CARGO_PKG_VERSION"));
        return gtk::glib::ExitCode::SUCCESS;
    }
    let minimized = args.iter().any(|a| a == "--minimized");
    // GTK_THEME (set by some parent apps and terminals) replaces libadwaita's
    // stylesheet, which the popup's look depends on. Single-threaded here.
    std::env::remove_var("GTK_THEME");

    // GApplication owns APP_BUS_NAME, which also makes uti single-instance.
    let app = adw::Application::builder().application_id(APP_BUS_NAME).build();
    app.connect_startup(move |app| {
        // GTK's automatic font rendering (no hinting, subpixel positioning)
        // makes digit and capital tops look cut off at scale 1. Use the
        // desktop's own hinting / antialiasing settings instead.
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_font_rendering(gtk::FontRendering::Manual);
        }
        let css = gtk::CssProvider::new();
        css.load_from_string(include_str!("../data/style.css"));
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &css,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        app::start(app, minimized);
    });
    // GApplication emits `activate` on our own start (shown or not, decided
    // in `app::start`) and again when uti is launched a second time.
    let first_activation = std::cell::Cell::new(true);
    app.connect_activate(move |app| {
        if !first_activation.replace(false) {
            app.activate_action("show", None);
        }
    });
    // Arguments are handled above; GTK gets none.
    app.run_with_args::<&str>(&[])
}
