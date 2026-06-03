//! yororen-ui i18n Showcase Demo
//!
//! Verifies Phase A deliverables end-to-end:
//! - Three external locale crates (en / zh-CN / ar) installed at
//!   runtime; switching is a one-line global set.
//! - i18n keys (`common.save`, `common.cancel`, ...) resolve through
//!   `cx.t(...)` against the active translation map.
//! - PlaceholderResolver slot: text input / select pull from a
//!   registered resolver (or fall back to the built-in English text).
//! - RTL: switching to Arabic flips the global text direction so the
//!   layout mirrors.

use std::sync::Arc;

use gpui::{App, AppContext, Application, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::i18n::{
    GlobalPlaceholderResolver, I18nContext, Locale, PlaceholderKey, PlaceholderResolver,
    TextDirection,
};
use yororen_ui::theme::{GlobalTheme, ThemeSet};

use yororen_ui_locale_ar as locale_ar;
use yororen_ui_locale_en as locale_en;
use yororen_ui_locale_zh_cn as locale_zh_cn;
use yororen_ui_theme_system as theme_system;

mod i18n_app;
mod state;

use state::I18nShowcaseState;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        yororen_ui::component::init(cx);
        theme_system::install(cx, cx.window_appearance());
        cx.set_global(GlobalPlaceholderResolver(Arc::new(I18nPlaceholderResolver)));
        locale_en::install(cx);

        let st = I18nShowcaseState::new(cx);
        cx.set_global(st);

        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(560.0), px(560.0)),
                cx,
            ))),
            ..Default::default()
        };

        let _ = cx.open_window(options, |_, cx| cx.new(i18n_app::I18nShowcaseApp::new));
    });
}

/// Placeholder resolver that maps each key to a fixed English string.
/// In a real app this would consult `cx.i18n().t("select.placeholder")`
/// or a per-locale translation map.
struct I18nPlaceholderResolver;

impl PlaceholderResolver for I18nPlaceholderResolver {
    fn resolve(&self, key: PlaceholderKey) -> Option<gpui::SharedString> {
        let s = match key {
            PlaceholderKey::Select => "Pick one…",
            PlaceholderKey::ComboBoxSearch => "Type to filter…",
            PlaceholderKey::KeybindingPressKeys => "Press a key…",
            PlaceholderKey::KeybindingWaiting => "Waiting…",
            PlaceholderKey::FilePath => "Pick a file…",
        };
        Some(s.into())
    }
}

pub fn switch_locale(cx: &mut App, tag: &str) {
    let Ok(locale) = Locale::new(tag) else { return };
    let map = match tag {
        "en" => locale_en::translation_map(),
        "zh" => locale_zh_cn::translation_map(),
        "ar" => locale_ar::translation_map(),
        _ => return,
    };
    let mut current = cx.i18n().clone();
    current.set_locale(locale.clone());
    current.load_translations(locale, map);
    cx.set_global(current);

    let direction = if tag == "ar" {
        TextDirection::Rtl
    } else {
        TextDirection::Ltr
    };
    // Inline `cx.global::<...>().current()` so the immutable borrow
    // ends before we call `cx.set_global`.
    let mut theme = cx.global::<GlobalTheme>().current().as_ref().clone();
    theme.text_direction = direction;
    cx.set_global(GlobalTheme::new_with_themes(
        gpui::WindowAppearance::Light,
        ThemeSet::new(theme),
    ));
    cx.refresh_windows();
}
