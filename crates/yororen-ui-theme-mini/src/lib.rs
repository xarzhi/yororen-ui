//! A minimal theme package: 3 custom renderers layered on system palette.
//!
//! This crate exists to prove the Renderer trait system actually works
//! end-to-end. It overrides 3 renderers (button, card, modal) on top of
//! `RendererRegistry::token_based()` and reuses the rest of the
//! `theme-system` light/dark palette as-is. The visual result is
//! intentionally bold (indigo surface, cyan accent) so that a
//! before/after diff is unmistakable.
//!
//! # Usage
//!
//! ```rust,ignore
//! use yororen_ui_theme_mini as theme_mini;
//!
//! theme_mini::install(cx, cx.window_appearance());
//! ```
//!
//! Or, to mix-and-match with `theme-system`:
//!
//! ```rust,ignore
//! use yororen_ui::theme_system;
//! use yororen_ui_theme_mini as theme_mini;
//!
//! // Start with the default token-based registry, then swap 3 entries.
//! let mut registry = yororen_ui_renderer::renderers::RendererRegistry::token_based();
//! registry = registry
//!     .with_button(theme_mini::MiniButtonRenderer::arc())
//!     .with_card(theme_mini::MiniCardRenderer::arc())
//!     .with_modal(theme_mini::MiniModalRenderer::arc());
//! // Plug registry into your own Theme and GlobalTheme install.
//! ```
//!
//! # What this crate is *not*
//!
//! It is **not** a complete theme. It does not ship its own Theme struct
//! or asset palette. It only registers 3 custom renderers. Apps that
//! want a full "skin" (custom fonts, custom spacing, custom shape
//! language) should look at how `yororen-ui-theme-system` builds a
//! `Theme` and follow the same pattern.

use std::sync::Arc;

use gpui::{App, Hsla, Pixels, WindowAppearance};
use yororen_ui_renderer::renderers::{
    ButtonRenderState, ButtonRenderer, CardRenderState, CardRenderer, ModalRenderState,
    ModalRenderer, RendererRegistry,
};
use yororen_ui_renderer::theme::{GlobalTheme, Theme};

use yororen_ui_theme_system as theme_system;

/// Bold indigo / cyan accent palette used by the mini theme's three
/// renderers. These colors are intentionally not in the v0.4 token
/// system — that's the whole point: a theme package can pick colors
/// the v0.4 designer never imagined.
pub mod palette {
    use gpui::{Hsla, hsla, rgb};

    /// Bright cyan for primary actions.
    pub fn cyan() -> Hsla {
        rgb(0x22D3EE).into()
    }
    /// Slightly darker cyan on hover.
    pub fn cyan_hover() -> Hsla {
        rgb(0x06B6D4).into()
    }
    /// Soft white-ish for primary fg.
    pub fn cyan_fg() -> Hsla {
        hsla(0.0, 0.0, 0.99, 1.0)
    }
    /// Indigo surface for cards / panels.
    pub fn indigo() -> Hsla {
        rgb(0x312E81).into()
    }
    /// Deeper indigo for the modal panel so it pops over the scrim.
    pub fn indigo_deep() -> Hsla {
        rgb(0x1E1B4B).into()
    }
    /// Fuchsia accent ring on modals (think 80s / 90s UI).
    pub fn fuchsia() -> Hsla {
        rgb(0xE879F9).into()
    }
}

/// Button renderer that turns neutral / primary / danger into
/// cyan / cyan-hover / fuchsia. Note: it ignores `theme.tokens` for
/// padding and border-radius — uses its own 12-px radius and 6-px gap,
/// which is bigger than the v0.4 system default of 6 px.
pub struct MiniButtonRenderer;

impl MiniButtonRenderer {
    pub fn arc() -> Arc<dyn ButtonRenderer> {
        Arc::new(Self)
    }
}

impl ButtonRenderer for MiniButtonRenderer {
    fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        use yororen_ui_renderer::theme::ActionVariantKind;
        if state.disabled {
            theme.action_variant(state.variant).disabled_bg
        } else {
            match state.variant {
                ActionVariantKind::Primary => palette::cyan(),
                ActionVariantKind::Neutral => theme.action.neutral.bg,
                ActionVariantKind::Danger => palette::fuchsia(),
            }
        }
    }

    fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        use yororen_ui_renderer::theme::ActionVariantKind;
        if state.disabled {
            theme.action_variant(state.variant).disabled_fg
        } else {
            match state.variant {
                ActionVariantKind::Primary => palette::cyan_fg(),
                ActionVariantKind::Neutral => theme.action.neutral.fg,
                ActionVariantKind::Danger => palette::cyan_fg(),
            }
        }
    }

    fn padding(
        &self,
        _state: &ButtonRenderState,
        _theme: &Theme,
    ) -> yororen_ui_renderer::renderers::Edges<Pixels> {
        yororen_ui_renderer::renderers::Edges::symmetric(gpui::px(20.), gpui::px(12.))
    }

    fn border_radius(&self, _state: &ButtonRenderState, _theme: &Theme) -> Pixels {
        gpui::px(12.)
    }

    fn border(
        &self,
        _state: &ButtonRenderState,
        _theme: &Theme,
    ) -> Option<yororen_ui_renderer::renderers::BorderSpec> {
        None
    }

    fn shadow(
        &self,
        _state: &ButtonRenderState,
        _theme: &Theme,
    ) -> Option<yororen_ui_renderer::renderers::ShadowSpec> {
        None
    }

    fn min_height(&self, _state: &ButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }

    fn disabled_opacity(&self, _state: &ButtonRenderState, _theme: &Theme) -> f32 {
        1.0
    }
}

/// Card renderer that swaps the v0.4 surface.base for indigo.
pub struct MiniCardRenderer;

impl MiniCardRenderer {
    pub fn arc() -> Arc<dyn CardRenderer> {
        Arc::new(Self)
    }
}

impl CardRenderer for MiniCardRenderer {
    fn bg(&self, _state: &CardRenderState, _theme: &Theme) -> Hsla {
        palette::indigo()
    }
    fn border(&self, _state: &CardRenderState, _theme: &Theme) -> Hsla {
        palette::fuchsia()
    }
    fn padding(
        &self,
        _state: &CardRenderState,
        _theme: &Theme,
    ) -> yororen_ui_renderer::renderers::Edges<Pixels> {
        yororen_ui_renderer::renderers::Edges::all(gpui::px(20.))
    }
    fn border_radius(&self, _state: &CardRenderState, _theme: &Theme) -> Pixels {
        gpui::px(16.)
    }
    fn shadow_alpha(&self, _state: &CardRenderState, _theme: &Theme) -> f32 {
        0.4
    }
}

/// Modal renderer that gives the scrim a fuchsia tint and the panel a
/// deep-indigo background.
pub struct MiniModalRenderer;

impl MiniModalRenderer {
    pub fn arc() -> Arc<dyn ModalRenderer> {
        Arc::new(Self)
    }
}

impl ModalRenderer for MiniModalRenderer {
    fn scrim(&self, _state: &ModalRenderState, _theme: &Theme) -> Hsla {
        let mut s = palette::indigo_deep();
        s.a = 0.6;
        s
    }
    fn panel_bg(&self, _state: &ModalRenderState, _theme: &Theme) -> Hsla {
        palette::indigo_deep()
    }
    fn panel_border(&self, _state: &ModalRenderState, _theme: &Theme) -> Hsla {
        palette::fuchsia()
    }
    fn panel_padding(
        &self,
        _state: &ModalRenderState,
        _theme: &Theme,
    ) -> yororen_ui_renderer::renderers::Edges<Pixels> {
        yororen_ui_renderer::renderers::Edges::all(gpui::px(28.))
    }
    fn panel_border_radius(&self, _state: &ModalRenderState, _theme: &Theme) -> Pixels {
        gpui::px(20.)
    }
    fn panel_shadow_alpha(&self, _state: &ModalRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

/// Build a `RendererRegistry` that swaps button / card / modal for the
/// mini variants. Other 35 components keep their token_based() defaults.
pub fn mini_registry() -> RendererRegistry {
    RendererRegistry::token_based()
        .with_button(MiniButtonRenderer::arc())
        .with_card(MiniCardRenderer::arc())
        .with_modal(MiniModalRenderer::arc())
}

/// Convenience: install a full `GlobalTheme` with the default
/// `theme-system` light/dark palette but using the mini registry.
pub fn install(cx: &mut App, appearance: WindowAppearance) {
    let mut theme = match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => theme_system::light(),
        _ => theme_system::dark(),
    };
    theme.renderers = mini_registry();
    cx.set_global(GlobalTheme::new(theme));
}

// Re-export token fallback renderers so callers that want to extend
// the mini registry don't have to import the core crate directly.
#[allow(unused_imports)]
pub use yororen_ui_renderer::renderers::{TokenButtonRenderer, TokenCardRenderer, TokenModalRenderer};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mini_registry_swaps_three_renderers() {
        let reg = mini_registry();
        // The mini registry overrides button / card / modal, so its
        // output for those entries must differ from the v0.4 default.
        let theme = theme_system::light();
        let state = ButtonRenderState {
            variant: yororen_ui_renderer::theme::ActionVariantKind::Primary,
            ..Default::default()
        };
        let mini_bg = reg
            .get_button()
            .expect("ButtonRenderer registered")
            .bg(&state, &theme);
        let default_bg = yororen_ui_renderer::renderers::TokenButtonRenderer.bg(&state, &theme);
        assert_ne!(mini_bg, default_bg);

        let card_state = CardRenderState::default();
        let mini_card_bg = reg
            .get_card()
            .expect("CardRenderer registered")
            .bg(&card_state, &theme);
        let default_card_bg = yororen_ui_renderer::renderers::TokenCardRenderer.bg(&card_state, &theme);
        assert_ne!(mini_card_bg, default_card_bg);

        let modal_state = ModalRenderState::default();
        let mini_modal_bg = reg
            .get_modal()
            .expect("ModalRenderer registered")
            .panel_bg(&modal_state, &theme);
        let default_modal_bg =
            yororen_ui_renderer::renderers::TokenModalRenderer.panel_bg(&modal_state, &theme);
        assert_ne!(mini_modal_bg, default_modal_bg);
    }

    #[test]
    fn mini_button_picks_cyan_for_primary() {
        let theme = theme_system::light();
        let r = MiniButtonRenderer;
        let state = ButtonRenderState {
            variant: yororen_ui_renderer::theme::ActionVariantKind::Primary,
            ..Default::default()
        };
        assert_eq!(r.bg(&state, &theme), palette::cyan());
        assert_eq!(r.fg(&state, &theme), palette::cyan_fg());
    }

    #[test]
    fn mini_button_picks_fuchsia_for_danger() {
        let theme = theme_system::light();
        let r = MiniButtonRenderer;
        let state = ButtonRenderState {
            variant: yororen_ui_renderer::theme::ActionVariantKind::Danger,
            ..Default::default()
        };
        assert_eq!(r.bg(&state, &theme), palette::fuchsia());
    }

    #[test]
    fn mini_button_uses_12_px_radius_not_6() {
        let theme = theme_system::light();
        let r = MiniButtonRenderer;
        let state = ButtonRenderState::default();
        let radius: f32 = r.border_radius(&state, &theme).into();
        // v0.4 token default is 6 px; the mini theme picks 12 px to
        // look chunkier.
        assert!((radius - 12.0).abs() < 0.5);
    }
}
