//! Items of the special item palette, by category (no GIO types)

use super::apps::{self, AppMeta};
use super::power::PowerAction;
use super::{CommandItem, NodeKind};

/// The program behind GNOME Settings panels.
const CONTROL_CENTER: &str = "gnome-control-center";

/// A palette tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteCategory {
    Basic,
    Applications,
    Power,
    Settings,
}

impl PaletteCategory {
    pub const ALL: [PaletteCategory; 4] =
        [Self::Basic, Self::Applications, Self::Power, Self::Settings];

    pub fn label(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Applications => "Applications",
            Self::Power => "Power and Session",
            Self::Settings => "GNOME Settings",
        }
    }

    /// Theme icon name of its tab.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Basic => "folder-symbolic",
            Self::Applications => "view-app-grid-symbolic",
            Self::Power => "system-shutdown-symbolic",
            Self::Settings => "preferences-system-symbolic",
        }
    }
}

/// A panel of GNOME Settings, from its hidden `gnome-*-panel.desktop` file.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsPanel {
    pub name: String,
    pub icon: Option<String>,
    /// Arguments of `gnome-control-center`: `wifi`, `system about`.
    pub args: Vec<String>,
}

impl SettingsPanel {
    /// The panel a desktop file describes: id `gnome-<x>-panel.desktop`
    /// running `gnome-control-center <panel>..`; `None` for anything else.
    pub fn from_desktop(id: &str, name: &str, icon: Option<&str>, argv: &[String]) -> Option<Self> {
        if !(id.starts_with("gnome-") && id.ends_with("-panel.desktop")) {
            return None;
        }
        let (program, args) = argv.split_first()?;
        let program = program.rsplit('/').next().unwrap_or(program);
        (program == CONTROL_CENTER && !args.is_empty()).then(|| Self {
            name: name.to_string(),
            icon: icon.map(String::from),
            args: args.to_vec(),
        })
    }
}

/// A row of the palette: what it shows and the node it adds.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteItem {
    pub name: String,
    /// Theme icon names, best first.
    pub icons: Vec<String>,
    pub kind: NodeKind,
}

fn item(name: &str, icons: &[&str], kind: NodeKind) -> PaletteItem {
    PaletteItem {
        name: name.to_string(),
        icons: icons.iter().map(|s| s.to_string()).collect(),
        kind,
    }
}

/// The rows of `category`: generated menus first, then their single
/// items; categories of apps only when some app has them; panels by name.
pub fn items(
    category: PaletteCategory,
    apps: &[AppMeta],
    panels: &[SettingsPanel],
) -> Vec<PaletteItem> {
    match category {
        PaletteCategory::Basic => vec![
            item(
                "New Folder",
                &["folder"],
                NodeKind::Folder {
                    name: "New Folder".to_string(),
                    children: Vec::new(),
                },
            ),
            item("Separator", &["list-remove-symbolic"], NodeKind::Separator),
        ],
        PaletteCategory::Applications => {
            let all = item(
                "Applications",
                &["view-app-grid-symbolic"],
                NodeKind::Apps {
                    name: None,
                    category: None,
                },
            );
            let categories = apps::group(apps).into_iter().map(|(c, _)| {
                item(
                    c.label(),
                    c.icons(),
                    NodeKind::Apps {
                        name: None,
                        category: Some(c),
                    },
                )
            });
            std::iter::once(all).chain(categories).collect()
        },
        PaletteCategory::Power => {
            let all = item(
                "Power",
                &["system-shutdown-symbolic"],
                NodeKind::Power {
                    name: None,
                    action: None,
                },
            );
            let actions = PowerAction::ALL.into_iter().map(|a| {
                item(
                    a.label(),
                    a.icons(),
                    NodeKind::Power {
                        name: None,
                        action: Some(a),
                    },
                )
            });
            std::iter::once(all).chain(actions).collect()
        },
        PaletteCategory::Settings => {
            let mut panels: Vec<&SettingsPanel> = panels.iter().collect();
            panels.sort_by_key(|p| p.name.to_lowercase());
            panels
                .into_iter()
                .map(|p| PaletteItem {
                    name: p.name.clone(),
                    icons: vec![p
                        .icon
                        .clone()
                        .unwrap_or_else(|| "preferences-system-symbolic".to_string())],
                    kind: NodeKind::Command(CommandItem {
                        name: p.name.clone(),
                        command: CONTROL_CENTER.to_string(),
                        args: p.args.clone(),
                        history_source: None,
                        icon: p.icon.clone(),
                    }),
                })
                .collect()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::launcher::apps::Category;

    fn app(name: &str, category: &str) -> AppMeta {
        AppMeta {
            id: format!("{name}.desktop"),
            name: name.to_string(),
            categories: vec![category.to_string()],
            ..Default::default()
        }
    }

    fn names(items: &[PaletteItem]) -> Vec<&str> {
        items.iter().map(|i| i.name.as_str()).collect()
    }

    fn argv(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn basic_has_folder_and_separator() {
        let items = items(PaletteCategory::Basic, &[], &[]);
        assert_eq!(names(&items), ["New Folder", "Separator"]);
        assert!(matches!(&items[0].kind, NodeKind::Folder { name, children }
            if name == "New Folder" && children.is_empty()));
        assert_eq!(items[1].kind, NodeKind::Separator);
    }

    #[test]
    fn applications_lists_the_menu_then_categories_that_have_apps() {
        let apps = [app("Gimp", "Graphics"), app("Firefox", "Network"), app("Chess", "Game")];
        let items = items(PaletteCategory::Applications, &apps, &[]);
        assert_eq!(names(&items), ["Applications", "Games", "Graphics", "Internet"]);
        assert_eq!(
            items[0].kind,
            NodeKind::Apps {
                name: None,
                category: None
            }
        );
        assert_eq!(
            items[1].kind,
            NodeKind::Apps {
                name: None,
                category: Some(Category::Game)
            }
        );
        assert_eq!(items[1].icons, ["applications-games-symbolic"]);
    }

    #[test]
    fn power_lists_the_menu_then_each_action() {
        let items = items(PaletteCategory::Power, &[], &[]);
        assert_eq!(names(&items), ["Power", "Lock", "Suspend", "Log Out", "Restart", "Power Off"]);
        assert_eq!(
            items[5].kind,
            NodeKind::Power {
                name: None,
                action: Some(PowerAction::PowerOff)
            }
        );
    }

    #[test]
    fn settings_lists_panels_by_name_as_commands() {
        let panels = [
            SettingsPanel {
                name: "Wi-Fi".to_string(),
                icon: Some("network-wireless-symbolic".to_string()),
                args: vec!["wifi".to_string()],
            },
            SettingsPanel {
                name: "about".to_string(),
                icon: None,
                args: vec!["system".to_string(), "about".to_string()],
            },
        ];
        let items = items(PaletteCategory::Settings, &[], &panels);
        assert_eq!(names(&items), ["about", "Wi-Fi"]);
        assert_eq!(items[0].icons, ["preferences-system-symbolic"]);
        assert!(matches!(&items[0].kind, NodeKind::Command(c) if c.args == ["system", "about"]));
        let NodeKind::Command(c) = &items[1].kind else {
            panic!("not a command");
        };
        assert_eq!((c.name.as_str(), c.command.as_str()), ("Wi-Fi", "gnome-control-center"));
        assert_eq!(c.args, ["wifi"]);
        assert_eq!(c.icon.as_deref(), Some("network-wireless-symbolic"));
    }

    #[test]
    fn panels_come_from_control_center_panel_desktop_files() {
        let wifi = SettingsPanel::from_desktop(
            "gnome-wifi-panel.desktop",
            "Wi-Fi",
            Some("network-wireless-symbolic"),
            &argv(&["gnome-control-center", "wifi"]),
        );
        assert_eq!(wifi.map(|p| p.args), Some(argv(&["wifi"])));
        let about = SettingsPanel::from_desktop(
            "gnome-about-panel.desktop",
            "About",
            None,
            &argv(&["gnome-control-center", "system", "about"]),
        );
        assert_eq!(about.map(|p| p.args), Some(argv(&["system", "about"])));
        let full_path = SettingsPanel::from_desktop(
            "gnome-sound-panel.desktop",
            "Sound",
            None,
            &argv(&["/usr/bin/gnome-control-center", "sound"]),
        );
        assert!(full_path.is_some());
    }

    #[test]
    fn other_desktop_files_are_not_panels() {
        let cc = argv(&["gnome-control-center", "wifi"]);
        assert_eq!(SettingsPanel::from_desktop("org.gnome.Settings.desktop", "S", None, &cc), None);
        let other = argv(&["ibus", "emoji"]);
        assert_eq!(
            SettingsPanel::from_desktop("gnome-emoji-panel.desktop", "E", None, &other),
            None
        );
        let bare = argv(&["gnome-control-center"]);
        assert_eq!(SettingsPanel::from_desktop("gnome-x-panel.desktop", "X", None, &bare), None);
    }
}
