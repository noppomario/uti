//! Settings window translations from `data/i18n/<lang>.json`
//!
//! Nested JSON objects are flattened into dot-separated keys
//! (`general.autoStart.label`). The main window is English only.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Supported languages, in the order the settings window lists them.
pub const LANGUAGES: [&str; 2] = ["en", "ja"];

type Table = HashMap<String, String>;

fn parse(json: &str) -> Table {
    fn walk(prefix: &str, value: &serde_json::Value, out: &mut Table) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    let path = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}.{key}")
                    };
                    walk(&path, value, out);
                }
            },
            serde_json::Value::String(s) => {
                out.insert(prefix.to_string(), s.clone());
            },
            _ => {},
        }
    }
    let value: serde_json::Value = serde_json::from_str(json).expect("bundled i18n JSON is valid");
    let mut out = Table::new();
    walk("", &value, &mut out);
    out
}

fn table(lang: &str) -> &'static Table {
    static EN: OnceLock<Table> = OnceLock::new();
    static JA: OnceLock<Table> = OnceLock::new();
    match lang {
        "ja" => JA.get_or_init(|| parse(include_str!("../data/i18n/ja.json"))),
        _ => EN.get_or_init(|| parse(include_str!("../data/i18n/en.json"))),
    }
}

/// Text for `key` in `lang`, falling back to English, then to the key.
pub fn tr(lang: &str, key: &str) -> String {
    table(lang)
        .get(key)
        .or_else(|| table("en").get(key))
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_nested_keys() {
        let t = parse(r#"{"a": {"b": "x", "c": {"d": "y"}}, "e": "z"}"#);
        assert_eq!(t["a.b"], "x");
        assert_eq!(t["a.c.d"], "y");
        assert_eq!(t["e"], "z");
    }

    #[test]
    fn every_language_has_the_same_keys() {
        let mut en: Vec<_> = table("en").keys().collect();
        en.sort();
        for lang in LANGUAGES {
            let mut keys: Vec<_> = table(lang).keys().collect();
            keys.sort();
            assert_eq!(keys, en, "{lang} keys differ from en");
        }
    }

    #[test]
    fn translates_and_falls_back() {
        assert_eq!(tr("en", "sections.general"), "General");
        assert_eq!(tr("ja", "sections.general"), "一般");
        assert_eq!(tr("fr", "sections.general"), "General");
        assert_eq!(tr("ja", "no.such.key"), "no.such.key");
    }
}
