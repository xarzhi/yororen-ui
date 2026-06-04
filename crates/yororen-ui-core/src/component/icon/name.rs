use std::sync::OnceLock;

use gpui::SharedString;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ArrowDirection {
    fn slug(self) -> &'static str {
        match self {
            Self::Up => "arrow-up",
            Self::Down => "arrow-down",
            Self::Left => "arrow-left",
            Self::Right => "arrow-right",
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum IconName {
    Search,

    Arrow(ArrowDirection),
    Check,
    Warning,
    Info,
    Close,
    Maximize(bool),
    Minimize,
    User,
    Pencil,
    Trash,
    File,
    Folder,
}

impl IconName {
    /// Returns the filename slug for this icon. Variants that carry
    /// data (Arrow direction, Maximize on/off) MUST include every
    /// data field so distinct variants don't collide in the cache.
    fn slug(self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::Arrow(direction) => direction.slug(),
            Self::Check => "check",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Close => "close",
            Self::Maximize(true) => "maximize-on",
            Self::Maximize(false) => "maximize-off",
            Self::Minimize => "minimize",
            Self::User => "user",
            Self::Pencil => "pencil",
            Self::Trash => "trash",
            Self::File => "file",
            Self::Folder => "folder",
        }
    }
}

/// Cache the `SharedString` per `IconName` so we don't allocate a
/// `String` on every `Icon::from(name)` call. The cache key is the
/// slug itself, which encodes every discriminant of `IconName`, so
/// e.g. `Maximize(true)` and `Maximize(false)` cannot collide.
fn cached(name: IconName) -> SharedString {
    static CACHE: OnceLock<std::sync::Mutex<std::collections::HashMap<&'static str, SharedString>>> =
        OnceLock::new();
    let slug = name.slug();
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(Default::default()));
    if let Some(s) = cache.lock().expect("icon cache poisoned").get(slug).cloned() {
        return s;
    }
    let s: SharedString = format!("icons/{slug}.svg").into();
    cache.lock().expect("icon cache poisoned").insert(slug, s.clone());
    s
}

impl From<IconName> for SharedString {
    fn from(value: IconName) -> Self {
        cached(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maximize_variants_do_not_collide() {
        let on: SharedString = IconName::Maximize(true).into();
        let off: SharedString = IconName::Maximize(false).into();
        assert_eq!(on.as_ref(), "icons/maximize-on.svg");
        assert_eq!(off.as_ref(), "icons/maximize-off.svg");
        assert_ne!(on, off);

        // Re-fetch from cache to ensure no overwrite happened.
        let on_again: SharedString = IconName::Maximize(true).into();
        assert_eq!(on_again.as_ref(), "icons/maximize-on.svg");
    }

    #[test]
    fn arrow_variants_do_not_collide() {
        let up: SharedString = IconName::Arrow(ArrowDirection::Up).into();
        let down: SharedString = IconName::Arrow(ArrowDirection::Down).into();
        let left: SharedString = IconName::Arrow(ArrowDirection::Left).into();
        let right: SharedString = IconName::Arrow(ArrowDirection::Right).into();
        assert_eq!(up.as_ref(), "icons/arrow-up.svg");
        assert_eq!(down.as_ref(), "icons/arrow-down.svg");
        assert_eq!(left.as_ref(), "icons/arrow-left.svg");
        assert_eq!(right.as_ref(), "icons/arrow-right.svg");
    }
}
