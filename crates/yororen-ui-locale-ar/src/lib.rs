//! Arabic (ar) translation catalog for `yororen-ui`.
//!
//! # Quick start
//!
//! ```ignore
//! use yororen_ui::locale_ar;
//!
//! locale_ar::install(cx);
//! ```
//!
//! See [`yororen_ui_locale_en`](https://docs.rs/yororen-ui-locale-en) for
//! the equivalent English package; both expose the same `install` /
//! `translation_map` / `LOCALE_TAG` shape.
//!
//! Note: `install` only sets the *i18n* global. To also flip the global
//! theme's text direction to RTL, do it in your own app bootstrap
//! (typically after the theme package's `install`):
//!
//! ```ignore
//! theme_system::install(cx, cx.window_appearance());
//! locale_ar::install(cx);
//! theme_system::with_text_direction(cx, TextDirection::Rtl);
//! ```

use yororen_ui_core::i18n::{I18n, Locale, TextDirection, TranslationMap, parse_translation_value};

/// BCP-47 tag identifying this locale.
pub const LOCALE_TAG: &str = "ar";

/// Text direction this locale uses.
pub const TEXT_DIRECTION: TextDirection = TextDirection::Rtl;

const RAW_JSON: &str = include_str!("../translations/ar.json");

/// Parse and return the bundled Arabic translation map.
pub fn translation_map() -> TranslationMap {
    let value: serde_json::Value =
        serde_json::from_str(RAW_JSON).expect("bundled ar.json must be valid JSON");
    parse_translation_value(value).expect("bundled ar.json must be a JSON object")
}

/// Convenience: install Arabic as the active locale on `cx` and load
/// the bundled translation map.
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
        assert_eq!(
            map.get("common.save").map(String::from),
            Some("حفظ".to_string())
        );
        assert_eq!(
            map.get("select.placeholder").map(String::from),
            Some("اختر…".to_string())
        );
    }

    #[test]
    fn locale_tag_is_rtl() {
        let locale = Locale::new(LOCALE_TAG).expect("LOCALE_TAG must be a valid locale");
        assert!(locale.text_direction().is_rtl());
    }
}
