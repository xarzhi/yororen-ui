//! `FormRenderer` — visual side of `Form`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

pub use yororen_ui_core::renderer::form::{FormRenderState, FormRenderer};
use yororen_ui_core::theme::Theme;

pub struct TokenFormRenderer;

impl FormRenderer for TokenFormRenderer {
    fn gap(&self, _state: &FormRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.form.field_gap")
                .unwrap_or(0.0) as f32,
        )
    }
    fn label_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.secondary").unwrap_or_default()
    }
    fn error_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("status.error.bg").unwrap_or_default()
    }
    fn helper_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
}

pub fn arc_form<T: FormRenderer + 'static>(r: T) -> Arc<dyn FormRenderer> {
    Arc::new(r)
}
