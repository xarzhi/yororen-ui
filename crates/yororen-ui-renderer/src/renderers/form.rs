//! `FormRenderer` — visual side of `Form`.

use std::any::Any;
use std::sync::Arc;

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

pub struct TokenFormRenderer;

impl FormRenderer for TokenFormRenderer {
    fn gap(&self, _state: &FormRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.form.field_gap
    }
    fn label_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.content.secondary
    }
    fn error_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.status.error.bg
    }
    fn helper_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }
}

pub fn arc_form<T: FormRenderer + 'static>(r: T) -> Arc<dyn FormRenderer> {
    Arc::new(r)
}
