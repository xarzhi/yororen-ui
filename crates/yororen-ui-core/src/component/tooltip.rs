use gpui::{
    AnyView, AppContext, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Render,
    RenderOnce, Styled, div,
};

use crate::renderer::TooltipRenderState;
use crate::theme::ActiveTheme;

/// Defines the placement position of a tooltip relative to its trigger element.
pub enum TooltipPlacement {
    /// Automatically determines the best placement based on available space.
    Auto,
    /// Positions the tooltip above the trigger element.
    Top,
    /// Positions the tooltip to the right of the trigger element.
    Right,
    /// Positions the tooltip below the trigger element.
    Bottom,
    /// Positions the tooltip to the left of the trigger element.
    Left,
}

/// Creates a new tooltip with text content.
///
/// Use `.placement()` to control positioning and `.bg()`/`.text_color()` for customization.
/// The tooltip is typically used with `.with_tooltip()` on interactive elements.
///
/// # Example
/// ```rust,ignore
/// use yororen_ui::component::{button, tooltip, TooltipPlacement};
///
/// let btn = button("my-button")
///     .child("Hover me")
///     .with_tooltip(tooltip("Helpful information").placement(TooltipPlacement::Bottom));
/// ```
pub fn tooltip(content: impl Into<String>) -> Tooltip {
    Tooltip::text(content)
}

/// A tooltip component that displays contextual information on hover.
///
/// Tooltips are typically used with `.with_tooltip()` on interactive elements like buttons or icons.
/// The tooltip will automatically position itself based on available space, or you can specify
/// a fixed placement using `.placement()`.
#[derive(IntoElement)]
pub struct Tooltip {
    element_id: ElementId,
    content: String,
    placement: TooltipPlacement,
    bg: Option<Hsla>,
    text_color: Option<Hsla>,
    /// Whether to honour an Escape keypress by closing any parent
    /// overlay / popover that hosts this tooltip. The tooltip
    /// itself is a passive element; this flag is consumed by
    /// the surrounding `.with_tooltip()` host. Default: `true`.
    dismiss_on_escape: bool,
}

struct TooltipView {
    element_id: ElementId,
    content: String,
    bg: Option<Hsla>,
    text_color: Option<Hsla>,
}

impl Tooltip {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            element_id: "ui:tooltip".into(),
            content: content.into(),
            placement: TooltipPlacement::Auto,
            bg: None,
            text_color: None,
            dismiss_on_escape: true,
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

    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }

    /// Set whether to honour an Escape keypress. Default: `true`.
    /// See [`Tooltip::dismiss_on_escape`] for details.
    pub fn dismiss_on_escape(mut self, dismiss: bool) -> Self {
        self.dismiss_on_escape = dismiss;
        self
    }

    pub fn build(self) -> impl Fn(&mut gpui::Window, &mut gpui::App) -> AnyView {
        let element_id = self.element_id;
        let content = self.content;
        let _placement = self.placement;
        let bg = self.bg;
        let text_color = self.text_color;
        move |_, cx| {
            cx.new(|_| TooltipView {
                element_id: element_id.clone(),
                content: content.clone(),
                bg,
                text_color,
            })
            .into()
        }
    }
}

impl Render for TooltipView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let r = &theme.renderers.tooltip;
        let state = TooltipRenderState {
            has_custom_bg: self.bg.is_some(),
            has_custom_fg: self.text_color.is_some(),
        };
        let bg = self.bg.unwrap_or_else(|| r.bg(&state, theme));
        let fg = self.text_color.unwrap_or_else(|| r.fg(&state, theme));
        let padding = r.padding(&state, theme);
        let font_size = r.font_size(&state, theme);
        let radius = r.border_radius(&state, theme);

        div()
            .id(self.element_id.clone())
            .px(padding.left)
            .py(padding.top)
            .rounded(radius)
            .text_size(font_size)
            .bg(bg)
            .text_color(fg)
            .child(self.content.clone())
    }
}

impl RenderOnce for Tooltip {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div().id(self.element_id).child(self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_dismiss_on_escape_is_true() {
        let t = Tooltip::text("hi");
        assert!(t.dismiss_on_escape);
    }
    #[test]
    fn dismiss_on_escape_setter_updates() {
        let t = Tooltip::text("hi").dismiss_on_escape(false);
        assert!(!t.dismiss_on_escape);
    }
}
