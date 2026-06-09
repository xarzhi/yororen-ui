//! Brutalist surface renderers: `Tooltip`, `Avatar`, `Panel`,
//! `Card`.

use gpui::{Hsla, Pixels, px};
use yororen_ui_core::theme::Theme;
use yororen_ui_default_renderer::renderers::spec::Edges;

use crate::style::{BRUTAL_BORDER, BRUTAL_RADIUS, BRUTAL_SMALL_BORDER_WIDTH, brutal_border_color};

// =====================================================================
// Tooltip
// =====================================================================

pub use yororen_ui_default_renderer::renderers::tooltip::{
    TooltipRenderState, TooltipRenderer,
};

pub struct BrutalTooltipRenderer;

impl TooltipRenderer for BrutalTooltipRenderer {
    fn bg(&self, _: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }

    fn fg(&self, _: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.fg")
            .unwrap_or(BRUTAL_BORDER)
    }

    fn padding(&self, _: &TooltipRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.tooltip.padding_x")
            .unwrap_or(10.0) as f32;
        let v = theme
            .get_number("tokens.control.tooltip.padding_y")
            .unwrap_or(6.0) as f32;
        Edges::symmetric(px(h), px(v))
    }

    fn font_size(&self, _: &TooltipRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.tooltip.font_size")
                .unwrap_or(12.0) as f32,
        )
    }

    fn border_radius(&self, _: &TooltipRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

// =====================================================================
// Avatar
// =====================================================================

pub use yororen_ui_default_renderer::renderers::avatar::{
    AvatarRenderState, AvatarRenderer,
};

pub struct BrutalAvatarRenderer;

impl AvatarRenderer for BrutalAvatarRenderer {
    fn default_bg(&self, _: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("surface.hover")
            .unwrap_or(BRUTAL_BORDER)
    }

    fn border_radius(&self, _: &AvatarRenderState, _: &Theme) -> Pixels {
        // Brutalism: square avatars (no pill, no radius).
        px(BRUTAL_RADIUS)
    }

    fn status_dot_size(&self, _: &AvatarRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.avatar.status_dot_size")
                .unwrap_or(12.0) as f32,
        )
    }

    fn status_inset(&self, _: &AvatarRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.avatar.status_inset")
                .unwrap_or(2.0) as f32,
        )
    }

    fn status_border_w(&self, _: &AvatarRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.avatar.border_w")
                .unwrap_or(BRUTAL_SMALL_BORDER_WIDTH as f64) as f32,
        )
    }

    fn status_border_color(&self, _: &AvatarRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
}

// =====================================================================
// Panel
// =====================================================================

pub use yororen_ui_default_renderer::renderers::panel::{PanelRenderState, PanelRenderer};

pub struct BrutalPanelRenderer;

impl PanelRenderer for BrutalPanelRenderer {
    fn bg(&self, _: &PanelRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("surface.raised")
            .unwrap_or(BRUTAL_BORDER)
    }

    fn border(&self, _: &PanelRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }

    fn padding(&self, _: &PanelRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.panel.padding")
            .unwrap_or(16.0) as f32;
        Edges::all(px(p))
    }

    fn border_radius(&self, _: &PanelRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    fn shadow_alpha(&self, _: &PanelRenderState, _: &Theme) -> f32 {
        // The renderer's `shadow` would be the brutalism 6px Y offset
        // shadow; `shadow_alpha` is the alpha multiplier the headless
        // applies on top. Use 1.0 to keep the shadow fully opaque.
        1.0
    }
}

// =====================================================================
// Card
// =====================================================================

pub use yororen_ui_default_renderer::renderers::card::{CardRenderState, CardRenderer};

pub struct BrutalCardRenderer;

impl CardRenderer for BrutalCardRenderer {
    fn bg(&self, _: &CardRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("surface.base")
            .unwrap_or(BRUTAL_BORDER)
    }

    fn border(&self, _: &CardRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }

    fn padding(&self, _: &CardRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.card.padding")
            .unwrap_or(16.0) as f32;
        Edges::all(px(p))
    }

    fn border_radius(&self, _: &CardRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    fn shadow_alpha(&self, _: &CardRenderState, _: &Theme) -> f32 {
        1.0
    }
}

// End of card impl.
