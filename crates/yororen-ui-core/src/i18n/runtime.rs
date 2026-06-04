//! Internationalization runtime state and management.

use std::collections::HashMap;
use std::sync::Arc;

use gpui::{App, Global, SharedString};

use super::locale::{Locale, SupportedLocale, TextDirection};

/// Global i18n state that stores the current locale and available translations.
#[derive(Clone)]
pub struct I18n {
    /// Current active locale.
    pub current_locale: Locale,
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
            available_locales: SupportedLocale::all().to_vec(),
            translations: HashMap::new(),
        }
    }

    /// Set the current locale.
    pub fn set_locale(&mut self, locale: Locale) {
        self.current_locale = locale;
    }

    /// Get the current locale.
    pub fn locale(&self) -> &Locale {
        &self.current_locale
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
    pub fn t(&self, key: &str) -> Option<&str> {
        self.translations()?.get(key)
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
        self.lookup(key).unwrap_or_else(|| key.to_string().into())
    }

    fn t_with_args(&self, key: &str, args: &[&str]) -> SharedString {
        let i18n = self.i18n();
        let base = match i18n.t(key) {
            Some(s) => s.to_string(),
            None => key.to_string(),
        };

        replace_placeholders(&base, args).into()
    }

    fn lookup(&self, key: &str) -> Option<SharedString> {
        self.i18n().t(key).map(|s| s.to_string().into())
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
                if arg_idx < args.len() {
                    out.push_str(args[arg_idx]);
                    arg_idx += 1;
                } else {
                    // Missing arg: leave the `{}` literal so the
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
        let template = "Hello {name}, you have {count} items";
        let mut args = HashMap::new();
        args.insert("name", "World");
        args.insert("count", "5");

        let result = replace_placeholders(template, &args);
        assert_eq!(result, "Hello World, you have 5 items");
    }
}
