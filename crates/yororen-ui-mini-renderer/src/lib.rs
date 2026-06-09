//! `yororen-ui-mini-renderer` — minimal renderer for yororen-ui.
//!
//! Where the default renderer reads ~80 different paths from the
//! theme JSON (one per visual decision), this mini renderer reads
//! just **two** colors and bakes every other dimension into the
//! code. It exists to demonstrate the v0.3 split's payoff: a
//! third-party renderer can swap in *with* the default renderer
//! still installed — the mini overrides only the components it
//! actually cares about, and the rest fall through to the
//! defaults.
//!
//! ## Mini theme
//!
//! ```json
//! {
//!   "themeColor":  "#3b82f6",
//!   "accentColor": "#2563eb"
//! }
//! ```
//!
//! `themeColor` is the action.primary.bg fallback and
//! `accentColor` is the hover/active fallback. The renderer's
//! hardcoded geometry lives in `lib.rs` so swapping themes never
//! reshapes a control.
//!
//! ## Install
//!
//! ```ignore
//! // Order: default first (so the 38 fallbacks are in place),
//! // then mini on top (overrides the components we care about).
//! yororen_ui_default_renderer::install(cx, appearance);
//! yororen_ui_mini_renderer::install(cx);
//! ```

use std::sync::Arc;

use gpui::App;

use yororen_ui_core::renderer::RendererContext;
use yororen_ui_core::renderer::markers;
use yororen_ui_core::theme::{ActiveTheme, Theme};

use yororen_ui_default_renderer::{
    ButtonRenderer, IconButtonRenderer, LabelRenderer, ToggleButtonRenderer,
};

mod renderers;
use renderers::{
    MiniButtonRenderer, MiniIconButtonRenderer, MiniLabelRenderer, MiniToggleButtonRenderer,
};

/// Install the mini renderer. Overrides the 4 components
/// overridden by the mini crate (button, icon_button,
/// toggle_button, label); every other component continues to
/// come from the default renderer installed earlier.
///
/// Color schema (in priority order, first hit wins):
/// 1. `themeColor` — the mini's flat schema (used by
///    `mini-default.json`).
/// 2. `surface.hover` — a v0.3 surface color (always a
///    sensible *button* bg, in both light and dark modes).
///    We deliberately don't fall back to `action.primary.bg`
///    because in dark mode that path holds a *foreground*
///    tint (light gray on dark surface) and would render
///    a near-invisible button.
/// 3. `Hsla::default()` — fully transparent; the user
///    sees a debug-friendly blank instead of a hard crash.
pub fn install(cx: &mut App) {
    let theme = cx.theme();
    let base = theme
        .get_color("themeColor")
        .or_else(|| theme.get_color("surface.hover"))
        .unwrap_or_default();
    let accent = theme
        .get_color("accentColor")
        .or_else(|| theme.get_color("surface.hover"))
        .unwrap_or(base);
    let _ = accent; // currently unused; future button-renderer fields can use it

    cx.register_renderer_arc::<markers::Button, dyn ButtonRenderer>(Arc::new(MiniButtonRenderer {
        base,
    }));
    cx.register_renderer_arc::<markers::IconButton, dyn IconButtonRenderer>(Arc::new(
        MiniIconButtonRenderer { base },
    ));
    cx.register_renderer_arc::<markers::ToggleButton, dyn ToggleButtonRenderer>(Arc::new(
        MiniToggleButtonRenderer { base },
    ));
    cx.register_renderer_arc::<markers::Label, dyn LabelRenderer>(Arc::new(MiniLabelRenderer));
}

/// Load the bundled `themes/mini-default.json` and install a
/// global `Theme` from it. Convenience for the mini demo; real
/// apps keep their own JSON theme.
pub fn install_with_default_theme(cx: &mut App) {
    let json = include_str!("../themes/mini-default.json");
    let theme = Theme::from_json(json).expect("mini-default.json is valid");
    yororen_ui_core::theme::install(cx, theme);
}
