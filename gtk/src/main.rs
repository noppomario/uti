//! uti: clipboard history, snippets and launcher popup for GNOME.

mod app;
mod dbus;
mod state;
mod ui;

use relm4::RelmApp;
use uti_core::dbus::APP_BUS_NAME;

fn main() {
    let minimized = std::env::args().skip(1).any(|a| a == "--minimized");
    if std::env::args().skip(1).any(|a| a == "--version" || a == "-V" || a == "-v") {
        println!("uti {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // GApplication owns APP_BUS_NAME, which also makes uti single-instance.
    // Arguments are parsed above, so GTK gets none.
    RelmApp::new(APP_BUS_NAME)
        .with_args(Vec::new())
        .visible_on_activate(false)
        .run::<app::App>(app::Init { minimized });
}
