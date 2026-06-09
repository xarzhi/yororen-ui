//! `IconButtonRenderer` — visual side of `IconButton`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::variant::{ActionVariantKind, VariantStyle};
use crate::theme::Theme;

#[derive(Clone, Debug, Default)]
pub struct IconButtonRenderState {
    pub variant: ActionVariantKind,
    pub disabled: bool,
    pub has_custom_bg: bool,
    pub has_custom_hover_bg: bool,
    pub custom_style: Option<Arc<dyn VariantStyle>>,
}

pub trait IconButtonRenderer: Any + Send + Sync {
    fn bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla;
    fn hover_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla;
    fn active_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla;
    fn size(&self, state: &IconButtonRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &IconButtonRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &IconButtonRenderState, theme: &Theme) -> f32;
}
