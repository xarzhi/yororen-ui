//! Global state for the i18n showcase demo.

use gpui::{App, AppContext, Entity, Global};

/// Currently selected locale tag (one of "en" / "zh" / "ar"). Stored
/// as an `Entity<String>` so the GPUI diffing machinery tracks the
/// `cx.notify()` chain automatically.
pub struct I18nShowcaseState {
    pub locale: Entity<String>,
}

impl Global for I18nShowcaseState {}

impl I18nShowcaseState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            locale: cx.new(|_| "en".to_string()),
        }
    }

    pub fn current(&self, cx: &App) -> String {
        self.locale.read(cx).clone()
    }
}
