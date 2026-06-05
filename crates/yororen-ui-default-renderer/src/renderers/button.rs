//! `ButtonRenderer` trait — the reference example.
//!
//! One trait, one component. The other 37 component renderer
//! traits follow the same shape.
//!
//! `DefaultButton::default_render` is defined at the bottom of
//! this file — it's the `headless::ButtonProps`-flavoured sugar
//! that reads this renderer and decorates a `Div` with its bg /
//! fg / padding / radius / min_height.
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
//! The `default_render` sugar uses
//! `cx.renderer_arc::<headless::Button, dyn ButtonRenderer>()`
//! to fetch the registered renderer from the core registry.

use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::headless::button::Button as ButtonMarker;
use yororen_ui_core::theme::Theme;

use super::spec::{BorderSpec, Edges, ShadowSpec};
use super::variant::{VariantState, VariantStyle};

/// State passed to a `ButtonRenderer`. Fields are deliberately minimal —
/// a renderer can read more from the `Theme` if it needs to.
#[derive(Clone, Debug, Default)]
pub struct ButtonRenderState {
    pub variant: ActionVariantKind,
    pub disabled: bool,
    pub is_rtl: bool,
    /// `true` if the user supplied `.bg(...)` on the builder.
    pub has_custom_bg: bool,
    /// `true` if the user supplied `.hover_bg(...)` on the builder.
    pub has_custom_hover_bg: bool,
    /// Pre-resolved custom variant from the global `VariantRegistry`.
    /// When `Some`, the renderer should delegate color decisions
    /// (bg/fg/border/disabled_opacity) to the contained `VariantStyle`
    /// instead of reading `theme.get_color("action.<v>.<field>")`. When
    /// `None`, the renderer falls back to the built-in token path.
    pub custom_style: Option<Arc<dyn VariantStyle>>,
}

/// Logical kind of an action button. Maps to one of the three
/// entries under `theme.action.*` in the JSON theme.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ActionVariantKind {
    #[default]
    Neutral,
    Primary,
    Danger,
}

impl ActionVariantKind {
    /// Lowercase key used to look up `action.<key>.<field>` paths
    /// in the theme JSON. Stable, exposed for diagnostic
    /// messages.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Primary => "primary",
            Self::Danger => "danger",
        }
    }
}

/// Renderer for the `Button` component. Implementations decide
/// what the button looks like in every state.
///
/// Default: [`TokenButtonRenderer`]. Theme packages / renderer
/// crates override this by registering their own
/// `ButtonRenderer` impl via
/// `cx.register_renderer_arc::<ButtonMarker, dyn ButtonRenderer>(…)`.
pub trait ButtonRenderer: Any + Send + Sync {
    fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &ButtonRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &ButtonRenderState, theme: &Theme) -> Pixels;
    fn border(&self, state: &ButtonRenderState, theme: &Theme) -> Option<BorderSpec>;
    fn shadow(&self, state: &ButtonRenderState, theme: &Theme) -> Option<ShadowSpec>;
    fn min_height(&self, state: &ButtonRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &ButtonRenderState, theme: &Theme) -> f32;
}

use std::any::Any;

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
        let h = theme
            .get_number("tokens.control.button.horizontal_padding")
            .unwrap_or(12.0) as f32;
        let v = theme
            .get_number("tokens.control.button.vertical_padding")
            .unwrap_or((h as f64) / 2.0) as f32;
        Edges::symmetric(gpui::px(h), gpui::px(v))
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
        // We do not bridge `VariantStyle::border` here on purpose:
        // the default renderer does not draw a border at all
        // (v0.3 / v0.4 behavior), and a custom renderer that wants
        // to consume a variant-supplied border can do so itself.
        None
    }

    fn shadow(&self, _state: &ButtonRenderState, _theme: &Theme) -> Option<ShadowSpec> {
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
        let expected = theme.get_number("tokens.control.button.min_height").unwrap() as f32;
        // Pixels equality is f32-based; compare values.
        assert_eq!(
            r.min_height(&state, &theme),
            gpui::px(expected),
        );
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

    // Smoke-test that the renderer's bg/fg read the same path
    // values the theme JSON declares. (The renderer's
    // `bg` calls `theme.get_color("action.<variant>.<field>")`
    // directly, so this is a direct equality check.)
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

// =====================================================================
// `DefaultButton` — render a `headless::ButtonProps` with the
// registered `ButtonRenderer`. Lives in this same file because
// it is the `headless`-shaped entry point for *this* renderer.
// =====================================================================

use gpui::{div, App, Stateful, Styled};
use yororen_ui_core::headless::button::ButtonProps;
use yororen_ui_core::renderer::RendererContext;
use yororen_ui_core::theme::ActiveTheme;

/// Sugar trait. Add `use yororen_ui_renderer::renderers::button::DefaultButton;`
/// to a file to unlock `.default_render(cx)` on every `ButtonProps`.
pub trait DefaultButton: Sized {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div>;
}

impl DefaultButton for ButtonProps {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn ButtonRenderer> = cx
            .renderer_arc::<ButtonMarker, dyn ButtonRenderer>()
            .expect("ButtonRenderer registered");
        let state = ButtonRenderState::default();
        let bg = r.bg(&state, theme);
        let fg = r.fg(&state, theme);
        let padding = r.padding(&state, theme);
        let radius = r.border_radius(&state, theme);
        let min_h = r.min_height(&state, theme);
        let opacity = if self.disabled {
            r.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let div = div()
            .bg(bg)
            .text_color(fg)
            .px(padding.left)
            .py(padding.top)
            .rounded(radius)
            .min_h(min_h)
            .flex()
            .items_center()
            .justify_center()
            .opacity(opacity);
        self.apply(div)
    }
}
