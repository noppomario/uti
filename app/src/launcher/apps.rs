//! Installed apps grouped by freedesktop main category (no GIO types)

use serde::{Deserialize, Serialize};

/// What the launcher needs of an installed app (`gio::DesktopAppInfo`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppMeta {
    /// Desktop file id: `org.gnome.Nautilus.desktop`.
    pub id: String,
    pub name: String,
    pub description: String,
    /// Program of its command line.
    pub exec: Option<String>,
    /// `Categories` of its desktop file.
    pub categories: Vec<String>,
}

/// A freedesktop main category, in display order; serialized by its
/// freedesktop name (`AudioVideo`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Category {
    Utility,
    Education,
    Game,
    Graphics,
    Network,
    Office,
    Development,
    Science,
    Settings,
    AudioVideo,
    System,
    Other,
}

impl Category {
    /// The category a desktop file's `Categories` value names, if a main one.
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "Utility" => Self::Utility,
            "Education" => Self::Education,
            "Game" => Self::Game,
            "Graphics" => Self::Graphics,
            "Network" => Self::Network,
            "Office" => Self::Office,
            "Development" => Self::Development,
            "Science" => Self::Science,
            "Settings" => Self::Settings,
            "AudioVideo" | "Audio" | "Video" => Self::AudioVideo,
            "System" => Self::System,
            _ => return None,
        })
    }

    /// Category of an app: its first main category, else [`Category::Other`].
    pub fn of(app: &AppMeta) -> Self {
        app.categories.iter().find_map(|c| Self::parse(c)).unwrap_or(Self::Other)
    }

    /// Menu name, as in GNOME's classic menus.
    pub fn label(self) -> &'static str {
        match self {
            Self::Utility => "Accessories",
            Self::Education => "Education",
            Self::Game => "Games",
            Self::Graphics => "Graphics",
            Self::Network => "Internet",
            Self::Office => "Office",
            Self::Development => "Programming",
            Self::Science => "Science",
            Self::Settings => "Settings",
            Self::AudioVideo => "Sound and Video",
            Self::System => "System Tools",
            Self::Other => "Other",
        }
    }

    /// Theme icon names, best first: the icon naming spec's name, then
    /// names Adwaita has.
    pub fn icons(self) -> &'static [&'static str] {
        match self {
            Self::Utility => {
                &["applications-utilities-symbolic", "applications-accessories-symbolic"]
            },
            Self::Education => {
                &["applications-education-symbolic", "accessories-dictionary-symbolic"]
            },
            Self::Game => &["applications-games-symbolic"],
            Self::Graphics => &["applications-graphics-symbolic"],
            Self::Network => &["applications-internet-symbolic", "web-browser-symbolic"],
            Self::Office => &["applications-office-symbolic", "x-office-document-symbolic"],
            Self::Development => {
                &["applications-development-symbolic", "utilities-terminal-symbolic"]
            },
            Self::Science => &["applications-science-symbolic"],
            Self::Settings => &["preferences-system-symbolic"],
            Self::AudioVideo => &["applications-multimedia-symbolic"],
            Self::System => &["applications-system-symbolic"],
            Self::Other => &["applications-other-symbolic", "view-app-grid-symbolic"],
        }
    }
}

/// Indices of `apps` by category: categories in display order, empty ones
/// left out, apps by name.
pub fn group(apps: &[AppMeta]) -> Vec<(Category, Vec<usize>)> {
    let mut sorted: Vec<(Category, String, usize)> = apps
        .iter()
        .enumerate()
        .map(|(i, a)| (Category::of(a), a.name.to_lowercase(), i))
        .collect();
    sorted.sort();
    let mut groups: Vec<(Category, Vec<usize>)> = Vec::new();
    for (category, _, i) in sorted {
        match groups.last_mut() {
            Some((c, members)) if *c == category => members.push(i),
            _ => groups.push((category, vec![i])),
        }
    }
    groups
}

/// Apps of one category, by name.
pub fn in_category(apps: &[AppMeta], category: Category) -> Vec<usize> {
    group(apps)
        .into_iter()
        .find(|(c, _)| *c == category)
        .map(|(_, a)| a)
        .unwrap_or_default()
}

/// Names an app may be recorded under in `recently-used.xbel`: its desktop
/// id without `.desktop`, its program and its display name.
pub fn xbel_names(app: &AppMeta) -> Vec<String> {
    let mut names = vec![app.id.strip_suffix(".desktop").unwrap_or(&app.id).to_string()];
    if let Some(exec) = &app.exec {
        names.push(exec.rsplit('/').next().unwrap_or(exec).to_string());
    }
    names.push(app.name.clone());
    names.dedup();
    names.retain(|n| !n.is_empty());
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(id: &str, name: &str, categories: &str) -> AppMeta {
        AppMeta {
            id: id.to_string(),
            name: name.to_string(),
            exec: None,
            description: String::new(),
            categories: categories.split(';').filter(|c| !c.is_empty()).map(String::from).collect(),
        }
    }

    #[test]
    fn the_first_main_category_wins() {
        assert_eq!(Category::of(&app("a", "A", "GTK;Development;Utility;")), Category::Development);
        assert_eq!(Category::of(&app("a", "A", "Audio;Player;")), Category::AudioVideo);
        assert_eq!(Category::of(&app("a", "A", "GNOME;GTK;")), Category::Other);
        assert_eq!(Category::of(&app("a", "A", "")), Category::Other);
    }

    #[test]
    fn groups_are_in_display_order_with_apps_by_name() {
        let apps = [
            app("t", "terminal", "System;"),
            app("f", "Firefox", "Network;WebBrowser;"),
            app("x", "Mystery", ""),
            app("c", "Chrome", "Network;"),
            app("e", "Calculator", "Utility;"),
        ];
        assert_eq!(
            group(&apps),
            [
                (Category::Utility, vec![4]),
                (Category::Network, vec![3, 1]),
                (Category::System, vec![0]),
                (Category::Other, vec![2]),
            ]
        );
        assert_eq!(in_category(&apps, Category::Network), [3, 1]);
        assert!(in_category(&apps, Category::Game).is_empty());
    }

    #[test]
    fn every_category_has_a_label_and_an_icon() {
        let all = [
            Category::Utility,
            Category::Education,
            Category::Game,
            Category::Graphics,
            Category::Network,
            Category::Office,
            Category::Development,
            Category::Science,
            Category::Settings,
            Category::AudioVideo,
            Category::System,
            Category::Other,
        ];
        let labels: Vec<&str> = all.iter().map(|c| c.label()).collect();
        let mut sorted = labels.clone();
        sorted[..11].sort();
        assert_eq!(labels, sorted, "main categories are in label order, Other last");
        assert!(all.iter().all(|c| !c.icons().is_empty()));
    }

    #[test]
    fn xbel_names_cover_id_program_and_name() {
        let mut loupe = app("org.gnome.Loupe.desktop", "Image Viewer", "Graphics;");
        loupe.exec = Some("/usr/bin/loupe".to_string());
        assert_eq!(xbel_names(&loupe), ["org.gnome.Loupe", "loupe", "Image Viewer"]);
        let firefox = app("firefox.desktop", "firefox", "");
        assert_eq!(xbel_names(&firefox), ["firefox"]);
    }
}
