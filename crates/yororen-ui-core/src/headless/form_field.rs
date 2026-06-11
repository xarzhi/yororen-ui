//! Headless `form_field` — a labelled field slot. The caller
//! provides the input element as a child; the headless layer only
//! owns the label and the error string.

use gpui::{App, Div, ElementId, InteractiveElement, SharedString, Stateful};
use crate::renderer::RendererContext;

#[derive(Clone, Debug)]
pub struct FormFieldProps {
    pub id: ElementId,
    pub name: SharedString,
    pub label: Option<SharedString>,
    pub required: bool,
    pub error: Option<SharedString>,
    pub help: Option<SharedString>,
}

pub fn form_field(
    id: impl Into<ElementId>,
    name: impl Into<SharedString>,
    _cx: &mut gpui::App,
) -> FormFieldProps {
    FormFieldProps {
        id: id.into(),
        name: name.into(),
        label: None,
        required: false,
        error: None,
        help: None,
    }
}

impl FormFieldProps {
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn required(mut self, v: bool) -> Self {
        self.required = v;
        self
    }
    pub fn error(mut self, e: impl Into<SharedString>) -> Self {
        self.error = Some(e.into());
        self
    }
    pub fn help(mut self, h: impl Into<SharedString>) -> Self {
        self.help = Some(h.into());
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the form field through the registered `FormFieldRenderer`.
    ///
    /// The input element should be added as a child after `.render(cx)`.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let r = cx
            .renderer_arc::<crate::renderer::markers::FormField, dyn crate::renderer::form_field::FormFieldRenderer>()
            .expect("FormFieldRenderer registered");
        r.compose(&self, cx)
    }
}
