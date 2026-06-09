//! Mini renderer impls. Each one reads the bare minimum from
//! the theme — typically 0 to 2 fields — and bakes the rest
//! of the visual into Rust code.
//!
//! All 4 mini renderer impls (Button / IconButton / ToggleButton /
//! Label) take the v0.3 core `Theme` now that the default
//! renderer's traits have been migrated to the path-based
//! schema.

use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::theme::Theme;
use yororen_ui_default_renderer::renderers::spec::{BorderSpec, Edges, ShadowSpec};
use yororen_ui_default_renderer::renderers::{
    ButtonRenderState, ButtonRenderer, IconButtonRenderState, IconButtonRenderer, LabelRenderState,
    LabelRenderer, ToggleButtonRenderState, ToggleButtonRenderer,
};

/// The mini palette: just one `Hsla` reused everywhere.
#[derive(Copy, Clone, Debug)]
pub struct MiniPalette {
    #[allow(dead_code)]
    pub base: Hsla,
}

impl MiniPalette {
    /// Read the single `themeColor` field from the v0.3
    /// JSON-backed theme. Returns a zeroed `Hsla` if the path
    /// is missing.
    ///
    /// When the host theme uses the v0.3 dot-path schema
    /// (e.g. `surface.hover`) instead of the mini's flat
    /// 5-field schema, we fall back to `surface.hover`.
    /// Note: we deliberately do *not* fall back to
    /// `action.primary.bg` — in dark mode that field holds a
    /// foreground tint (light on dark), which would render
    /// the button near-invisible.
    pub fn from_theme(theme: &Theme) -> Self {
        let base = theme
            .get_color("themeColor")
            .or_else(|| theme.get_color("surface.hover"))
            .unwrap_or_default();
        Self { base }
    }
}

fn radius() -> Pixels {
    // Mini bakes geometry into code, so the radius never changes
    // with the theme.
    gpui::px(4.0)
}

fn min_h() -> Pixels {
    gpui::px(32.0)
}

fn pad_x() -> Pixels {
    gpui::px(12.0)
}

fn pad_y() -> Pixels {
    gpui::px(6.0)
}

// =====================================================================
// `ButtonRenderer` — only reads `themeColor`. Padding, radius,
// height, and border are all hard-coded.
// =====================================================================

pub struct MiniButtonRenderer {
    pub base: Hsla,
}

impl ButtonRenderer for MiniButtonRenderer {
    fn bg(&self, _state: &ButtonRenderState, _theme: &Theme) -> Hsla {
        if _state.disabled {
            // A subtle grey, completely independent of the theme.
            gpui::hsla(0.0, 0.0, 0.5, 0.6)
        } else {
            self.base
        }
    }
    fn hover_bg(&self, _state: &ButtonRenderState, _theme: &Theme) -> Hsla {
        // Mini doesn't differentiate hover from base — the
        // override is a "minimal" renderer.
        if _state.disabled {
            gpui::hsla(0.0, 0.0, 0.5, 0.6)
        } else {
            self.base
        }
    }
    fn active_bg(&self, _state: &ButtonRenderState, _theme: &Theme) -> Hsla {
        if _state.disabled {
            gpui::hsla(0.0, 0.0, 0.5, 0.6)
        } else {
            self.base
        }
    }
    fn fg(&self, _state: &ButtonRenderState, _theme: &Theme) -> Hsla {
        gpui::hsla(0.0, 0.0, 1.0, 1.0)
    }
    fn padding(&self, _state: &ButtonRenderState, _theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(pad_x(), pad_y())
    }
    fn border_radius(&self, _state: &ButtonRenderState, _theme: &Theme) -> Pixels {
        radius()
    }
    fn border(&self, _state: &ButtonRenderState, _theme: &Theme) -> Option<BorderSpec> {
        None
    }
    fn shadow(&self, _state: &ButtonRenderState, _theme: &Theme) -> Option<ShadowSpec> {
        None
    }
    fn min_height(&self, _state: &ButtonRenderState, _theme: &Theme) -> Pixels {
        min_h()
    }
    fn disabled_opacity(&self, _state: &ButtonRenderState, _theme: &Theme) -> f32 {
        1.0
    }
}

// =====================================================================
// `IconButtonRenderer` — reuses the MiniButtonRenderer's bg but
// with a square aspect.
// =====================================================================

pub struct MiniIconButtonRenderer {
    pub base: Hsla,
}

impl IconButtonRenderer for MiniIconButtonRenderer {
    fn bg(&self, _state: &IconButtonRenderState, _theme: &Theme) -> Hsla {
        self.base
    }
    fn hover_bg(&self, _state: &IconButtonRenderState, _theme: &Theme) -> Hsla {
        // No hover differentiation in the mini.
        self.base
    }
    fn active_bg(&self, _state: &IconButtonRenderState, _theme: &Theme) -> Hsla {
        self.base
    }
    fn size(&self, _state: &IconButtonRenderState, _theme: &Theme) -> Pixels {
        min_h()
    }
    fn border_radius(&self, _state: &IconButtonRenderState, _theme: &Theme) -> Pixels {
        radius()
    }
    fn disabled_opacity(&self, _state: &IconButtonRenderState, _theme: &Theme) -> f32 {
        1.0
    }
}

// =====================================================================
// `ToggleButtonRenderer` — selected state uses the base color;
// unselected state is "no fill".
// =====================================================================

pub struct MiniToggleButtonRenderer {
    pub base: Hsla,
}

impl ToggleButtonRenderer for MiniToggleButtonRenderer {
    fn bg(&self, state: &ToggleButtonRenderState, _theme: &Theme) -> Hsla {
        if state.selected {
            self.base
        } else {
            gpui::hsla(0.0, 0.0, 0.95, 1.0)
        }
    }
    fn hover_bg(&self, state: &ToggleButtonRenderState, _theme: &Theme) -> Hsla {
        // Mini: hover mirrors base. (Future: differentiate.)
        if state.selected {
            self.base
        } else {
            gpui::hsla(0.0, 0.0, 0.95, 1.0)
        }
    }
    fn active_bg(&self, state: &ToggleButtonRenderState, _theme: &Theme) -> Hsla {
        if state.selected {
            self.base
        } else {
            gpui::hsla(0.0, 0.0, 0.9, 1.0)
        }
    }
    fn fg(&self, state: &ToggleButtonRenderState, _theme: &Theme) -> Hsla {
        if state.selected {
            gpui::hsla(0.0, 0.0, 1.0, 1.0)
        } else {
            gpui::hsla(0.0, 0.0, 0.1, 1.0)
        }
    }
    fn min_height(&self, _state: &ToggleButtonRenderState, _theme: &Theme) -> Pixels {
        min_h()
    }
    fn border_radius(&self, _state: &ToggleButtonRenderState, _theme: &Theme) -> Pixels {
        radius()
    }
    fn disabled_opacity(&self, _state: &ToggleButtonRenderState, _theme: &Theme) -> f32 {
        1.0
    }
}

// =====================================================================
// `LabelRenderer` — completely theme-agnostic. Always the same
// text color regardless of muted/strong/mono/inherit_color.
// =====================================================================

pub struct MiniLabelRenderer;

impl LabelRenderer for MiniLabelRenderer {
    fn color(&self, _state: &LabelRenderState, _theme: &Theme) -> Hsla {
        gpui::hsla(0.0, 0.0, 0.1, 1.0)
    }
    fn strong_weight(&self, _state: &LabelRenderState, _theme: &Theme) -> gpui::FontWeight {
        gpui::FontWeight(700.0)
    }
    fn family_mono(&self, _state: &LabelRenderState, _theme: &Theme) -> gpui::SharedString {
        "ui-monospace".into()
    }
}

#[allow(dead_code)]
fn _force_imports(_: Arc<()>) {
    let _ = MiniPalette::from_theme;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mini_palette_reads_theme_color_when_present() {
        let theme = yororen_ui_core::theme::Theme::from_value(serde_json::json!({
            "themeColor": "#abcdef"
        }));
        let p = MiniPalette::from_theme(&theme);
        assert!(p.base.a > 0.5, "expected opaque base, got {:?}", p.base);
    }

    #[test]
    fn mini_palette_falls_back_to_surface_hover() {
        let theme = yororen_ui_core::theme::Theme::from_value(serde_json::json!({
            "surface": { "hover": "#654321" }
        }));
        let p = MiniPalette::from_theme(&theme);
        // Should fall back to surface.hover (#654321 → opaque
        // dark-ish color) rather than the default
        // `action.primary.bg` (which in dark mode is a
        // foreground tint, not a button bg).
        assert!(p.base.a > 0.5, "expected opaque fallback, got {:?}", p.base);
    }

    #[test]
    fn mini_palette_yields_zero_when_no_relevant_key() {
        let theme = yororen_ui_core::theme::Theme::from_value(serde_json::json!({}));
        let p = MiniPalette::from_theme(&theme);
        assert_eq!(p.base.a, 0.0);
    }
}
