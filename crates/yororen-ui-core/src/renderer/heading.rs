//! `HeadingRenderer` — the visual side of `Heading`.

use std::any::Any;

use gpui::{FontWeight, Hsla, Pixels};

use crate::headless::heading::HeadingLevel;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug)]
pub struct HeadingRenderState {
    pub level: HeadingLevel,
}

pub trait HeadingRenderer: Any + Send + Sync {
    fn size(&self, state: &HeadingRenderState, theme: &Theme) -> Pixels;
    fn weight(&self, state: &HeadingRenderState, theme: &Theme) -> FontWeight;
    fn color(&self, state: &HeadingRenderState, theme: &Theme) -> Hsla;
}
