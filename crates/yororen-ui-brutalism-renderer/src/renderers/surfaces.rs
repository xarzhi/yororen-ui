//! Brutalist surface renderers: `Tooltip`, `Avatar`, `Panel`,
//! `Card`.

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div, px};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::style::{BRUTAL_BORDER, BRUTAL_RADIUS, BRUTAL_SMALL_BORDER_WIDTH, brutal_border_color};

// =====================================================================
// Tooltip
// =====================================================================

pub use yororen_ui_core::renderer::tooltip::{TooltipRenderState, TooltipRenderer};

pub struct BrutalTooltipRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalTooltipRenderer {
    pub fn bg(&self, _: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn fg(&self, _: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn padding(&self, _: &TooltipRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.tooltip.padding_x")
            .unwrap_or(10.0) as f32;
        let v = theme
            .get_number("tokens.control.tooltip.padding_y")
            .unwrap_or(6.0) as f32;
        Edges::symmetric(px(h), px(v))
    }
    pub fn font_size(&self, _: &TooltipRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tooltip.font_size")
            .unwrap_or(12.0) as f32)
    }
    pub fn border_radius(&self, _: &TooltipRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl TooltipRenderer for BrutalTooltipRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::tooltip::TooltipProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TooltipRenderState {
            has_custom_bg: props.has_custom_bg,
            has_custom_fg: props.has_custom_fg,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let fs = self.font_size(&state, theme);
        let r = self.border_radius(&state, theme);
        let open = props.state.read(cx).is_open();
        let mut el = gpui::div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .p(pad.top)
            .text_size(fs)
            .rounded(r)
            .child(props.text.clone());
        if !open {
            el = el.invisible();
        }
        el
    }
}

// =====================================================================
// Avatar
// =====================================================================

pub use yororen_ui_core::renderer::avatar::{AvatarRenderState, AvatarRenderer};

pub struct BrutalAvatarRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalAvatarRenderer {
    pub fn default_bg(&self, _: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }

    pub fn border_radius(&self, _: &AvatarRenderState, _: &Theme) -> Pixels {
        // Brutalism: square avatars (no pill, no radius).
        px(BRUTAL_RADIUS)
    }

    pub fn status_dot_size(&self, _: &AvatarRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.avatar.status_dot_size")
            .unwrap_or(12.0) as f32)
    }

    pub fn status_inset(&self, _: &AvatarRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.avatar.status_inset")
            .unwrap_or(2.0) as f32)
    }

    pub fn status_border_w(&self, _: &AvatarRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.avatar.border_w")
            .unwrap_or(BRUTAL_SMALL_BORDER_WIDTH as f64) as f32)
    }

    pub fn status_border_color(&self, _: &AvatarRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
}

impl AvatarRenderer for BrutalAvatarRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::avatar::AvatarProps,
        cx: &App,
    ) -> Div {
        let theme = cx.theme();
        let state = AvatarRenderState {
            has_custom_bg: props.has_custom_bg,
            has_status: props.has_status,
            is_circle: props.circle,
        };
        let bg = self.default_bg(&state, theme);
        let r = self.border_radius(&state, theme);
        let size = props.size.unwrap_or(px(40.0));
        let content = if let Some(initials) = &props.initials {
            div().child(initials.clone())
        } else if let Some(name) = &props.name {
            div().child(name.to_string())
        } else {
            div()
        };
        let mut el = div()
            .flex()
            .items_center()
            .justify_center()
            .bg(bg)
            .rounded(r)
            .size(size)
            .child(content);
        if props.has_status {
            let dot = self.status_dot_size(&state, theme);
            let inset = self.status_inset(&state, theme);
            let bw = self.status_border_w(&state, theme);
            let bc = self.status_border_color(&state, theme);
            el = el.child(
                div()
                    .absolute()
                    .right(inset)
                    .bottom(inset)
                    .size(dot)
                    .rounded(dot / 2.)
                    .border(bw)
                    .border_color(bc)
                    .bg(theme.get_color("status.success.bg").unwrap_or(BRUTAL_BORDER)),
            );
        }
        el
    }
}

// =====================================================================
// Panel
// =====================================================================

pub use yororen_ui_core::renderer::panel::{PanelRenderState, PanelRenderer};

pub struct BrutalPanelRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalPanelRenderer {
    pub fn bg(&self, _: &PanelRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or(BRUTAL_BORDER)
    }

    pub fn border(&self, _: &PanelRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }

    pub fn padding(&self, _: &PanelRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.panel.padding")
            .unwrap_or(16.0) as f32;
        Edges::all(px(p))
    }

    pub fn border_radius(&self, _: &PanelRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn shadow_alpha(&self, _: &PanelRenderState, _: &Theme) -> f32 {
        1.0
    }
}

impl PanelRenderer for BrutalPanelRenderer {
    fn compose(
        &self,
        _props: &yororen_ui_core::headless::panel::PanelProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = PanelRenderState {
            has_custom_bg: false,
            has_custom_border: false,
            has_custom_padding: false,
        };
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let pad = self.padding(&state, theme);
        let r = self.border_radius(&state, theme);
        div().bg(bg).border_color(border).p(pad.top).rounded(r)
    }
}

// =====================================================================
// Card
// =====================================================================

pub use yororen_ui_core::renderer::card::{CardRenderState, CardRenderer};

pub struct BrutalCardRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalCardRenderer {
    pub fn bg(&self, _: &CardRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }

    pub fn border(&self, _: &CardRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }

    pub fn padding(&self, _: &CardRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.card.padding")
            .unwrap_or(16.0) as f32;
        Edges::all(px(p))
    }

    pub fn border_radius(&self, _: &CardRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn shadow_alpha(&self, _: &CardRenderState, _: &Theme) -> f32 {
        1.0
    }
}

impl CardRenderer for BrutalCardRenderer {
    fn compose(
        &self,
        _props: &yororen_ui_core::headless::card::CardProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = CardRenderState { has_custom_bg: false };
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let pad = self.padding(&state, theme);
        let r = self.border_radius(&state, theme);
        div().bg(bg).border_color(border).p(pad.top).rounded(r)
    }
}

// End of card impl.
