//! Actions exposed through `org.freedesktop.Application.ActivateAction`

use super::AppBus;
use crate::settings::open_settings_window;
use crate::updater::open_update_dialog;
use crate::window::{self, PinState};
use tauri::{AppHandle, Manager};

/// An action the extension's panel menu (or `gdbus`) can activate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Toggle,
    Show,
    Hide,
    Settings,
    CheckUpdates,
    Quit,
}

impl Action {
    /// Parses a GAction name. `open-settings` is accepted as an alias of `settings`.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "toggle" => Some(Self::Toggle),
            "show" => Some(Self::Show),
            "hide" => Some(Self::Hide),
            "settings" | "open-settings" => Some(Self::Settings),
            "check-updates" => Some(Self::CheckUpdates),
            "quit" => Some(Self::Quit),
            _ => None,
        }
    }
}

/// Runs `action` against the running app.
pub(super) async fn run(app: &AppHandle, action: Action) {
    match action {
        Action::Settings => return open_settings_window(app),
        Action::CheckUpdates => return open_update_dialog(app.clone()),
        Action::Quit => return app.exit(0),
        Action::Toggle | Action::Show | Action::Hide => {}
    }

    let Some(main) = app.get_webview_window("main") else {
        return;
    };
    let bus = app.state::<AppBus>();
    match action {
        Action::Toggle => {
            let pinned = app.state::<PinState>().is_pinned.clone();
            window::toggle(&main, &pinned, &bus).await;
        }
        Action::Show => window::show(&main, &bus).await,
        Action::Hide => {
            let _ = main.hide();
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_actions() {
        assert_eq!(Action::parse("toggle"), Some(Action::Toggle));
        assert_eq!(Action::parse("show"), Some(Action::Show));
        assert_eq!(Action::parse("hide"), Some(Action::Hide));
        assert_eq!(Action::parse("settings"), Some(Action::Settings));
        assert_eq!(Action::parse("open-settings"), Some(Action::Settings));
        assert_eq!(Action::parse("check-updates"), Some(Action::CheckUpdates));
        assert_eq!(Action::parse("quit"), Some(Action::Quit));
    }

    #[test]
    fn rejects_unknown_actions() {
        assert_eq!(Action::parse(""), None);
        assert_eq!(Action::parse("Toggle"), None);
        assert_eq!(Action::parse("pinned"), None);
    }
}
