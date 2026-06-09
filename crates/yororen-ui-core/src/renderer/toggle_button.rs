//! `ToggleButtonRenderer` — visual side of `ToggleButton`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::variant::{ActionVariantKind, VariantStyle};
use crate::theme::Theme;

#[derive(Clone, Debug, Default)]
pub struct ToggleButtonRenderState {
    pub variant: ActionVariantKind,
    pub selected: bool,
    pub disabled: bool,
    pub custom_style: Option<Arc<dyn VariantStyle>>,
}

pub trait ToggleButtonRenderer: Any + Send + Sync {
    fn bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla;
    fn hover_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla;
    fn active_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &ToggleButtonRenderState, theme: &Theme) -> f32;
}
