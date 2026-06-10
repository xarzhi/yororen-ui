//! `TokenAvatarRenderer` — default `AvatarRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::avatar::AvatarProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::avatar::{AvatarRenderState, AvatarRenderer};

pub struct TokenAvatarRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenAvatarRenderer {
    pub fn default_bg(&self, _state: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }

    pub fn border_radius(&self, state: &AvatarRenderState, theme: &Theme) -> Pixels {
        if state.is_circle {
            gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32)
        } else {
            gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
        }
    }

    pub fn status_dot_size(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.avatar.status_dot_size")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn status_inset(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.avatar.status_inset")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn status_border_w(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.avatar.border_w")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn status_border_color(&self, _state: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
}

impl AvatarRenderer for TokenAvatarRenderer {
    fn compose(&self, props: &AvatarProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = AvatarRenderState {
            has_custom_bg: props.has_custom_bg,
            has_status: props.has_status,
            is_circle: props.circle,
        };
        let bg = self.default_bg(&state, theme);
        let r = self.border_radius(&state, theme);
        let size = props.size.unwrap_or(gpui::px(40.0));
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
                    .bg(theme.get_color("status.success.bg").unwrap_or_default()),
            );
        }
        el
    }
}

pub fn arc_avatar<T: AvatarRenderer + 'static>(r: T) -> Arc<dyn AvatarRenderer> {
    Arc::new(r)
}
