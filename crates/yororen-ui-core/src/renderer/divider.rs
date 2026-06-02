//! `DividerRenderer` — the visual side of `Divider`.

use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct DividerRenderState {
    pub vertical: bool,
}

pub trait DividerRenderer: Send + Sync {
    fn color(&self, state: &DividerRenderState, theme: &Theme) -> Hsla;
    fn thickness(&self, state: &DividerRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenDividerRenderer;

impl DividerRenderer for TokenDividerRenderer {
    fn color(&self, _state: &DividerRenderState, theme: &Theme) -> Hsla {
        theme.border.divider
    }

    fn thickness(&self, _state: &DividerRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.divider.thickness
    }
}

pub fn arc_divider<T: DividerRenderer + 'static>(r: T) -> Arc<dyn DividerRenderer> {
    Arc::new(r)
}
