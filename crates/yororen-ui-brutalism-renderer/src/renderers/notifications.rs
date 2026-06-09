//! Brutalist notification renderers: `Toast`, `Notification`.

use gpui::{Hsla, Pixels, px};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::style::{BRUTAL_BORDER, BRUTAL_RADIUS, brutal_border_color};

// =====================================================================
// Toast
// =====================================================================

pub use yororen_ui_core::renderer::toast::{ToastRenderState, ToastRenderer};

pub struct BrutalToastRenderer;

impl ToastRenderer for BrutalToastRenderer {
    fn bg(&self, _: &ToastRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or(BRUTAL_BORDER)
    }
    fn fg(&self, _: &ToastRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    fn padding(&self, _: &ToastRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.toast.padding")
            .unwrap_or(12.0) as f32;
        Edges::all(px(p))
    }
    fn border_radius(&self, _: &ToastRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    fn border(&self, _: &ToastRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    fn shadow_alpha(&self, _: &ToastRenderState, _: &Theme) -> f32 {
        1.0
    }
}

// =====================================================================
// Notification
// =====================================================================

pub use yororen_ui_core::renderer::notification::{NotificationRenderState, NotificationRenderer};

pub struct BrutalNotificationRenderer;

impl NotificationRenderer for BrutalNotificationRenderer {
    fn bg(&self, _: &NotificationRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or(BRUTAL_BORDER)
    }
    fn border(&self, _: &NotificationRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    fn padding(&self, _: &NotificationRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.notification.padding")
            .unwrap_or(16.0) as f32;
        Edges::all(px(p))
    }
    fn border_radius(&self, _: &NotificationRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    fn shadow_alpha(&self, _: &NotificationRenderState, _: &Theme) -> f32 {
        1.0
    }
}
