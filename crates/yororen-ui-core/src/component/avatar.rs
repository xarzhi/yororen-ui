use std::sync::Arc;

use gpui::{
    Div, ElementId, Hsla, Image, InteractiveElement, IntoElement, ObjectFit, ParentElement,
    RenderOnce, Styled, StyledImage, div, img, prelude::FluentBuilder,
};

use crate::renderer::AvatarRenderState;
use crate::theme::ActiveTheme;

/// Creates a new avatar element.
pub fn avatar(image: Option<Arc<Image>>) -> Avatar {
    Avatar::new(image)
}

#[derive(Clone, Copy)]
pub enum AvatarShape {
    Circle,
    Square,
}

#[derive(IntoElement)]
pub struct Avatar {
    element_id: ElementId,
    base: Div,
    image: Option<Arc<Image>>,
    shape: AvatarShape,
    bg: Option<Hsla>,
    status: Option<Hsla>,
}

impl Avatar {
    pub fn new(image: Option<Arc<Image>>) -> Self {
        Self {
            element_id: "ui:avatar".into(),
            base: div(),
            image,
            shape: AvatarShape::Circle,
            bg: None,
            status: None,
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

    pub fn shape(mut self, shape: AvatarShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    pub fn status(mut self, color: impl Into<Hsla>) -> Self {
        self.status = Some(color.into());
        self
    }
}

impl ParentElement for Avatar {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Avatar {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let is_circle = matches!(self.shape, AvatarShape::Circle);

        let theme = cx.theme();
        let r = &theme.renderers.avatar;
        let state = AvatarRenderState {
            has_custom_bg: self.bg.is_some(),
            has_status: self.status.is_some(),
            is_circle,
        };
        let user_bg = self.bg;
        let user_radius = r.border_radius(&state, theme);

        let mut base = self.base.id(self.element_id);

        if let Some(bg) = user_bg {
            base = base.bg(bg);
        } else {
            base = base.bg(r.default_bg(&state, theme));
        }

        base = base.rounded(user_radius);

        let base = if let Some(image) = self.image {
            base.child(
                img(image)
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .rounded(user_radius),
            )
        } else {
            base.child("?")
        };

        let direction = cx.theme().text_direction;
        let status_dot_size = r.status_dot_size(&state, theme);
        let status_inset = r.status_inset(&state, theme);
        let status_border_w = r.status_border_w(&state, theme);
        let status_border_color = r.status_border_color(&state, theme);

        base.when_some(self.status, move |this, color| {
            this.child(
                div()
                    .absolute()
                    .when(direction.is_rtl(), |this| this.left(status_inset))
                    .when(!direction.is_rtl(), |this| this.right(status_inset))
                    .bottom(status_inset)
                    .size(status_dot_size)
                    .rounded_full()
                    .bg(color)
                    .border(status_border_w)
                    .border_color(status_border_color),
            )
        })
    }
}
