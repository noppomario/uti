//! Power actions and the D-Bus calls behind them (no GIO types)

use serde::{Deserialize, Serialize};

/// A session or power action of the `power` node; serialized in camelCase
/// (`logOut`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PowerAction {
    Lock,
    Suspend,
    LogOut,
    Restart,
    PowerOff,
}

/// A D-Bus method call without a reply value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DBusCall {
    /// System bus (else the session bus).
    pub system: bool,
    pub dest: &'static str,
    pub path: &'static str,
    pub interface: &'static str,
    pub method: &'static str,
    pub args: CallArgs,
}

/// Arguments of a [`DBusCall`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallArgs {
    None,
    Bool(bool),
    U32(u32),
}

const SESSION_MANAGER: (&str, &str, &str) = (
    "org.gnome.SessionManager",
    "/org/gnome/SessionManager",
    "org.gnome.SessionManager",
);

impl PowerAction {
    pub const ALL: [PowerAction; 5] =
        [Self::Lock, Self::Suspend, Self::LogOut, Self::Restart, Self::PowerOff];

    pub fn label(self) -> &'static str {
        match self {
            Self::Lock => "Lock",
            Self::Suspend => "Suspend",
            Self::LogOut => "Log Out",
            Self::Restart => "Restart",
            Self::PowerOff => "Power Off",
        }
    }

    /// Theme icon names, best first.
    pub fn icons(self) -> &'static [&'static str] {
        match self {
            Self::Lock => &["system-lock-screen-symbolic"],
            Self::Suspend => &["system-suspend-symbolic", "media-playback-pause-symbolic"],
            Self::LogOut => &["system-log-out-symbolic"],
            Self::Restart => &["system-reboot-symbolic"],
            Self::PowerOff => &["system-shutdown-symbolic"],
        }
    }

    /// The call that performs it. Log out, restart and power off go through
    /// GNOME's session manager, which asks for confirmation.
    pub fn call(self) -> DBusCall {
        let session = |method, args| DBusCall {
            system: false,
            dest: SESSION_MANAGER.0,
            path: SESSION_MANAGER.1,
            interface: SESSION_MANAGER.2,
            method,
            args,
        };
        match self {
            Self::Lock => DBusCall {
                system: false,
                dest: "org.gnome.ScreenSaver",
                path: "/org/gnome/ScreenSaver",
                interface: "org.gnome.ScreenSaver",
                method: "Lock",
                args: CallArgs::None,
            },
            Self::Suspend => DBusCall {
                system: true,
                dest: "org.freedesktop.login1",
                path: "/org/freedesktop/login1",
                interface: "org.freedesktop.login1.Manager",
                method: "Suspend",
                // interactive: polkit may ask
                args: CallArgs::Bool(true),
            },
            // mode 0: normal, with the confirmation dialog
            Self::LogOut => session("Logout", CallArgs::U32(0)),
            Self::Restart => session("Reboot", CallArgs::None),
            Self::PowerOff => session("Shutdown", CallArgs::None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_actions_use_gnome_confirmation() {
        let calls: Vec<(&str, &str, CallArgs)> =
            [PowerAction::LogOut, PowerAction::Restart, PowerAction::PowerOff]
                .iter()
                .map(|a| a.call())
                .map(|c| (c.dest, c.method, c.args))
                .collect();
        assert_eq!(
            calls,
            [
                ("org.gnome.SessionManager", "Logout", CallArgs::U32(0)),
                ("org.gnome.SessionManager", "Reboot", CallArgs::None),
                ("org.gnome.SessionManager", "Shutdown", CallArgs::None),
            ]
        );
    }

    #[test]
    fn lock_and_suspend() {
        let lock = PowerAction::Lock.call();
        assert_eq!(
            (lock.system, lock.interface, lock.method),
            (false, "org.gnome.ScreenSaver", "Lock")
        );
        let suspend = PowerAction::Suspend.call();
        assert_eq!(
            (suspend.system, suspend.interface, suspend.method, suspend.args),
            (true, "org.freedesktop.login1.Manager", "Suspend", CallArgs::Bool(true))
        );
    }

    #[test]
    fn every_action_has_a_label_and_an_icon() {
        assert!(PowerAction::ALL.iter().all(|a| !a.label().is_empty() && !a.icons().is_empty()));
    }
}
