//! `DividerRenderer` — the visual side of `Divider`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct DividerRenderState {
    pub vertical: bool,
}

pub trait DividerRenderer: Any + Send + Sync {
    fn color(&self, state: &DividerRenderState, theme: &Theme) -> Hsla;
    fn thickness(&self, state: &DividerRenderState, theme: &Theme) -> Pixels;
}
