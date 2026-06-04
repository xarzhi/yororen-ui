use gpui::{
    AbsoluteLength, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
};

use crate::renderer::FocusRingRenderState;
use crate::theme::ActiveTheme;

pub fn focus_ring() -> FocusRing {
    FocusRing::new()
}

#[derive(IntoElement)]
pub struct FocusRing {
    element_id: ElementId,
    base: Div,
    color: Option<Hsla>,
    radius: Option<AbsoluteLength>,
}

impl Default for FocusRing {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusRing {
    pub fn new() -> Self {
        Self {
            element_id: "ui:focus-ring".into(),
            base: div(),
            color: None,
            radius: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn radius(mut self, radius: AbsoluteLength) -> Self {
        self.radius = Some(radius);
        self
    }
}

impl ParentElement for FocusRing {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for FocusRing {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for FocusRing {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for FocusRing {}

impl RenderOnce for FocusRing {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let user_color = self.color;
        let radius = self.radius;
        let id = self.element_id.clone();

        let theme = cx.theme();
        let r = &theme.renderers.focus_ring;
        let state = FocusRingRenderState {
            has_custom_color: user_color.is_some(),
        };
        let ring_color = user_color.unwrap_or_else(|| r.color(&state, theme));
        let ring_width = r.width(&state, theme);

        self.base
            .id(id)
            .focusable()
            .when_some(radius, |this, radius| this.rounded(radius))
            .focus_visible(move |style| {
                // BorderSpec is a single uniform value; map to gpui's
                // border_1 / border_2 helpers from the renderer's width.
                let w: f32 = ring_width.into();
                if w >= 2.0 {
                    style.border_2().border_color(ring_color)
                } else {
                    style.border_1().border_color(ring_color)
                }
            })
    }
}
