//! `DisclosureRenderer` — visual side of `Disclosure`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct DisclosureRenderState {
    pub open: bool,
}

pub trait DisclosureRenderer: Any + Send + Sync {
    fn trigger_bg(&self, state: &DisclosureRenderState, theme: &Theme) -> Hsla;
    fn trigger_fg(&self, state: &DisclosureRenderState, theme: &Theme) -> Hsla;
    fn trigger_hover_bg(&self, state: &DisclosureRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &DisclosureRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &DisclosureRenderState, theme: &Theme) -> Pixels;
    fn chevron_rotation(&self, state: &DisclosureRenderState, theme: &Theme) -> f32;
    fn body_padding(&self, state: &DisclosureRenderState, theme: &Theme) -> Pixels;
}
