//! Field values of the launcher item editor (no widgets)

use gtk::glib;

use super::HistorySource;

/// Text of an optional field: trimmed, `None` when empty.
pub fn optional(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// A link as typed: trimmed, `None` without a scheme (`https://…`).
pub fn url(text: &str) -> Option<String> {
    let text = text.trim();
    super::external::uri_list(text)
        .filter(|u| u.len() == 1)
        .map(|_| text.to_owned())
}

/// A path as typed, saved with `~` under `home`; `None` when empty.
pub fn path(text: &str, home: &std::path::Path) -> Option<String> {
    optional(text).map(|p| super::places::contract(&p, home))
}

/// Arguments as typed in one line, split like a shell would; `None` when
/// the quoting is unbalanced.
pub fn parse_args(text: &str) -> Option<Vec<String>> {
    if text.trim().is_empty() {
        return Some(Vec::new());
    }
    let argv = glib::shell_parse_argv(text).ok()?;
    Some(argv.into_iter().map(|a| a.to_string_lossy().into_owned()).collect())
}

/// Arguments as one line that [`parse_args`] reads back.
pub fn format_args(args: &[String]) -> String {
    args.iter()
        .map(|a| {
            if !a.is_empty() && a.chars().all(|c| c.is_alphanumeric() || "-_./=:,+@%".contains(c)) {
                a.clone()
            } else {
                glib::shell_quote(a).to_string_lossy().into_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Where a command's recent files come from, as the editor offers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum History {
    None,
    /// The system history, filtered by application name.
    AppName,
    /// An application's own XBEL file.
    XbelFile,
    /// A VS Code `state.vscdb`.
    Vscode,
}

impl History {
    pub const ALL: [History; 4] = [Self::None, Self::AppName, Self::XbelFile, Self::Vscode];

    /// The choice and its text field for `source`.
    pub fn of(source: Option<&HistorySource>) -> (Self, String) {
        match source {
            None => (Self::None, String::new()),
            Some(HistorySource::RecentlyUsed {
                path: Some(path), ..
            }) => (Self::XbelFile, path.clone()),
            Some(HistorySource::RecentlyUsed { app_name, .. }) => {
                (Self::AppName, app_name.clone().unwrap_or_default())
            },
            Some(HistorySource::Vscode { path }) => (Self::Vscode, path.clone()),
        }
    }

    /// The source for this choice and text; `Err` when the text is needed
    /// but empty.
    pub fn source(self, text: &str) -> Result<Option<HistorySource>, ()> {
        let value = optional(text);
        Ok(match (self, value) {
            (Self::None, _) => None,
            (_, None) => return Err(()),
            (Self::AppName, Some(app_name)) => Some(HistorySource::RecentlyUsed {
                app_name: Some(app_name),
                path: None,
            }),
            (Self::XbelFile, Some(path)) => Some(HistorySource::RecentlyUsed {
                app_name: None,
                path: Some(path),
            }),
            (Self::Vscode, Some(path)) => Some(HistorySource::Vscode { path }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_fields_are_trimmed() {
        assert_eq!(optional("  Code "), Some("Code".to_owned()));
        assert_eq!(optional("   "), None);
    }

    #[test]
    fn links_need_a_scheme_and_paths_are_contracted() {
        assert_eq!(url(" https://example.com/a "), Some("https://example.com/a".to_owned()));
        assert_eq!(url("example.com"), None);
        assert_eq!(url("https://a\nhttps://b"), None);
        let home = std::path::Path::new("/home/u");
        assert_eq!(path(" /home/u/x ", home), Some("~/x".to_owned()));
        assert_eq!(path("trash:///", home), Some("trash:///".to_owned()));
        assert_eq!(path(" ", home), None);
    }

    #[test]
    fn args_round_trip_through_one_line() {
        let args =
            vec!["--new-window".to_owned(), "a b".to_owned(), "it's".to_owned(), String::new()];
        assert_eq!(parse_args(&format_args(&args)), Some(args));
        assert_eq!(format_args(&["-n".to_owned(), "~/x.txt".to_owned()]), "-n '~/x.txt'");
        assert_eq!(parse_args("  "), Some(Vec::new()));
        assert_eq!(parse_args("\"open"), None);
    }

    #[test]
    fn history_choices_round_trip() {
        let sources = [
            None,
            Some(HistorySource::RecentlyUsed {
                app_name: Some("org.gnome.Nautilus".to_owned()),
                path: None,
            }),
            Some(HistorySource::RecentlyUsed {
                app_name: None,
                path: Some("~/x.xbel".to_owned()),
            }),
            Some(HistorySource::Vscode {
                path: "~/state.vscdb".to_owned(),
            }),
        ];
        for source in sources {
            let (choice, text) = History::of(source.as_ref());
            assert_eq!(choice.source(&text), Ok(source));
        }
        assert_eq!(History::Vscode.source(" "), Err(()));
        assert_eq!(History::None.source(""), Ok(None));
    }
}
