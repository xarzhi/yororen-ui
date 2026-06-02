//! Translation resource loaders for loading JSON locale files.
//!
//! `core` itself is data-agnostic: locale JSON files ship in dedicated
//! `yororen-ui-locale-*` crates. This module provides:
//!
//! - [`TranslationLoader`] — trait every loader implements
//! - [`FileLoader`] — reads `<base>/<lang>.json` (or `<base>/<lang>-<region>.json`)
//!   from disk
//! - [`FallbackLoader`] — wraps a `FileLoader`; returns a "not found" error
//!   only if the file does not exist on disk
//! - [`parse_translation_value`] — public helper that turns a parsed JSON
//!   value into a [`TranslationMap`]; used by the locale crates to
//!   `include_str!` their bundled JSON at compile time

use std::fmt;
use std::path::Path;

use super::locale::Locale;
use super::runtime::TranslationMap;

/// Error type for translation loading.
#[derive(Debug)]
pub enum LoadError {
    /// Failed to parse JSON.
    ParseError(String),
    /// Locale not found.
    LocaleNotFound(String),
    /// File IO error.
    IoError(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LoadError::LocaleNotFound(locale) => write!(f, "Locale not found: {}", locale),
            LoadError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for LoadError {}

/// Trait for loading translation resources.
pub trait TranslationLoader {
    /// Load translations for a specific locale.
    fn load(&self, locale: &Locale) -> Result<TranslationMap, LoadError>;

    /// Check if a locale is available.
    fn is_available(&self, locale: &Locale) -> bool;
}

/// Build the canonical JSON filename for a locale.
///
/// Returns `"{lang}.json"` for language-only tags (e.g. `en`, `zh`) and
/// `"{lang}-{region}.json"` for region tags (e.g. `en-US`, `zh-CN`).
pub fn filename_for_locale(locale: &Locale) -> String {
    let lang = locale.language();
    if let Some(region) = locale.region() {
        format!("{}-{}.json", lang, region)
    } else {
        format!("{}.json", lang)
    }
}

/// Parse a JSON value into a [`TranslationMap`].
///
/// JSON objects become nested `TranslationMap`s; JSON strings become flat
/// key-value entries. Other JSON types (numbers, arrays, booleans, null)
/// are silently dropped, since they have no translation meaning.
pub fn parse_translation_value(value: serde_json::Value) -> Result<TranslationMap, LoadError> {
    let mut map = TranslationMap::new();

    let obj = match value {
        serde_json::Value::Object(obj) => obj,
        _ => {
            return Err(LoadError::ParseError("Expected JSON object".to_string()));
        }
    };

    for (key, val) in obj {
        match val {
            serde_json::Value::String(s) => {
                map.insert(&key, &s);
            }
            serde_json::Value::Object(nested) => {
                let nested_map = parse_translation_value(serde_json::Value::Object(nested))?;
                map.insert_nested(&key, nested_map);
            }
            _ => {
                // Skip non-string, non-object values
            }
        }
    }

    Ok(map)
}

/// Loader that loads translations from external files.
pub struct FileLoader {
    base_path: String,
}

impl FileLoader {
    /// Create a new file loader with the given base path.
    pub fn new(base_path: impl Into<String>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
}

impl TranslationLoader for FileLoader {
    fn load(&self, locale: &Locale) -> Result<TranslationMap, LoadError> {
        let filename = filename_for_locale(locale);
        let path = Path::new(&self.base_path).join(&filename);

        let content = std::fs::read_to_string(&path)
            .map_err(|e| LoadError::IoError(format!("Failed to read file: {}", e)))?;

        let value: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| LoadError::ParseError(e.to_string()))?;

        parse_translation_value(value)
    }

    fn is_available(&self, locale: &Locale) -> bool {
        let filename = filename_for_locale(locale);
        let path = Path::new(&self.base_path).join(&filename);
        path.exists()
    }
}

/// Create a loader that tries the filesystem first; returns
/// `LocaleNotFound` only when no file is available.
///
/// In v0.4+ there is no embedded fallback; locale data ships in
/// dedicated `yororen-ui-locale-*` crates that bundle their own JSON
/// via `include_str!`. This loader remains for callers that prefer to
/// drop raw JSON files on disk (e.g. user-supplied translations).
pub struct FallbackLoader {
    file: Option<FileLoader>,
}

impl FallbackLoader {
    /// Create a new fallback loader. If `file_base_path` is `None`, every
    /// `load` call will return `LocaleNotFound`.
    pub fn new(file_base_path: Option<impl Into<String>>) -> Self {
        Self {
            file: file_base_path.map(FileLoader::new),
        }
    }
}

impl TranslationLoader for FallbackLoader {
    fn load(&self, locale: &Locale) -> Result<TranslationMap, LoadError> {
        if let Some(ref file) = self.file
            && file.is_available(locale)
        {
            return file.load(locale);
        }

        Err(LoadError::LocaleNotFound(locale.to_string()))
    }

    fn is_available(&self, locale: &Locale) -> bool {
        self.file.as_ref().is_some_and(|f| f.is_available(locale))
    }
}
