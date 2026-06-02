//! Simplified Chinese (zh-CN) translation catalog for `yororen-ui`.
//!
//! # Quick start
//!
//! ```ignore
//! use yororen_ui::locale_zh_cn;
//!
//! locale_zh_cn::install(cx);
//! ```
//!
//! See [`yororen_ui_locale_en`](https://docs.rs/yororen-ui-locale-en) for
//! the equivalent English package; both expose the same `install` /
//! `translation_map` / `LOCALE_TAG` shape.

use yororen_ui_core::i18n::{I18n, Locale, TranslationMap, parse_translation_value};

/// BCP-47 tag identifying this locale.
pub const LOCALE_TAG: &str = "zh-CN";

const RAW_JSON: &str = include_str!("../translations/zh-CN.json");

/// Parse and return the bundled Simplified Chinese translation map.
pub fn translation_map() -> TranslationMap {
    let value: serde_json::Value =
        serde_json::from_str(RAW_JSON).expect("bundled zh-CN.json must be valid JSON");
    parse_translation_value(value).expect("bundled zh-CN.json must be a JSON object")
}

/// Convenience: install Simplified Chinese as the active locale on `cx`
/// and load the bundled translation map.
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
            Some("保存".to_string())
        );
        assert_eq!(
            map.get("select.placeholder").map(String::from),
            Some("请选择…".to_string())
        );
    }

    #[test]
    fn locale_tag_is_valid() {
        assert!(Locale::new(LOCALE_TAG).is_ok());
    }
}
