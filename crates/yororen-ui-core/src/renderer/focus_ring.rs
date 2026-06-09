//! `FocusRingRenderer` — the visual side of `FocusRing`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct FocusRingRenderState {
    pub has_custom_color: bool,
}

pub trait FocusRingRenderer: Any + Send + Sync {
    fn color(&self, state: &FocusRingRenderState, theme: &Theme) -> Hsla;
    fn width(&self, state: &FocusRingRenderState, theme: &Theme) -> Pixels;
}
