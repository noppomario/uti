//! List scrollbar style for `theme.scrollbar` (no GTK types)

/// How list scrollbars are shown (`theme.scrollbar`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scrollbar {
    /// GTK's overlay scrollbar, shown while scrolling or hovering.
    Overlay,
    /// A classic scrollbar beside the list.
    Always,
    /// No scrollbar; the list still scrolls.
    Hidden,
}

impl Scrollbar {
    /// Every style, in the order the settings window lists them.
    pub const ALL: [Scrollbar; 3] = [Scrollbar::Overlay, Scrollbar::Always, Scrollbar::Hidden];

    /// Parses a `theme.scrollbar` value; unknown values fall back to `Overlay`.
    pub fn parse(value: &str) -> Scrollbar {
        Self::ALL.into_iter().find(|s| s.id() == value).unwrap_or(Scrollbar::Overlay)
    }

    /// `theme.scrollbar` value; also the translation key suffix.
    pub fn id(self) -> &'static str {
        match self {
            Scrollbar::Overlay => "overlay",
            Scrollbar::Always => "always",
            Scrollbar::Hidden => "hidden",
        }
    }

    /// Whether a scrollbar widget is shown at all.
    pub fn visible(self) -> bool {
        self != Scrollbar::Hidden
    }

    /// Whether the scrollbar floats over the rows.
    pub fn overlays(self) -> bool {
        self == Scrollbar::Overlay
    }

    /// CSS class on the main window.
    pub fn css_class(self) -> &'static str {
        match self {
            Scrollbar::Overlay => "scrollbar-overlay",
            Scrollbar::Always => "scrollbar-always",
            Scrollbar::Hidden => "scrollbar-hidden",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_round_trips_and_falls_back_to_overlay() {
        for s in Scrollbar::ALL {
            assert_eq!(Scrollbar::parse(s.id()), s);
        }
        assert_eq!(Scrollbar::parse("nope"), Scrollbar::Overlay);
    }

    #[test]
    fn only_overlay_floats_and_only_hidden_has_no_bar() {
        assert!(Scrollbar::Overlay.overlays() && Scrollbar::Overlay.visible());
        assert!(!Scrollbar::Always.overlays() && Scrollbar::Always.visible());
        assert!(!Scrollbar::Hidden.overlays() && !Scrollbar::Hidden.visible());
    }

    #[test]
    fn classes_are_distinct() {
        let classes = Scrollbar::ALL.map(Scrollbar::css_class);
        assert!(classes.iter().enumerate().all(|(i, c)| !classes[..i].contains(c)));
    }
}
