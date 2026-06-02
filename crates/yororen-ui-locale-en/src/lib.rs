//! English (en) translation catalog for `yororen-ui`.
//!
//! # Quick start
//!
//! ```ignore
//! use yororen_ui::locale_en;
//!
//! locale_en::install(cx);
//! ```
//!
//! This installs a fresh `I18n` global on `cx` with the current locale
//! set to `en` and the bundled English translation map loaded.
//!
//! For fine-grained control (loading multiple locales, layering your own
//! translations on top) use [`translation_map`] and `I18n::merge_translations`
//! directly.

use yororen_ui_core::i18n::{I18n, Locale, TranslationMap, parse_translation_value};

/// BCP-47 tag identifying this locale.
pub const LOCALE_TAG: &str = "en";

const RAW_JSON: &str = include_str!("../translations/en.json");

/// Parse and return the bundled English translation map.
pub fn translation_map() -> TranslationMap {
    let value: serde_json::Value =
        serde_json::from_str(RAW_JSON).expect("bundled en.json must be valid JSON");
    parse_translation_value(value).expect("bundled en.json must be a JSON object")
}

/// Convenience: install English as the active locale on `cx` and load the
/// bundled translation map.
pub fn install(cx: &mut gpui::App) {
    let locale = Locale::new(LOCALE_TAG).expect("LOCALE_TAG must be a valid locale");
    let mut i18n = I18n::with_locale(locale.clone());
    i18n.load_translations(locale, translation_map());
    cx.set_global(i18n);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bundled_json() {
        let map = translation_map();
        assert_eq!(map.get("common.save").map(String::from), Some("Save".to_string()));
        assert_eq!(
            map.get("select.placeholder").map(String::from),
            Some("Select…".to_string())
        );
    }

    #[test]
    fn locale_tag_is_valid() {
        assert!(Locale::new(LOCALE_TAG).is_ok());
    }
}
