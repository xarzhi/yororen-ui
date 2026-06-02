use gpui::{
    ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, div,
};

use crate::component::{ArrowDirection, IconName, icon};
use crate::theme::ActiveTheme;

/// A disclosure arrow with expanded/collapsed state.
///
/// This is a visual primitive only. It does not manage state by itself.
pub fn disclosure(id: impl Into<ElementId>) -> Disclosure {
    Disclosure::new().id(id)
}

#[derive(IntoElement)]
pub struct Disclosure {
    element_id: ElementId,
    base: gpui::Div,
    expanded: bool,
    size: gpui::Pixels,
}

impl Default for Disclosure {
    fn default() -> Self {
        Self::new()
    }
}

impl Disclosure {
    pub fn new() -> Self {
        Self {
            element_id: "ui:disclosure".into(),
            base: div(),
            expanded: false,
            size: gpui::px(0.),
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

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn size(mut self, size: gpui::Pixels) -> Self {
        self.size = size;
        self
    }
}

impl ParentElement for Disclosure {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Disclosure {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Disclosure {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl RenderOnce for Disclosure {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let element_id = self.element_id;
        let expanded = self.expanded;
        let size = self.size;
        let direction = cx.theme().text_direction;
        let size_f: f32 = size.into();
        let resolved_size = if size_f > 0.0 {
            size
        } else {
            cx.theme().tokens.control.disclosure.icon_size
        };

        self.base
            .id(element_id)
            .w(resolved_size)
            .h(resolved_size)
            .flex()
            .items_center()
            .justify_center()
            .text_color(cx.theme().content.tertiary)
            .child(
                icon(IconName::Arrow(if expanded {
                    ArrowDirection::Down
                } else if direction.is_rtl() {
                    ArrowDirection::Left
                } else {
                    ArrowDirection::Right
                }))
                .size(resolved_size),
            )
    }
}
