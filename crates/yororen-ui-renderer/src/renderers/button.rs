//! `ButtonRenderer` trait — the reference example.
//!
//! One trait, one component. The other 37 component renderer
//! traits follow the same shape.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::{ActionVariantKind, Theme};

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
    /// instead of reading `theme.action_variant(variant)`. When `None`,
    /// the renderer falls back to the built-in token path.
    pub custom_style: Option<Arc<dyn VariantStyle>>,
}

/// Renderer for the `Button` component. Implementations decide what the
/// button looks like in every state.
///
/// Default: [`TokenButtonRenderer`]. Theme packages override this through
/// `Theme.renderers.button` to ship a "skin".
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

/// Default implementation. Reads from `Theme.tokens().control.button.*` and
/// `Theme.action_variant(variant)`. Equivalent to the v0.3 / v0.4 button.
///
/// When `state.custom_style` is `Some`, color-related methods delegate to
/// the registered `VariantStyle` (passing the current `disabled` flag
/// through `VariantState`). Non-color properties (padding, radius, height)
/// continue to come from the theme.
pub struct TokenButtonRenderer;

impl ButtonRenderer for TokenButtonRenderer {
    fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let v = theme.action_variant(state.variant);
        if state.disabled { v.disabled_bg } else { v.bg }
    }

    fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.fg(&VariantState {
                disabled: state.disabled,
            });
        }
        let v = theme.action_variant(state.variant);
        if state.disabled { v.disabled_fg } else { v.fg }
    }

    fn padding(&self, _state: &ButtonRenderState, theme: &Theme) -> Edges<Pixels> {
        let t = &theme.tokens.control.button;
        Edges::symmetric(t.horizontal_padding, t.horizontal_padding / 2.0)
    }

    fn border_radius(&self, _state: &ButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }

    fn border(&self, _state: &ButtonRenderState, _theme: &Theme) -> Option<BorderSpec> {
        // We do not bridge `VariantStyle::border` here on purpose: the
        // default renderer does not draw a border at all (v0.3 / v0.4
        // behavior), and a custom renderer that wants to consume a
        // variant-supplied border can do so itself.
        None
    }

    fn shadow(&self, _state: &ButtonRenderState, _theme: &Theme) -> Option<ShadowSpec> {
        None
    }

    fn min_height(&self, _state: &ButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }

    fn disabled_opacity(&self, state: &ButtonRenderState, _theme: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        1.0
    }
}

/// Convenience: build a registry entry that wraps the given renderer in an Arc.
pub fn arc<T: ButtonRenderer + 'static>(r: T) -> Arc<dyn ButtonRenderer> {
    Arc::new(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    fn fixture_dark() -> Theme {
        // delegate to the private fixture used by mod.rs
        Theme {
            surface: crate::theme::SurfaceTheme {
                canvas: gpui::rgb(0x000000).into(),
                base: gpui::rgb(0x111111).into(),
                raised: gpui::rgb(0x222222).into(),
                sunken: gpui::rgb(0x000000).into(),
                hover: gpui::rgb(0x333333).into(),
            },
            content: Default::default(),
            border: Default::default(),
            action: Default::default(),
            status: Default::default(),
            shadow: Default::default(),
            text_direction: crate::i18n::TextDirection::Ltr,
            tokens: Default::default(),
            renderers: super::super::registry::RendererRegistry::token_based(),
        }
    }

    #[test]
    fn token_button_renderer_returns_dark_blue_for_primary() {
        let theme = fixture_dark();
        let r = TokenButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            disabled: false,
            ..Default::default()
        };
        // We just assert bg and fg are equal to the underlying action variant
        // (so behaviour is identical to v0.3's `compute_action_style`).
        assert_eq!(r.bg(&state, &theme), theme.action.primary.bg);
        assert_eq!(r.fg(&state, &theme), theme.action.primary.fg);
    }

    #[test]
    fn disabled_uses_disabled_palette() {
        let theme = fixture_dark();
        let r = TokenButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            disabled: true,
            ..Default::default()
        };
        assert_eq!(r.bg(&state, &theme), theme.action.primary.disabled_bg);
        assert_eq!(r.fg(&state, &theme), theme.action.primary.disabled_fg);
    }
}
