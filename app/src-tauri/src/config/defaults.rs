//! Default configuration values
//!
//! Single source of truth for all default configuration values.
//! TypeScript frontend uses Rust backend as authoritative source.

/// Default color theme
pub const DEFAULT_COLOR: &str = "dark";

/// Default size theme
pub const DEFAULT_SIZE: &str = "normal";

/// Default UI language
pub const DEFAULT_LANGUAGE: &str = "en";

/// Default clipboard history limit
pub const DEFAULT_CLIPBOARD_LIMIT: usize = 50;

// Compile-time validation of default values
const _: () = assert!(DEFAULT_CLIPBOARD_LIMIT > 0);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_are_valid() {
        // Color must be one of the valid options
        assert!(matches!(DEFAULT_COLOR, "midnight" | "dark" | "light"));

        // Size must be one of the valid options
        assert!(matches!(DEFAULT_SIZE, "minimal" | "normal" | "wide"));

        // Language must be one of the valid options
        assert!(matches!(DEFAULT_LANGUAGE, "en" | "ja"));

        // Clipboard limit is validated at compile-time via const assertion above
    }
}
