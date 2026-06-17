//! `Row` — horizontal flex container (`div().flex().flex_row()`).
//!
//! The primary layout primitive for arranging children left-to-right.
//! All spacing values resolve through the active theme's
//! `tokens.spacing.*` paths; see [`super::types`] for the token mapping.

use gpui::{
    App, Div, ElementId, InteractiveElement, IntoElement, ParentElement, Stateful,
    StatefulInteractiveElement, Styled, div,
};

use crate::theme::ActiveTheme;
use super::types::{
    AlignItems, Inset, JustifyContent, Length, Spacing, apply_height, apply_width,
};

pub struct RowProps {
    pub id: ElementId,
    pub gap: Option<Spacing>,
    pub padding: Option<Inset>,
    pub px: Option<Spacing>,
    pub py: Option<Spacing>,
    pub margin: Option<Inset>,
    pub items: Option<AlignItems>,
    pub justify: Option<JustifyContent>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub scrollable: bool,
    pub children: Vec<gpui::AnyElement>,
}

pub fn row(id: impl Into<ElementId>, _cx: &mut App) -> RowProps {
    RowProps {
        id: id.into(),
        gap: None,
        padding: None,
        px: None,
        py: None,
        margin: None,
        items: None,
        justify: None,
        width: None,
        height: None,
        scrollable: false,
        children: Vec::new(),
    }
}

impl RowProps {
    pub fn gap(mut self, gap: impl Into<Spacing>) -> Self {
        self.gap = Some(gap.into());
        self
    }
    pub fn p(mut self, padding: impl Into<Inset>) -> Self {
        self.padding = Some(padding.into());
        self
    }
    pub fn px(mut self, val: impl Into<Spacing>) -> Self {
        self.px = Some(val.into());
        self
    }
    pub fn py(mut self, val: impl Into<Spacing>) -> Self {
        self.py = Some(val.into());
        self
    }
    pub fn m(mut self, margin: impl Into<Inset>) -> Self {
        self.margin = Some(margin.into());
        self
    }
    pub fn items(mut self, items: AlignItems) -> Self {
        self.items = Some(items);
        self
    }
    pub fn items_center(mut self) -> Self {
        self.items = Some(AlignItems::Center);
        self
    }
    pub fn justify(mut self, justify: JustifyContent) -> Self {
        self.justify = Some(justify);
        self
    }
    pub fn justify_between(mut self) -> Self {
        self.justify = Some(JustifyContent::Between);
        self
    }
    pub fn w(mut self, w: impl Into<Length>) -> Self {
        self.width = Some(w.into());
        self
    }
    pub fn w_full(mut self) -> Self {
        self.width = Some(Length::Full);
        self
    }
    pub fn h(mut self, h: impl Into<Length>) -> Self {
        self.height = Some(h.into());
        self
    }
    pub fn h_full(mut self) -> Self {
        self.height = Some(Length::Full);
        self
    }
    pub fn scrollable(mut self) -> Self {
        self.scrollable = true;
        self
    }
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }

    pub fn render(self, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let mut el: Stateful<Div> = div().id(self.id).flex().flex_row();

        if let Some(gap) = self.gap {
            el = el.gap(gap.to_pixels(theme));
        }
        if let Some(pad) = self.padding {
            el = el.p(pad.to_pixels(theme));
        }
        if let Some(px) = self.px {
            el = el.px(px.to_pixels(theme));
        }
        if let Some(py) = self.py {
            el = el.py(py.to_pixels(theme));
        }
        if let Some(margin) = self.margin {
            el = el.m(margin.to_pixels(theme));
        }
        if let Some(items) = self.items {
            el = match items {
                AlignItems::Start => el.items_start(),
                AlignItems::End => el.items_end(),
                AlignItems::Center => el.items_center(),
                AlignItems::Baseline => el.items_baseline(),
                AlignItems::Stretch => el,
            };
        }
        if let Some(justify) = self.justify {
            el = match justify {
                JustifyContent::Start => el.justify_start(),
                JustifyContent::End => el.justify_end(),
                JustifyContent::Center => el.justify_center(),
                JustifyContent::Between => el.justify_between(),
                JustifyContent::Around => el.justify_around(),
                JustifyContent::Evenly => el.justify_evenly(),
            };
        }
        if let Some(w) = self.width {
            el = apply_width(el, w);
        }
        if let Some(h) = self.height {
            el = apply_height(el, h);
        }
        if self.scrollable {
            el = el.overflow_scroll();
        }
        for child in self.children {
            el = el.child(child);
        }
        el
    }
}