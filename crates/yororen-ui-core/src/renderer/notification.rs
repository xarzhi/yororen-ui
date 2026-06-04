//! `NotificationRenderer` — visual side of the notification host.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct NotificationRenderState {}

pub trait NotificationRenderer: Any + Send + Sync {
    fn bg(&self, state: &NotificationRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &NotificationRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &NotificationRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &NotificationRenderState, theme: &Theme) -> Pixels;
    fn shadow_alpha(&self, state: &NotificationRenderState, theme: &Theme) -> f32;
}

pub struct TokenNotificationRenderer;

impl NotificationRenderer for TokenNotificationRenderer {
    fn bg(&self, _state: &NotificationRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }
    fn border(&self, _state: &NotificationRenderState, theme: &Theme) -> Hsla {
        theme.border.muted
    }
    fn padding(&self, _state: &NotificationRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(theme.tokens.spacing.inset_md)
    }
    fn border_radius(&self, _state: &NotificationRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.lg
    }
    fn shadow_alpha(&self, _state: &NotificationRenderState, theme: &Theme) -> f32 {
        theme.shadow.elevation_2.a
    }
}

pub fn arc_notification<T: NotificationRenderer + 'static>(r: T) -> Arc<dyn NotificationRenderer> {
    Arc::new(r)
}
