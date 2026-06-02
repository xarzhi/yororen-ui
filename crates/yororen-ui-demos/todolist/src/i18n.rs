use std::path::Path;

use serde_json::Value;
use yororen_ui::i18n::{I18n, LoadError, Locale, TranslationMap, parse_translation_value};

/// Build the i18n state for the todolist demo.
///
/// Layers, in order:
/// 1. Bundled English translations (from `yororen-ui-locale-en`)
/// 2. Demo-only translations (e.g. `demo.todolist.*`)
///
/// Demo keys never override core keys because the demo's own map is
/// `merge_translations`-ed last with the demo keys scoped under `demo.*`.
///
/// Reads any pre-existing translations from the global I18n first, so it
/// is safe to call *after* a locale package's `install`.
pub fn load_demo_i18n(locale: Locale) -> Result<I18n, LoadError> {
    let mut i18n = I18n::with_locale(locale.clone());

    if locale.language() == yororen_ui::locale_en::LOCALE_TAG
        || locale_matches(&locale, yororen_ui::locale_en::LOCALE_TAG)
    {
        i18n.load_translations(locale.clone(), yororen_ui::locale_en::translation_map());
    }

    let map = load_demo_translation_map(&locale)?;
    i18n.merge_translations(locale, map);
    Ok(i18n)
}

fn locale_matches(locale: &Locale, tag: &str) -> bool {
    let (lang, region) = match tag.split_once('-') {
        Some((l, r)) => (l, r),
        None => (tag, ""),
    };
    locale.language() == lang
        && (locale.region().is_none() || region.is_empty() || locale.region() == Some(region))
}

fn load_demo_translation_map(locale: &Locale) -> Result<TranslationMap, LoadError> {
    let filename = if let Some(region) = locale.region() {
        format!("{}-{}.json", locale.language(), region)
    } else {
        format!("{}.json", locale.language())
    };

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("locales")
        .join(filename);

    let content = std::fs::read_to_string(&path).map_err(|e| {
        LoadError::IoError(format!("Failed to read demo locale file {path:?}: {e}"))
    })?;

    let value: Value =
        serde_json::from_str(&content).map_err(|e| LoadError::ParseError(e.to_string()))?;

    parse_translation_value(value)
}
