//! Meta-crate for yororen-ui.
//!
//! Re-exports the three layers:
//!
//! - [`yororen_ui_core`] — headless primitives + i18n + a11y + rtl + animation + assets.
//! - [`yororen_ui_default_renderer`] — 38 component renderers + Theme (JSON-backed) + bundled themes.
//!
//! Plus the bundled locale catalogs (`en`, `zh-CN`, `ar`).
//!
//! Themes ship as JSON files in the renderer crate; no separate
//! theme package is needed. See `yororen_ui_default_renderer::themes`
//! for the bundled `system_light`, `system_dark`, etc. loaders.

pub use yororen_ui_core::renderer::{
    BuiltinVariantKey, ButtonVariant, GlobalVariantRegistry, RendererContext, RendererMarker,
    RendererRegistry, TokenVariantStyle, VariantKey, VariantRegistry, VariantState, VariantStyle,
    markers,
};
/// Re-export of the core `theme` module so user code can write
/// `use yororen_ui::theme::ActiveTheme;` (the v0.3 meta exposes
/// every public item at the crate root too, but the
/// `theme::` namespace matches the v0.2 import path).
pub use yororen_ui_core::theme;
pub use yororen_ui_core::theme::{ActiveTheme, GlobalTheme, Theme};
pub use yororen_ui_core::{a11y, animation, assets, headless, i18n, notification, rtl};
pub use yororen_ui_default_renderer as renderer;
/// `ActionVariantKind` is re-exported from the renderer
/// (the canonical home in v0.3). Also reachable via
/// `yororen_ui::renderer::ActionVariantKind`.
pub use yororen_ui_default_renderer::renderers::button::ActionVariantKind;
pub use yororen_ui_default_renderer::renderers::spec::{
    BorderSpec, Edges, IconPosition, ShadowSpec,
};

#[cfg(feature = "mini")]
pub use yororen_ui_mini_renderer as mini_renderer;

pub use yororen_ui_locale_ar as locale_ar;
pub use yororen_ui_locale_en as locale_en;
pub use yororen_ui_locale_zh_cn as locale_zh_cn;
