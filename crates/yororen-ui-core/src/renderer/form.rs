//! `FormRenderer` — visual side of `Form`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct FormRenderState {}

pub trait FormRenderer: Any + Send + Sync {
    fn gap(&self, state: &FormRenderState, theme: &Theme) -> Pixels;
    fn label_color(&self, state: &FormRenderState, theme: &Theme) -> Hsla;
    fn error_color(&self, state: &FormRenderState, theme: &Theme) -> Hsla;
    fn helper_color(&self, state: &FormRenderState, theme: &Theme) -> Hsla;
}
