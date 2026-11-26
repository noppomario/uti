//! Launcher search ranking (no GIO types)

/// Maximum number of search results.
pub const MAX_RESULTS: usize = 20;

/// Name and description of a launcher item or installed app, as searched.
pub struct SearchText<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

/// Indices of `items` matching `query` (name or description, case-insensitive
/// substring), ordered exact name match, then name prefix, then by name;
/// at most [`MAX_RESULTS`]. Duplicate names keep the first entry, so items
/// listed first (the launcher tree before installed apps) win.
pub fn rank(items: &[SearchText<'_>], query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let mut seen = std::collections::HashSet::new();
    let mut hits: Vec<(u8, String, usize)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, app)| {
            let name = app.name.to_lowercase();
            let matched = name.contains(&query) || app.description.to_lowercase().contains(&query);
            if !matched || !seen.insert(name.clone()) {
                return None;
            }
            let rank = if name == query {
                0
            } else if name.starts_with(&query) {
                1
            } else {
                2
            };
            Some((rank, name, i))
        })
        .collect();
    hits.sort();
    hits.into_iter().take(MAX_RESULTS).map(|(_, _, i)| i).collect()
}

/// The installed app a launcher command starts, to borrow its icon: the
/// one whose command line has the same program (by file name), or among
/// several, the one with the same arguments too. `apps` holds each app's
/// command line without field codes (`%U`); `None` when unclear.
pub fn match_command(command: &str, args: &[String], apps: &[Vec<String>]) -> Option<usize> {
    let program = |p: &str| p.rsplit('/').next().unwrap_or(p).to_owned();
    let name = program(command);
    let candidates: Vec<usize> = (0..apps.len())
        .filter(|&i| apps[i].first().is_some_and(|p| program(p) == name))
        .collect();
    match candidates.as_slice() {
        [one] => Some(*one),
        many => {
            let same: Vec<usize> =
                many.iter().copied().filter(|&i| apps[i][1..] == *args).collect();
            (same.len() == 1).then(|| same[0])
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apps(list: &[(&'static str, &'static str)]) -> Vec<SearchText<'static>> {
        list.iter()
            .map(|(name, description)| SearchText { name, description })
            .collect()
    }

    fn names<'a>(apps: &'a [SearchText<'a>], idx: &[usize]) -> Vec<&'a str> {
        idx.iter().map(|&i| apps[i].name).collect()
    }

    fn argv(lines: &[&str]) -> Vec<Vec<String>> {
        lines.iter().map(|l| l.split(' ').map(String::from).collect()).collect()
    }

    #[test]
    fn commands_match_apps_by_program_name() {
        let apps = argv(&["nautilus --new-window", "/usr/share/code/code", "gnome-text-editor"]);
        assert_eq!(match_command("nautilus", &[], &apps), Some(0));
        assert_eq!(match_command("/usr/bin/code", &["-n".into()], &apps), Some(1));
        assert_eq!(match_command("ptyxis", &[], &apps), None);
    }

    #[test]
    fn shared_programs_need_the_same_arguments() {
        let apps =
            argv(&["libreoffice --writer", "libreoffice --calc", "flatpak run a", "flatpak run b"]);
        assert_eq!(match_command("libreoffice", &["--calc".into()], &apps), Some(1));
        assert_eq!(match_command("libreoffice", &[], &apps), None);
        assert_eq!(match_command("flatpak", &["run".into(), "b".into()], &apps), Some(3));
    }

    #[test]
    fn exact_then_prefix_then_alphabetical() {
        let a = apps(&[
            ("Text Editor", "Edit text files"),
            ("Files", "Access and organize files"),
            ("Firefox", "Browse the web"),
            ("Fi", "Exact"),
            ("Profile", "Has fi inside"),
        ]);
        assert_eq!(
            names(&a, &rank(&a, "fi")),
            vec!["Fi", "Files", "Firefox", "Profile", "Text Editor"]
        );
    }

    #[test]
    fn matches_description_case_insensitively() {
        let a = apps(&[("Nautilus", "Access and organize FILES"), ("Calculator", "Math")]);
        assert_eq!(names(&a, &rank(&a, "files")), vec!["Nautilus"]);
    }

    #[test]
    fn empty_query_matches_nothing() {
        let a = apps(&[("Files", "")]);
        assert!(rank(&a, "").is_empty());
        assert!(rank(&a, "   ").is_empty());
    }

    #[test]
    fn duplicates_are_dropped_and_results_capped() {
        let mut list: Vec<(&'static str, &'static str)> = vec![("App", "x"), ("App", "y")];
        let many: Vec<String> = (0..30).map(|i| format!("App {i:02}")).collect();
        let many: Vec<&'static str> =
            many.into_iter().map(|s| &*Box::leak(s.into_boxed_str())).collect();
        list.extend(many.iter().map(|n| (*n, "")));
        let a = apps(&list);
        let r = rank(&a, "app");
        assert_eq!(r.len(), MAX_RESULTS);
        assert_eq!(r[0], 0);
        assert!(!r.contains(&1));
    }
}
