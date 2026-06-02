use gpui::{
    ClickEvent, Div, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
};

use crate::{
    component::{IconName, icon},
    renderer::TagRenderState,
    theme::ActiveTheme,
};

type OnCloseHandler = dyn Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App);

pub fn tag(text: impl Into<String>) -> Tag {
    Tag::new(text)
}

#[derive(IntoElement)]
pub struct Tag {
    base: Div,
    text: String,
    selected: bool,
    closable: bool,
    on_close: Option<Box<OnCloseHandler>>,
    tone: Option<Hsla>,
}

impl Tag {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            base: div(),
            text: text.into(),
            selected: false,
            closable: false,
            on_close: None,
            tone: None,
        }
    }

    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }

    pub fn closable(mut self, value: bool) -> Self {
        self.closable = value;
        self
    }

    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_close = Some(Box::new(handler));
        self
    }

    pub fn tone(mut self, color: impl Into<Hsla>) -> Self {
        self.tone = Some(color.into());
        self
    }
}

impl ParentElement for Tag {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Tag {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Tag {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let direction = cx.theme().text_direction;
        let user_tone = self.tone;

        let theme = cx.theme();
        let r = &theme.renderers.tag;
        let state = TagRenderState {
            selected: self.selected,
            has_custom_tone: user_tone.is_some(),
            closable: self.closable,
        };
        let bg = user_tone.unwrap_or_else(|| r.bg(&state, theme));
        let fg = r.fg(&state, theme);
        let min_height = r.min_height(&state, theme);
        let padding_x = r.padding_x(&state, theme);
        let font_size = r.font_size(&state, theme);
        let font_weight = r.font_weight(&state, theme);
        let radius = r.border_radius(&state, theme);
        let close_size = r.close_size(&state, theme);
        let close_hover_bg = r.close_hover_bg(&state, theme);

        let mut base = self
            .base
            .h(min_height)
            .px(padding_x)
            .rounded(radius)
            .bg(bg)
            .text_color(fg)
            .text_size(font_size)
            .font_weight(font_weight)
            .flex()
            .when(direction.is_rtl(), |this| this.flex_row_reverse())
            .when(!direction.is_rtl(), |this| this.flex_row())
            .items_center()
            .gap_1()
            .child(self.text);

        if self.closable {
            let on_close = self.on_close;
            base = base.child(
                div()
                    .id("ui:tag:close")
                    .w(close_size)
                    .h(close_size)
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(move |this| this.bg(close_hover_bg))
                    .cursor_pointer()
                    .child(
                        icon(IconName::Close)
                            .size(theme.tokens.sizes.icon_xs)
                            .color(fg),
                    )
                    .on_click(move |ev, window, cx| {
                        if let Some(handler) = &on_close {
                            handler(ev, window, cx);
                        }
                    }),
            );
        }

        base
    }
}
