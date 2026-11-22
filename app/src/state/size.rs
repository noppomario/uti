//! Per-size layout rules for `theme.size` (no GTK types)

/// Main window size (`theme.size`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    /// Dense, small popup: no key hints, search bar only while searching.
    Minimal,
    Normal,
    Wide,
}

impl Size {
    /// Every size, smallest first.
    pub const ALL: [Size; 3] = [Size::Minimal, Size::Normal, Size::Wide];

    /// Parses a `theme.size` value; unknown values fall back to `Normal`.
    pub fn parse(value: &str) -> Size {
        match value {
            "minimal" => Size::Minimal,
            "wide" => Size::Wide,
            _ => Size::Normal,
        }
    }

    /// Default window width and height.
    pub fn window_size(self) -> (i32, i32) {
        match self {
            // Clibor's default: 25 rows of 24px below the header.
            Size::Minimal => (300, 640),
            Size::Normal => (320, 600),
            Size::Wide => (500, 700),
        }
    }

    /// CSS class on the main window.
    pub fn css_class(self) -> &'static str {
        match self {
            Size::Minimal => "density-minimal",
            Size::Normal => "density-normal",
            Size::Wide => "density-wide",
        }
    }

    /// Whether the key hints footer is shown.
    pub fn show_footer(self) -> bool {
        self != Size::Minimal
    }

    /// Whether the search bar is shown, given whether it has focus and
    /// whether the current tab has a query.
    pub fn show_search(self, focused: bool, has_query: bool) -> bool {
        self != Size::Minimal || focused || has_query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_falls_back_to_normal() {
        assert_eq!(Size::parse("minimal"), Size::Minimal);
        assert_eq!(Size::parse("normal"), Size::Normal);
        assert_eq!(Size::parse("wide"), Size::Wide);
        assert_eq!(Size::parse("huge"), Size::Normal);
    }

    #[test]
    fn test_widths_grow_and_classes_are_distinct() {
        // Heights do not: minimal is dense, not short (25 rows like Clibor).
        let widths = Size::ALL.map(|s| s.window_size().0);
        assert!(widths.windows(2).all(|w| w[0] < w[1]));
        let classes = Size::ALL.map(Size::css_class);
        assert!(classes.iter().enumerate().all(|(i, c)| !classes[..i].contains(c)));
    }

    #[test]
    fn test_minimal_hides_footer() {
        assert!(!Size::Minimal.show_footer());
        assert!(Size::Normal.show_footer());
        assert!(Size::Wide.show_footer());
    }

    #[test]
    fn test_minimal_shows_search_only_while_searching() {
        assert!(!Size::Minimal.show_search(false, false));
        assert!(Size::Minimal.show_search(true, false));
        assert!(Size::Minimal.show_search(false, true));
        assert!(Size::Normal.show_search(false, false));
        assert!(Size::Wide.show_search(false, false));
    }
}
