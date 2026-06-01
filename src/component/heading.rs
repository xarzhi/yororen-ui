use gpui::{
    Div, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, div, prelude::FluentBuilder, px,
};

use crate::rtl;
use crate::theme::ActiveTheme;

pub fn heading(text: impl Into<SharedString>) -> Heading {
    Heading::new(text)
}

#[derive(Clone, Copy)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
}

#[derive(IntoElement)]
pub struct Heading {
    element_id: ElementId,
    base: Div,
    text: SharedString,
    level: HeadingLevel,
}

impl Heading {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            element_id: "ui:heading".into(),
            base: div(),
            text: text.into(),
            level: HeadingLevel::H2,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn level(mut self, level: HeadingLevel) -> Self {
        self.level = level;
        self
    }
}

impl ParentElement for Heading {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Heading {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Heading {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let direction = cx.theme().text_direction;
        let (size, weight) = match self.level {
            HeadingLevel::H1 => (32., FontWeight::BOLD),
            HeadingLevel::H2 => (24., FontWeight::SEMIBOLD),
            HeadingLevel::H3 => (18., FontWeight::SEMIBOLD),
        };

        let mut temp = self.base;
        let has_custom_align = temp
            .style()
            .text
            .as_ref()
            .map_or(false, |t| t.text_align.is_some());
        temp.id(self.element_id)
            .when(!has_custom_align, |this| {
                this.text_align(rtl::text_align_start(direction))
            })
            .text_size(px(size))
            .font_weight(weight)
            .text_color(cx.theme().content.primary)
            .child(self.text)
    }
}
