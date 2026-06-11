//! `TokenFormFieldRenderer` — default `FormFieldRenderer` impl.
//!
//! Stacks label (with required indicator), the caller-supplied input
//! child, helper text, and error text vertically with theme-derived
//! gaps and colours.

use std::sync::Arc;

use gpui::{InteractiveElement, App, Div, Hsla, ParentElement, Pixels, SharedString, Stateful, Styled, div};

use yororen_ui_core::headless::form_field::FormFieldProps;
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub use yororen_ui_core::renderer::form_field::{FormFieldRenderState, FormFieldRenderer};

pub struct TokenFormFieldRenderer;

impl TokenFormFieldRenderer {
    pub fn label_color(&self, _state: &FormFieldRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn error_color(&self, _state: &FormFieldRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.error").unwrap_or_else(|| theme.get_color("status.danger").unwrap_or_default())
    }
    pub fn helper_color(&self, _state: &FormFieldRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    pub fn gap(&self, _state: &FormFieldRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.gap_1").unwrap_or(4.0) as f32)
    }
    pub fn font_size(&self, _state: &FormFieldRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.typography.font_size_sm").unwrap_or(12.0) as f32)
    }
}

impl FormFieldRenderer for TokenFormFieldRenderer {
    fn compose(&self, props: &FormFieldProps, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = FormFieldRenderState {
            has_error: props.error.is_some(),
            required: props.required,
        };
        let label_color = self.label_color(&state, theme);
        let error_color = self.error_color(&state, theme);
        let helper_color = self.helper_color(&state, theme);
        let gap = self.gap(&state, theme);
        let font_size = self.font_size(&state, theme);

        let mut wrapper = div()
            .id(props.id.clone())
            .flex()
            .flex_col()
            .gap(gap);

        // Label row
        if let Some(label) = &props.label {
            let mut label_text = SharedString::from(label.clone());
            if props.required {
                label_text = SharedString::from(format!("{} *", label));
            }
            wrapper = wrapper.child(
                div()
                    .text_size(font_size)
                    .text_color(label_color)
                    .child(label_text),
            );
        }

        // Input child will be appended by caller after .render(cx)

        // Error text
        if let Some(error) = &props.error {
            wrapper = wrapper.child(
                div()
                    .text_size(font_size)
                    .text_color(error_color)
                    .child(error.clone()),
            );
        }

        // Help text
        if let Some(help) = &props.help {
            wrapper = wrapper.child(
                div()
                    .text_size(font_size)
                    .text_color(helper_color)
                    .child(help.clone()),
            );
        }

        wrapper
    }
}

pub fn arc_form_field<T: FormFieldRenderer + 'static>(r: T) -> Arc<dyn FormFieldRenderer> {
    Arc::new(r)
}
