//! `ButtonRenderer` trait + `TokenButtonRenderer` impl.
//!
//! The trait + `ButtonRenderState` live in `yororen-ui-core` so
//! headless `ButtonProps::render` can call them. This module
//! re-exports them and provides the `TokenButtonRenderer` impl.
//!
//! ## Theme access
//!
//! `Theme` here is the v0.3 JSON-backed theme from
//! `yororen_ui_core::theme` — no fixed schema, just
//! dot-separated paths. `TokenButtonRenderer` reads:
//!
//! - `action.{neutral,primary,danger}.{bg,fg,disabled_bg,disabled_fg}` for colors
//! - `tokens.control.button.{min_height,horizontal_padding,radius}` for geometry
//!
//! The headless `ButtonProps::render` sugar uses
//! `cx.renderer_arc::<headless::Button, dyn ButtonRenderer>()`
//! to fetch the registered renderer from the core registry.

use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::headless::button::Button as ButtonMarker;
use yororen_ui_core::renderer::spec::{BorderSpec, Edges, ShadowSpec};
use yororen_ui_core::renderer::variant::VariantState;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::button::{ButtonRenderState, ButtonRenderer};
pub use yororen_ui_core::renderer::variant::ActionVariantKind;

/// Default implementation. Reads color from
/// `action.<variant>.<bg|fg|disabled_bg|disabled_fg>` and
/// geometry from `tokens.control.button.*`. Equivalent to the
/// v0.3 / v0.4 button.
///
/// When `state.custom_style` is `Some`, color-related methods
/// delegate to the registered `VariantStyle` (passing the
/// current `disabled` flag through `VariantState`).
/// Non-color properties (padding, radius, height) continue to
/// come from the theme.
pub struct TokenButtonRenderer;

impl ButtonRenderer for TokenButtonRenderer {
    fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled { "disabled_bg" } else { "bg" };
        let key = format!("action.{}.{}", state.variant.as_str(), field);
        theme.get_color(&key).unwrap_or_default()
    }

    fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.fg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled { "disabled_fg" } else { "fg" };
        let key = format!("action.{}.{}", state.variant.as_str(), field);
        theme.get_color(&key).unwrap_or_default()
    }

    fn padding(&self, _state: &ButtonRenderState, theme: &Theme) -> Edges<Pixels> {
        use gpui::px;
        let h = theme
            .get_number("tokens.control.button.horizontal_padding")
            .unwrap_or(12.0) as f32;
        let v = theme
            .get_number("tokens.control.button.vertical_padding")
            .unwrap_or((h as f64) / 2.0) as f32;
        Edges::symmetric(px(h), px(v))
    }

    fn border_radius(&self, _state: &ButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.radius")
                .or_else(|| theme.get_number("tokens.radii.md"))
                .unwrap_or(6.0) as f32,
        )
    }

    fn border(&self, _state: &ButtonRenderState, _theme: &Theme) -> Option<BorderSpec> {
        // The default renderer does not draw a border at all
        // (v0.3 / v0.4 behaviour). The brutalism renderer
        // returns a 3px black border; the headless's
        // `ButtonProps::render` dispatches based on whatever
        // is registered.
        None
    }

    fn shadow(&self, _state: &ButtonRenderState, _theme: &Theme) -> Option<ShadowSpec> {
        // The default renderer does not draw a shadow. The
        // brutalism renderer returns a 4px Y-offset hard
        // shadow; the headless's `render` reads it.
        None
    }

    fn min_height(&self, _state: &ButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.min_height")
                .unwrap_or(36.0) as f32,
        )
    }

    fn disabled_opacity(&self, state: &ButtonRenderState, _theme: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        1.0
    }

    fn hover_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled {
            "disabled_bg"
        } else {
            "hover_bg"
        };
        let key = format!("action.{}.{}", state.variant.as_str(), field);
        theme.get_color(&key).unwrap_or_default()
    }

    fn active_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled {
            "disabled_bg"
        } else {
            "active_bg"
        };
        let key = format!("action.{}.{}", state.variant.as_str(), field);
        theme.get_color(&key).unwrap_or_default()
    }
}

/// Convenience: build a registry entry that wraps the given
/// renderer in an Arc.
pub fn arc<T: ButtonRenderer + 'static>(r: T) -> Arc<dyn ButtonRenderer> {
    Arc::new(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yororen_ui_core::theme::Theme;

    fn fixture() -> Theme {
        let json = include_str!("../../themes/system-dark.json");
        Theme::from_json(json).expect("system-dark.json is valid")
    }

    #[test]
    fn token_button_renderer_returns_primary_palette() {
        let theme = fixture();
        let r = TokenButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            disabled: false,
            ..Default::default()
        };
        // bg should equal theme.action.primary.bg from JSON.
        assert_eq!(
            r.bg(&state, &theme),
            theme.get_color("action.primary.bg").unwrap()
        );
        assert_eq!(
            r.fg(&state, &theme),
            theme.get_color("action.primary.fg").unwrap()
        );
    }

    #[test]
    fn disabled_uses_disabled_palette() {
        let theme = fixture();
        let r = TokenButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            disabled: true,
            ..Default::default()
        };
        assert_eq!(
            r.bg(&state, &theme),
            theme.get_color("action.primary.disabled_bg").unwrap()
        );
        assert_eq!(
            r.fg(&state, &theme),
            theme.get_color("action.primary.disabled_fg").unwrap()
        );
    }

    #[test]
    fn neutral_variant_picks_neutral_palette() {
        let theme = fixture();
        let r = TokenButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Neutral,
            ..Default::default()
        };
        assert_eq!(
            r.bg(&state, &theme),
            theme.get_color("action.neutral.bg").unwrap()
        );
    }

    #[test]
    fn danger_variant_picks_danger_palette() {
        let theme = fixture();
        let r = TokenButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Danger,
            ..Default::default()
        };
        assert_eq!(
            r.bg(&state, &theme),
            theme.get_color("action.danger.bg").unwrap()
        );
    }

    #[test]
    fn min_height_uses_control_button_token() {
        let theme = fixture();
        let r = TokenButtonRenderer;
        let state = ButtonRenderState::default();
        let expected = theme
            .get_number("tokens.control.button.min_height")
            .unwrap() as f32;
        // Pixels equality is f32-based; compare values.
        assert_eq!(r.min_height(&state, &theme), gpui::px(expected));
    }

    #[test]
    fn hover_and_active_bg_read_action_hover_and_active_paths() {
        // Regression: headless's `render` chains
        // `.hover(|s| s.bg(r.hover_bg(&state, theme)))` and
        // `.active(|s| s.bg(r.active_bg(&state, theme)))`. If
        // either is missing, the closures compile but apply
        // `Hsla::default()`.
        let theme = fixture();
        let r = TokenButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            ..Default::default()
        };
        assert_eq!(
            r.hover_bg(&state, &theme),
            theme.get_color("action.primary.hover_bg").unwrap(),
        );
        assert_eq!(
            r.active_bg(&state, &theme),
            theme.get_color("action.primary.active_bg").unwrap(),
        );
        // And the three colours really are distinct.
        assert_ne!(r.bg(&state, &theme), r.hover_bg(&state, &theme));
        assert_ne!(r.hover_bg(&state, &theme), r.active_bg(&state, &theme));
    }

    #[test]
    fn missing_path_yields_zero_color_doesnt_panic() {
        // Theme with only one path — everything else returns None.
        let theme = Theme::from_value(serde_json::json!({}));
        let r = TokenButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            ..Default::default()
        };
        // Should not panic. Returns Hsla::default() (zeros).
        let _ = r.bg(&state, &theme);
        let _ = r.fg(&state, &theme);
        let _ = r.padding(&state, &theme);
        let _ = r.border_radius(&state, &theme);
        let _ = r.min_height(&state, &theme);
    }

    #[test]
    fn action_color_helper_reads_correct_path() {
        let theme = fixture();
        let r = TokenButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            ..Default::default()
        };
        assert_eq!(
            r.bg(&state, &theme),
            theme.get_color("action.primary.bg").unwrap(),
        );
        assert_eq!(
            r.fg(&state, &theme),
            theme.get_color("action.primary.fg").unwrap(),
        );
    }
}
