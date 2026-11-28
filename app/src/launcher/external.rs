//! Nodes for items dropped from other apps (no GIO types)

use std::path::Path;

use super::places;
use super::NodeKind;

/// What a dropped local URI is on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Local {
    Dir,
    File,
    /// Not readable (gone, no permission): nothing is added.
    Missing,
}

/// URIs of a `text/uri-list` (or of plain text holding URIs): one per
/// line, comments left out. `None` when a line is not a URI.
pub fn uri_list(text: &str) -> Option<Vec<String>> {
    let uris: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();
    let valid = !uris.is_empty() && uris.iter().all(|u| scheme(u).is_some());
    valid.then_some(uris)
}

/// The scheme of a URI (`https`), if it has one.
fn scheme(uri: &str) -> Option<&str> {
    let (scheme, rest) = uri.split_once(':')?;
    let ok = !scheme.is_empty()
        && scheme.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && scheme.chars().all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        && !rest.is_empty();
    ok.then_some(scheme)
}

/// Host of a URL, for its default name (`example.com`), else the URL.
pub fn host(url: &str) -> String {
    url.split_once("://")
        .map(|(_, rest)| rest.split(['/', '?', '#']).next().unwrap_or(rest))
        .map(|h| h.rsplit('@').next().unwrap_or(h))
        .filter(|h| !h.is_empty())
        .unwrap_or(url)
        .to_string()
}

/// The node for a dropped URI: a folder as `directory`, the desktop file
/// of an installed app (`installed` says by desktop id) as `app`, another
/// file as `file`, anything not local as `url`. `local` says what a
/// `file://` path is; paths under `home` are saved with `~`.
pub fn node_for(
    uri: &str,
    local: impl Fn(&str) -> Local,
    installed: impl Fn(&str) -> bool,
    home: &Path,
) -> Option<NodeKind> {
    let Some(path) = uri.strip_prefix("file://") else {
        scheme(uri)?;
        return Some(NodeKind::Url {
            url: uri.to_string(),
            name: None,
            icon: None,
        });
    };
    let path = percent_encoding::percent_decode_str(path).decode_utf8().ok()?.into_owned();
    let saved = places::contract(&path, home);
    match local(&path) {
        Local::Missing => None,
        Local::Dir => Some(NodeKind::Directory {
            path: saved,
            name: None,
            icon: None,
        }),
        Local::File => {
            let id = places::base_name(&path);
            if id.ends_with(".desktop") && installed(&id) {
                return Some(NodeKind::App {
                    desktop_id: id,
                    name: None,
                    icon: None,
                });
            }
            Some(NodeKind::File {
                path: saved,
                name: None,
                icon: None,
            })
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = "/home/u";

    fn node(uri: &str) -> Option<NodeKind> {
        node_for(
            uri,
            |p| {
                if p.ends_with('/') || p.ends_with("Projects") {
                    Local::Dir
                } else if p.contains("gone") {
                    Local::Missing
                } else {
                    Local::File
                }
            },
            |id| id == "org.gnome.Nautilus.desktop",
            Path::new(HOME),
        )
    }

    #[test]
    fn folders_files_and_apps_by_what_they_are() {
        assert_eq!(
            node("file:///home/u/Projects"),
            Some(NodeKind::Directory {
                path: "~/Projects".to_string(),
                name: None,
                icon: None
            })
        );
        assert_eq!(
            node("file:///tmp/a%20b.pdf"),
            Some(NodeKind::File {
                path: "/tmp/a b.pdf".to_string(),
                name: None,
                icon: None
            })
        );
        assert_eq!(
            node("file:///usr/share/applications/org.gnome.Nautilus.desktop"),
            Some(NodeKind::App {
                desktop_id: "org.gnome.Nautilus.desktop".to_string(),
                name: None,
                icon: None
            })
        );
        assert!(
            matches!(node("file:///tmp/other.desktop"), Some(NodeKind::File { .. })),
            "not an installed app"
        );
        assert_eq!(node("file:///tmp/gone.txt"), None);
    }

    #[test]
    fn other_uris_are_links() {
        assert_eq!(
            node("https://example.com/a?b"),
            Some(NodeKind::Url {
                url: "https://example.com/a?b".to_string(),
                name: None,
                icon: None
            })
        );
        assert_eq!(node("not a uri"), None);
    }

    #[test]
    fn uri_lists_take_every_uri_or_none() {
        assert_eq!(
            uri_list("# comment\r\nfile:///a\r\nhttps://b.org/\r\n"),
            Some(vec!["file:///a".to_string(), "https://b.org/".to_string()])
        );
        assert_eq!(uri_list("https://example.com"), Some(vec!["https://example.com".to_string()]));
        assert_eq!(uri_list("hello world"), None);
        assert_eq!(uri_list("C:"), None);
        assert_eq!(uri_list(""), None);
    }

    #[test]
    fn hosts_name_links() {
        assert_eq!(host("https://user@example.com:8080/x?y"), "example.com:8080");
        assert_eq!(host("https://example.com"), "example.com");
        assert_eq!(host("mailto:a@b.org"), "mailto:a@b.org");
    }
}
