use gpui::{
    Div, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, div,
};

use crate::renderer::DividerRenderState;
use crate::theme::ActiveTheme;

/// Creates a new divider element.
pub fn divider() -> Divider {
    Divider::new()
}

#[derive(IntoElement)]
pub struct Divider {
    element_id: ElementId,
    base: Div,
    vertical: bool,
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}

impl Divider {
    pub fn new() -> Self {
        Self {
            element_id: "ui:divider".into(),
            base: div(),
            vertical: false,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    pub fn vertical(mut self, value: bool) -> Self {
        self.vertical = value;
        self
    }
}

impl ParentElement for Divider {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Divider {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Divider {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let element_id = self.element_id;
        let theme = cx.theme();
        let r = &theme.renderers.divider;
        let state = DividerRenderState {
            vertical: self.vertical,
        };
        let color = r.color(&state, theme);
        let thickness = r.thickness(&state, theme);

        let base = self.base.id(element_id);

        if self.vertical {
            base.w(thickness).h_full().bg(color)
        } else {
            base.h(thickness).w_full().bg(color)
        }
    }
}
