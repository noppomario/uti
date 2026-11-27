//! Settings window pages and deep links

use crate::launcher::edit::Place;

/// Settings window pages, in sidebar order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsPage {
    /// General, mouse, clipboard and advanced settings.
    #[default]
    General,
    Appearance,
    Snippets,
    Launcher,
    About,
}

impl SettingsPage {
    pub const ALL: [SettingsPage; 5] =
        [Self::General, Self::Appearance, Self::Snippets, Self::Launcher, Self::About];

    /// Stack page name; also the `sections.<id>` translation key.
    pub fn id(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Appearance => "appearance",
            Self::Snippets => "snippets",
            Self::Launcher => "launcher",
            Self::About => "about",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.id() == id)
    }
}

/// A snippet to open in the snippet editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnippetTarget {
    /// The snippet with this id.
    Edit(String),
    New,
}

/// Kinds of launcher items the editor creates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewNode {
    Folder,
    Command,
}

/// A launcher item to open in the launcher item editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherTarget {
    /// The node with this id.
    Edit(String),
    /// A new node, saved at this place.
    New(NewNode, Place),
}

/// Where the settings window opens.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsTarget {
    pub page: SettingsPage,
    /// Editor to push on the Snippets page.
    pub snippet: Option<SnippetTarget>,
    /// Editor to push on the Launcher page.
    pub launcher: Option<LauncherTarget>,
}

impl SettingsTarget {
    /// The Snippets page with `snippet` open in the editor.
    pub fn snippet(snippet: SnippetTarget) -> Self {
        Self {
            page: SettingsPage::Snippets,
            snippet: Some(snippet),
            launcher: None,
        }
    }

    /// The Launcher page, with `launcher` open in the editor if given.
    pub fn launcher(launcher: Option<LauncherTarget>) -> Self {
        Self {
            page: SettingsPage::Launcher,
            snippet: None,
            launcher,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_and_are_unique() {
        for page in SettingsPage::ALL {
            assert_eq!(SettingsPage::from_id(page.id()), Some(page));
        }
        assert_eq!(SettingsPage::from_id("nope"), None);
    }

    #[test]
    fn snippet_target_opens_the_snippets_page() {
        let t = SettingsTarget::snippet(SnippetTarget::New);
        assert_eq!(t.page, SettingsPage::Snippets);
        assert_eq!(SettingsTarget::default().page, SettingsPage::General);
        assert_eq!(SettingsTarget::launcher(None).page, SettingsPage::Launcher);
    }
}
