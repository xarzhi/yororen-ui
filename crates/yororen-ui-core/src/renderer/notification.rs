//! `NotificationRenderer` — visual side of the notification host.

use std::any::Any;

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
