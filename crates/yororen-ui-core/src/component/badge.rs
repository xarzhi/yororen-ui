use gpui::{
    Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, div, prelude::FluentBuilder,
};

use crate::renderer::BadgeRenderState;
use crate::rtl;
use crate::theme::ActiveTheme;

/// Creates a new badge element.
pub fn badge(text: impl Into<SharedString>) -> Badge {
    Badge::new(text)
}

#[derive(IntoElement)]
pub struct Badge {
    element_id: ElementId,
    base: Div,
    text: SharedString,
    tone: Option<Hsla>,
}

impl Badge {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            element_id: "ui:badge".into(),
            base: div(),
            text: text.into(),
            tone: None,
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

    pub fn tone(mut self, color: impl Into<Hsla>) -> Self {
        self.tone = Some(color.into());
        self
    }
}

impl ParentElement for Badge {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Badge {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let direction = cx.theme().text_direction;
        let element_id = self.element_id;
        let user_tone = self.tone;

        let theme = cx.theme();
        let r = &theme.renderers.badge;
        let state = BadgeRenderState {
            has_custom_tone: user_tone.is_some(),
        };
        let bg = user_tone.unwrap_or_else(|| r.bg(&state, theme));
        let fg = r.fg(&state, theme);
        let padding_x = r.padding_x(&state, theme);
        let height = r.height(&state, theme);
        let font_size = r.font_size(&state, theme);
        let font_weight = r.font_weight(&state, theme);
        let radius = r.border_radius(&state, theme);

        let mut temp = self.base;
        let has_custom_align = temp
            .style()
            .text
            .as_ref()
            .is_some_and(|t| t.text_align.is_some());
        temp.id(element_id)
            .when(!has_custom_align, |this| {
                this.text_align(rtl::text_align_start(direction))
            })
            .px(padding_x)
            .h(height)
            .rounded(radius)
            .bg(bg)
            .text_color(fg)
            .text_size(font_size)
            .font_weight(font_weight)
            .flex()
            .items_center()
            .child(self.text)
    }
}
