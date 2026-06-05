//! Internationalization runtime state and management.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

use gpui::{App, Global, SharedString};

use super::locale::{Locale, SupportedLocale, TextDirection};

/// Global i18n state that stores the current locale and available translations.
#[derive(Clone)]
pub struct I18n {
    /// Current active locale.
    pub current_locale: Locale,
    /// Optional locale to consult when a key is missing from
    /// `current_locale`. This is the CLDR-style "base locale"
    /// fallback: apps typically set it to `en` so a partially
    /// translated `zh-CN` catalog never shows raw dotted key paths
    /// in the UI.
    pub fallback_locale: Option<Locale>,
    /// Available locales.
    pub available_locales: Vec<SupportedLocale>,
    /// Translation strings indexed by locale.
    translations: HashMap<Locale, Arc<TranslationMap>>,
}

impl Global for I18n {}

impl I18n {
    /// Create a new i18n instance with default English locale.
    pub fn new() -> Self {
        Self::with_locale(Locale::default())
    }

    /// Create a new i18n instance with a specific locale.
    pub fn with_locale(locale: Locale) -> Self {
        Self {
            current_locale: locale,
            fallback_locale: None,
            available_locales: SupportedLocale::all().to_vec(),
            translations: HashMap::new(),
        }
    }

    /// Create a new i18n instance with a current locale and a fallback
    /// locale used when a key is missing from the current catalog.
    ///
    /// The fallback is consulted only when a key is absent from
    /// `current_locale`. If the same key is missing in both catalogs
    /// `t` returns `None` and the `App::t` shortcut surfaces the raw
    /// key with a one-shot stderr warning.
    pub fn with_locale_fallback(locale: Locale, fallback: Locale) -> Self {
        Self {
            current_locale: locale,
            fallback_locale: Some(fallback),
            available_locales: SupportedLocale::all().to_vec(),
            translations: HashMap::new(),
        }
    }

    /// Set the current locale.
    pub fn set_locale(&mut self, locale: Locale) {
        self.current_locale = locale;
    }

    /// Set or clear the fallback locale consulted when a key is
    /// missing from `current_locale`. Pass `None` to disable the
    /// fallback chain.
    pub fn set_fallback_locale(&mut self, locale: Option<Locale>) {
        self.fallback_locale = locale;
    }

    /// Get the current locale.
    pub fn locale(&self) -> &Locale {
        &self.current_locale
    }

    /// Get the fallback locale if one has been configured.
    pub fn fallback_locale_ref(&self) -> Option<&Locale> {
        self.fallback_locale.as_ref()
    }

    /// Get the text direction for the current locale.
    pub fn text_direction(&self) -> TextDirection {
        self.current_locale.text_direction()
    }

    /// Check if RTL mode is active.
    pub fn is_rtl(&self) -> bool {
        self.text_direction().is_rtl()
    }

    /// Load translations for a locale.
    pub fn load_translations(&mut self, locale: Locale, translations: TranslationMap) {
        self.translations.insert(locale, Arc::new(translations));
    }

    /// Merge translations into an existing locale map.
    ///
    /// If the locale hasn't been loaded yet, this behaves like `load_translations`.
    pub fn merge_translations(&mut self, locale: Locale, translations: TranslationMap) {
        match self.translations.get_mut(&locale) {
            Some(existing) => {
                let existing_map = Arc::make_mut(existing);
                existing_map.merge(translations);
            }
            None => {
                self.load_translations(locale, translations);
            }
        }
    }

    /// Get translations for the current locale.
    pub fn translations(&self) -> Option<&Arc<TranslationMap>> {
        self.translations.get(&self.current_locale)
    }

    /// Get a translation by key.
    ///
    /// Lookup order: `current_locale` → `fallback_locale` (if set) →
    /// `None`. The fallback is the CLDR "base locale" mechanism: a
    /// partially translated `zh-CN` catalog should still render the
    /// English string instead of leaking the raw `common.cancel`
    /// key path into the UI.
    pub fn t(&self, key: &str) -> Option<&str> {
        if let Some(map) = self.translations.get(&self.current_locale)
            && let Some(value) = map.get(key)
        {
            return Some(value);
        }
        if let Some(fallback) = &self.fallback_locale
            && fallback != &self.current_locale
            && let Some(map) = self.translations.get(fallback)
            && let Some(value) = map.get(key)
        {
            return Some(value);
        }
        None
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

/// Translation map that stores key-value pairs.
#[derive(Clone, Debug, Default)]
pub struct TranslationMap {
    /// Flat key-value map for simple translations.
    values: HashMap<String, String>,
    /// Nested translations.
    nested: HashMap<String, TranslationMap>,
}

impl TranslationMap {
    /// Create a new empty translation map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a flat key-value pair.
    pub fn insert(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    /// Insert a nested translation map.
    pub fn insert_nested(&mut self, key: &str, map: TranslationMap) {
        self.nested.insert(key.to_string(), map);
    }

    /// Get a translation by key, supporting dot notation for nested keys.
    pub fn get(&self, key: &str) -> Option<&str> {
        // First try direct key
        if let Some(value) = self.values.get(key) {
            return Some(value);
        }

        // Try dot notation for nested keys
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() < 2 {
            return None;
        }

        let mut current = self;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - try to get from values
                return current.values.get(*part).map(|s| s.as_str());
            } else {
                // Navigate to nested map
                current = current.nested.get(*part)?;
            }
        }

        None
    }

    /// Get all flat key-value pairs.
    pub fn values(&self) -> &HashMap<String, String> {
        &self.values
    }

    /// Get all nested maps.
    pub fn nested(&self) -> &HashMap<String, TranslationMap> {
        &self.nested
    }

    /// Merge another translation map into this one.
    ///
    /// - Flat keys from `other` override existing keys.
    /// - Nested maps are merged recursively.
    pub fn merge(&mut self, other: TranslationMap) {
        for (key, value) in other.values {
            self.values.insert(key, value);
        }
        for (key, nested_other) in other.nested {
            match self.nested.get_mut(&key) {
                Some(existing) => existing.merge(nested_other),
                None => {
                    self.nested.insert(key, nested_other);
                }
            }
        }
    }
}

/// Helper to access i18n from app context.
pub trait I18nContext {
    fn i18n(&self) -> &I18n;
}

impl I18nContext for App {
    fn i18n(&self) -> &I18n {
        self.global::<I18n>()
    }
}

/// Helper to translate strings within app context.
pub trait Translate {
    /// Translate a key to a string.
    fn t(&self, key: &str) -> SharedString;

    /// Translate with positional placeholders.
    ///
    /// The `args` slice supplies the values for each `{}` in the
    /// template, in order. This replaces the previous
    /// `HashMap<&str, &str>` API, which was unsafe (a key being a
    /// substring of another key would corrupt the output).
    fn t_with_args(&self, key: &str, args: &[&str]) -> SharedString;

    /// Look up a translation, returning `None` if the key is missing.
    ///
    /// Production code that needs to distinguish "no translation"
    /// from "translation equals the key" should use this and decide
    /// whether to log, surface a metric, or panic at the call
    /// site. The `t(...)` shortcut remains for the 99% path where
    /// falling back to the key is acceptable.
    fn lookup(&self, key: &str) -> Option<SharedString>;
}

impl Translate for App {
    fn t(&self, key: &str) -> SharedString {
        match self.lookup(key) {
            Some(s) => s,
            None => {
                warn_missing_key(key);
                key.to_string().into()
            }
        }
    }

    fn t_with_args(&self, key: &str, args: &[&str]) -> SharedString {
        let i18n = self.i18n();
        let base = match i18n.t(key) {
            Some(s) => s.to_string(),
            None => {
                warn_missing_key(key);
                key.to_string()
            }
        };

        replace_placeholders(&base, args).into()
    }

    fn lookup(&self, key: &str) -> Option<SharedString> {
        self.i18n().t(key).map(|s| s.to_string().into())
    }
}

/// Emit a one-shot `eprintln!` warning for a missing translation
/// key. Subsequent calls for the same key are suppressed so that
/// production log volume stays bounded.
fn warn_missing_key(key: &str) {
    static SEEN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let seen = SEEN.get_or_init(|| Mutex::new(HashSet::new()));
    let inserted = match seen.lock() {
        Ok(mut guard) => guard.insert(key.to_string()),
        // If the lock is poisoned, swallow the call rather than
        // panicking; the original panic is the real bug.
        Err(_) => false,
    };
    if inserted {
        eprintln!("[yororen-ui-core::i18n] missing translation for key: {key}");
    }
}

/// Replace placeholders in a string with values from the args map.
///
/// Placeholders use the `{}` style (consistent with `format!`). The
/// parser is non-greedy and matches each `{}` to a key from `args`
/// in declaration order. `{{` and `}}` are escape sequences for
/// literal braces.
///
/// This is safer than the previous `String::replace` approach, which
/// silently corrupted templates when a value happened to be the
/// substring of another key (e.g. `name` vs `name_id`).
///
/// Trade-offs vs a `{key}` style:
/// - Pros: cannot be confused with literal text containing braces
///   such as math notation. Substring conflicts are impossible.
/// - Cons: requires declaration order, not named lookup. The
///   `t_with_args` call site must keep argument order in sync with
///   the template, but that's already the case for `format!` and
///   is the i18n convention.
fn replace_placeholders(template: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    let mut arg_idx = 0;
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                out.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                out.push('}');
            }
            '{' => {
                // Expect a matching `}` to close the positional slot.
                // Anything between the braces is ignored (so legacy
                // named placeholders like `{name}` are tolerated as
                // positional substitutions). If `}` is missing, the
                // literal `{` is preserved so the bug surfaces.
                let mut matched_close = false;
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next == '}' {
                        matched_close = true;
                        break;
                    }
                }
                if !matched_close {
                    out.push('{');
                    continue;
                }
                if arg_idx < args.len() {
                    out.push_str(args[arg_idx]);
                    arg_idx += 1;
                } else {
                    // Missing arg: leave the `{` literal so the
                    // bug is visible at runtime.
                    out.push('{');
                }
            }
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_map_nested() {
        let mut map = TranslationMap::new();
        map.insert("hello", "Hello");

        let mut nested = TranslationMap::new();
        nested.insert("placeholder", "Select…");
        map.insert_nested("select", nested);

        assert_eq!(map.get("hello"), Some("Hello"));
        assert_eq!(map.get("select.placeholder"), Some("Select…"));
    }

    #[test]
    fn test_replace_placeholders() {
        let template = "Hello {}, you have {} items";
        let args = ["World", "5"];

        let result = replace_placeholders(template, &args);
        assert_eq!(result, "Hello World, you have 5 items");
    }

    #[test]
    fn test_replace_placeholders_escaped_braces() {
        let template = "{{literal}} {}";
        let args = ["value"];
        assert_eq!(replace_placeholders(template, &args), "{literal} value");
    }

    #[test]
    fn test_replace_placeholders_missing_arg_visible() {
        // Missing args leave `{` so the bug is visible at runtime
        // instead of silently rendering empty.
        let template = "{} and {}";
        let args = ["only-one"];
        assert_eq!(replace_placeholders(template, &args), "only-one and {");
    }

    fn map_with(pairs: &[(&str, &str)]) -> TranslationMap {
        let mut map = TranslationMap::new();
        for (k, v) in pairs {
            map.insert(k, v);
        }
        map
    }

    #[test]
    fn test_t_returns_current_locale_value() {
        let en = Locale::new("en").unwrap();
        let mut i18n = I18n::with_locale(en.clone());
        i18n.load_translations(en, map_with(&[("common.save", "Save")]));

        assert_eq!(i18n.t("common.save"), Some("Save"));
    }

    #[test]
    fn test_t_falls_back_to_fallback_locale() {
        // zh-CN is current, en is fallback. zh-CN is missing the
        // `common.cancel` key entirely; the chain should surface the
        // English string instead of `None`.
        let zh = Locale::new("zh-CN").unwrap();
        let en = Locale::new("en").unwrap();
        let mut i18n = I18n::with_locale_fallback(zh.clone(), en.clone());
        i18n.load_translations(zh, map_with(&[("common.save", "保存")]));
        i18n.load_translations(
            en,
            map_with(&[("common.save", "Save"), ("common.cancel", "Cancel")]),
        );

        assert_eq!(i18n.t("common.save"), Some("保存"));
        assert_eq!(i18n.t("common.cancel"), Some("Cancel"));
    }

    #[test]
    fn test_t_returns_none_when_key_missing_everywhere() {
        let zh = Locale::new("zh-CN").unwrap();
        let en = Locale::new("en").unwrap();
        let mut i18n = I18n::with_locale_fallback(zh.clone(), en.clone());
        i18n.load_translations(zh, map_with(&[("common.save", "保存")]));
        i18n.load_translations(en, map_with(&[("common.save", "Save")]));

        assert_eq!(i18n.t("common.cancel"), None);
    }

    #[test]
    fn test_t_skips_fallback_when_current_and_fallback_match() {
        // current == fallback: only one catalog consulted, no risk
        // of returning a value from the wrong locale.
        let en = Locale::new("en").unwrap();
        let mut i18n = I18n::with_locale_fallback(en.clone(), en.clone());
        i18n.load_translations(en, map_with(&[("common.save", "Save")]));

        assert_eq!(i18n.t("common.save"), Some("Save"));
        assert_eq!(i18n.t("common.cancel"), None);
    }

    #[test]
    fn test_t_skips_fallback_when_fallback_not_loaded() {
        // Fallback locale is configured but no translations have been
        // loaded for it. Must not panic and must return None.
        let zh = Locale::new("zh-CN").unwrap();
        let en = Locale::new("en").unwrap();
        let mut i18n = I18n::with_locale_fallback(zh.clone(), en);
        i18n.load_translations(zh, map_with(&[("common.save", "保存")]));

        assert_eq!(i18n.t("common.save"), Some("保存"));
        assert_eq!(i18n.t("common.cancel"), None);
    }

    #[test]
    fn test_set_fallback_locale_updates_chain() {
        let zh = Locale::new("zh-CN").unwrap();
        let en = Locale::new("en").unwrap();
        let fr = Locale::new("fr").unwrap();

        let mut i18n = I18n::with_locale(zh.clone());
        assert_eq!(i18n.fallback_locale_ref(), None);

        i18n.set_fallback_locale(Some(en.clone()));
        assert_eq!(i18n.fallback_locale_ref(), Some(&en));

        i18n.load_translations(zh, map_with(&[("k", "保存")]));
        i18n.load_translations(en, map_with(&[("k", "Save")]));
        i18n.load_translations(fr.clone(), map_with(&[("k", "Enregistrer")]));
        assert_eq!(i18n.t("k"), Some("保存"));

        // Swap fallback: zh-CN is missing nothing, but verifying the
        // setter is honored by the chain.
        i18n.load_translations(
            Locale::new("zh-CN").unwrap(),
            map_with(&[("k", "保存"), ("only_zh", "仅中文")]),
        );
        i18n.set_fallback_locale(Some(fr));
        assert_eq!(i18n.t("only_zh"), Some("仅中文"));
        assert_eq!(
            i18n.fallback_locale_ref().map(Locale::to_tag),
            Some("fr".to_string())
        );

        // Clear the chain entirely.
        i18n.set_fallback_locale(None);
        assert_eq!(i18n.fallback_locale_ref(), None);
        // Only `k` and `only_zh` exist in zh-CN; an unknown key now
        // resolves to None again.
        assert_eq!(i18n.t("missing_in_zh"), None);
    }
}
