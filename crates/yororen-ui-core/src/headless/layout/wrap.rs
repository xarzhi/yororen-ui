//! `Wrap` — flex container with `flex_wrap()`.
//!
//! Children flow along the main axis and wrap to the next line
//! when they exceed the container's main-axis size.
//!
//! ## `row_gap` / `col_gap` limitation
//!
//! gpui-ce 0.3.3's `Styled` trait only exposes a single `.gap()`
//! method — there are no separate `row_gap()` / `col_gap()`
//! builders. The `row_gap` and `col_gap` props are stored as
//! separate fields for API completeness, but `render()` resolves
//! them to a single `.gap()` call with precedence:
//! `gap` > `row_gap` > `col_gap`. If gpui adds per-axis gap
//! methods in the future, the render method can be updated.

use gpui::{
    App, Div, ElementId, InteractiveElement, IntoElement, ParentElement, Stateful, Styled, div,
};

use crate::theme::ActiveTheme;
use super::types::{
    AlignItems, Inset, JustifyContent, Length, Spacing, apply_height, apply_width,
};

pub struct WrapProps {
    pub id: ElementId,
    pub gap: Option<Spacing>,
    pub row_gap: Option<Spacing>,
    pub col_gap: Option<Spacing>,
    pub padding: Option<Inset>,
    pub items: Option<AlignItems>,
    pub justify: Option<JustifyContent>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub children: Vec<gpui::AnyElement>,
}

pub fn wrap(id: impl Into<ElementId>, _cx: &mut App) -> WrapProps {
    WrapProps {
        id: id.into(),
        gap: None,
        row_gap: None,
        col_gap: None,
        padding: None,
        items: None,
        justify: None,
        width: None,
        height: None,
        children: Vec::new(),
    }
}

impl WrapProps {
    pub fn gap(mut self, gap: impl Into<Spacing>) -> Self {
        self.gap = Some(gap.into());
        self
    }
    pub fn row_gap(mut self, gap: impl Into<Spacing>) -> Self {
        self.row_gap = Some(gap.into());
        self
    }
    pub fn col_gap(mut self, gap: impl Into<Spacing>) -> Self {
        self.col_gap = Some(gap.into());
        self
    }
    pub fn p(mut self, padding: impl Into<Inset>) -> Self {
        self.padding = Some(padding.into());
        self
    }
    pub fn items(mut self, items: AlignItems) -> Self {
        self.items = Some(items);
        self
    }
    pub fn justify(mut self, justify: JustifyContent) -> Self {
        self.justify = Some(justify);
        self
    }
    pub fn w(mut self, w: Length) -> Self {
        self.width = Some(w);
        self
    }
    pub fn w_full(mut self) -> Self {
        self.width = Some(Length::Full);
        self
    }
    pub fn h(mut self, h: Length) -> Self {
        self.height = Some(h);
        self
    }
    pub fn h_full(mut self) -> Self {
        self.height = Some(Length::Full);
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
        let mut el: Stateful<Div> = div().id(self.id).flex().flex_wrap();

        // Precedence: gap > row_gap > col_gap. gpui has no per-axis
        // gap method, so all three resolve to a single `.gap()`.
        let effective_gap = self.gap.or(self.row_gap).or(self.col_gap);
        if let Some(g) = effective_gap {
            el = el.gap(g.to_pixels(theme));
        }
        if let Some(pad) = self.padding {
            el = el.p(pad.to_pixels(theme));
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
        for child in self.children {
            el = el.child(child);
        }
        el
    }
}