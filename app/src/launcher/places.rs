//! Paths of `file` and `directory` nodes and the rows of a directory level
//! (no GIO types)

use std::path::Path;

/// Rows a directory level lists at most.
pub const LISTING_CAP: usize = 200;

/// A directory entry as read from disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirItem {
    pub name: String,
    /// Its path, or URI in a folder given by URI.
    pub path: String,
    pub dir: bool,
}

/// The rows of a directory level: folders first, then files, each by name
/// ignoring case; hidden (`.x`) and backup (`x~`) entries left out; at most
/// `cap`, with the number of shown entries left out past it.
pub fn listing(mut items: Vec<DirItem>, cap: usize) -> (Vec<DirItem>, usize) {
    items.retain(|i| !i.name.starts_with('.') && !i.name.ends_with('~'));
    items.sort_by(|a, b| {
        b.dir
            .cmp(&a.dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    let more = items.len().saturating_sub(cap);
    items.truncate(cap);
    (items, more)
}

/// `path` as saved: `~/x` under `home`, else unchanged.
pub fn contract(path: &str, home: &Path) -> String {
    let home = home.to_string_lossy();
    match path.strip_prefix(home.as_ref()) {
        Some("") => "~".to_string(),
        Some(rest) if rest.starts_with('/') => format!("~{rest}"),
        _ => path.to_string(),
    }
}

/// `path` as used: `~` and `~/x` under `home`, else unchanged.
pub fn expand(path: &str, home: &Path) -> String {
    match path.strip_prefix('~') {
        Some("") => home.to_string_lossy().into_owned(),
        Some(rest) if rest.starts_with('/') => format!("{}{rest}", home.to_string_lossy()),
        _ => path.to_string(),
    }
}

/// `path` is a URI (`trash:///`), not a file path.
pub fn is_uri(path: &str) -> bool {
    !path.starts_with('/') && !path.starts_with('~') && path.contains("://")
}

/// Last component of a path or URI, for a default name.
pub fn base_name(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    let last = trimmed.rsplit('/').next().unwrap_or(trimmed);
    if last.is_empty() || trimmed.ends_with(':') {
        path.to_string()
    } else {
        percent_encoding::percent_decode_str(last).decode_utf8_lossy().into_owned()
    }
}

/// A GTK bookmark: a local folder and its label, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bookmark {
    pub path: String,
    pub label: Option<String>,
}

/// Local folders of a GTK `bookmarks` file (`file:///x Label` per line);
/// remote ones are left out.
pub fn bookmarks(text: &str) -> Vec<Bookmark> {
    text.lines()
        .filter_map(|line| {
            let (uri, label) = line.split_once(' ').unwrap_or((line, ""));
            let path = uri.strip_prefix("file://")?;
            let path = percent_encoding::percent_decode_str(path).decode_utf8().ok()?;
            Some(Bookmark {
                path: path.into_owned(),
                label: crate::launcher::form::optional(label),
            })
        })
        .collect()
}

/// An XDG user directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserDir {
    Desktop,
    Documents,
    Downloads,
    Music,
    Pictures,
    Videos,
    Public,
    Templates,
}

impl UserDir {
    /// Theme icon names, best first.
    pub fn icons(self) -> &'static [&'static str] {
        match self {
            Self::Desktop => &["user-desktop", "folder"],
            Self::Documents => &["folder-documents", "folder"],
            Self::Downloads => &["folder-download", "folder"],
            Self::Music => &["folder-music", "folder"],
            Self::Pictures => &["folder-pictures", "folder"],
            Self::Videos => &["folder-videos", "folder"],
            Self::Public => &["folder-publicshare", "folder"],
            Self::Templates => &["folder-templates", "folder"],
        }
    }
}

/// The user directories to offer: set and not the home directory itself
/// (XDG points unused ones there), each path once.
pub fn user_dirs(dirs: &[(UserDir, Option<String>)], home: &str) -> Vec<(UserDir, String)> {
    let mut out: Vec<(UserDir, String)> = Vec::new();
    for (dir, path) in dirs {
        let Some(path) = path else { continue };
        if path.trim_end_matches('/') != home.trim_end_matches('/')
            && !out.iter().any(|(_, p)| p == path)
        {
            out.push((*dir, path.clone()));
        }
    }
    out
}

/// Theme icon names of a `directory` node's folder, best first: home,
/// trash and user directories have their own.
pub fn dir_icons(path: &str, home: &Path, user: &[(UserDir, String)]) -> &'static [&'static str] {
    let path = expand(path, home);
    if path.starts_with("trash:") {
        return &["user-trash", "folder"];
    }
    if Path::new(&path) == home {
        return &["user-home", "folder"];
    }
    user.iter()
        .find(|(_, p)| Path::new(p) == Path::new(&path))
        .map_or(&["folder"], |(d, _)| d.icons())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, dir: bool) -> DirItem {
        DirItem {
            name: name.to_string(),
            path: format!("/d/{name}"),
            dir,
        }
    }

    fn names(items: &[DirItem]) -> Vec<&str> {
        items.iter().map(|i| i.name.as_str()).collect()
    }

    #[test]
    fn folders_come_first_by_name_ignoring_case() {
        let (rows, more) = listing(
            vec![
                item("b.txt", false),
                item("Zeta", true),
                item("A.pdf", false),
                item("alpha", true),
            ],
            10,
        );
        assert_eq!(names(&rows), ["alpha", "Zeta", "A.pdf", "b.txt"]);
        assert_eq!(more, 0);
    }

    #[test]
    fn hidden_and_backup_files_are_left_out() {
        let (rows, _) = listing(
            vec![
                item(".git", true),
                item(".bashrc", false),
                item("notes~", false),
                item("x", false),
            ],
            10,
        );
        assert_eq!(names(&rows), ["x"]);
    }

    #[test]
    fn rows_past_the_cap_are_counted() {
        let items = (0..5).map(|i| item(&format!("f{i}"), false)).collect();
        let (rows, more) = listing(items, 3);
        assert_eq!(names(&rows), ["f0", "f1", "f2"]);
        assert_eq!(more, 2);
    }

    #[test]
    fn home_paths_are_saved_with_a_tilde() {
        let home = Path::new("/home/u");
        assert_eq!(contract("/home/u/Documents/a b", home), "~/Documents/a b");
        assert_eq!(contract("/home/u", home), "~");
        assert_eq!(contract("/home/user2/x", home), "/home/user2/x");
        assert_eq!(contract("/etc", home), "/etc");
        assert_eq!(expand("~/Documents", home), "/home/u/Documents");
        assert_eq!(expand("~", home), "/home/u");
        assert_eq!(expand("~x/y", home), "~x/y");
        assert_eq!(expand("trash:///", home), "trash:///");
    }

    #[test]
    fn uris_and_names() {
        assert!(is_uri("trash:///"));
        assert!(!is_uri("~/x"));
        assert!(!is_uri("/a://b"));
        assert_eq!(base_name("/home/u/report.pdf"), "report.pdf");
        assert_eq!(base_name("~/Documents/"), "Documents");
        assert_eq!(base_name("file:///tmp/a%20b"), "a b");
        assert_eq!(base_name("trash:///"), "trash:///");
        assert_eq!(base_name("/"), "/");
    }

    #[test]
    fn local_bookmarks_with_labels() {
        let marks = bookmarks(
            "file:///home/u/Projects\nfile:///home/u/My%20Docs Docs\nsftp://host/x Server\n\n",
        );
        assert_eq!(
            marks,
            [
                Bookmark {
                    path: "/home/u/Projects".to_string(),
                    label: None
                },
                Bookmark {
                    path: "/home/u/My Docs".to_string(),
                    label: Some("Docs".to_string())
                },
            ]
        );
    }

    #[test]
    fn user_dirs_skip_unset_home_and_duplicates() {
        let dirs = [
            (UserDir::Desktop, Some("/home/u/Desktop".to_string())),
            (UserDir::Documents, None),
            (UserDir::Public, Some("/home/u/".to_string())),
            (UserDir::Templates, Some("/home/u/Desktop".to_string())),
            (UserDir::Music, Some("/home/u/Music".to_string())),
        ];
        assert_eq!(
            user_dirs(&dirs, "/home/u"),
            [
                (UserDir::Desktop, "/home/u/Desktop".to_string()),
                (UserDir::Music, "/home/u/Music".to_string())
            ]
        );
    }

    #[test]
    fn special_folders_have_their_icons() {
        let home = Path::new("/home/u");
        let user = [(UserDir::Music, "/home/u/Music".to_string())];
        assert_eq!(dir_icons("~", home, &user)[0], "user-home");
        assert_eq!(dir_icons("trash:///", home, &user)[0], "user-trash");
        assert_eq!(dir_icons("~/Music", home, &user)[0], "folder-music");
        assert_eq!(dir_icons("/tmp", home, &user), ["folder"]);
    }
}
