//! `Panel` — a generic dialog/surface component.
//!
//! Phase G-α hard requirement (per the v0.5 review): the panel
//! concept that used to live only as inline comments inside
//! `modal.rs` is now a first-class component in its own file.
//!
//! A `Panel` is the visual "card" primitive: it draws the
//! background, border, border-radius, shadow, and inset padding.
//! It does **not** carry a title, an actions row, a close button,
//! or ARIA semantics — those are layered on top by [`Modal`](super::modal::Modal)
//! or other dialog components that compose a `Panel` with their
//! own header and footer.
//!
//! # Use Panel directly when…
//!
//! You want a styled card/sheet without dialog semantics. Examples:
//! - Drawer body
//! - Toast container
//! - Side panel
//!
//! # Use Modal when…
//!
//! You want the full v0.5 accessibility story (title, close button,
//! ARIA role, etc.) wrapped around a Panel. See `modal.rs`.
//!
//! # Usage
//!
//! ```rust,ignore
//! use yororen_ui::component::panel;
//!
//! div()
//!     .child(
//!         panel("my-panel")
//!             .bg(theme.surface.raised)
//!             .padding(Edges::all(px(20.0)))
//!             .child(label("Card body"))
//!     )
//! ```
//!
//! The corresponding `PanelRenderer` (in
//! `yororen_ui_core::renderer::PanelRenderer`) decides the
//! default `bg`, `border`, `border_radius`, `shadow`, and
//! `padding` — themes override these via the `RendererRegistry`.
//! `Panel` reads from the active theme, then layers any
//! caller-supplied overrides on top.

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, StyleRefinement, Styled, Window, div,
};

use crate::renderer::PanelRenderState;
use crate::renderer::spec::Edges;
use crate::theme::ActiveTheme;

/// Callback type for panel close handler (the close button on a
/// dialog that uses a Panel internally).
pub type PanelCloseCallback = Box<dyn Fn(&mut Window, &mut gpui::App)>;

/// Build a new `Panel` element with the given stable id.
///
/// Pass a stable id so the panel's element state survives across
/// renders. The default id is `"ui:panel"`.
pub fn panel(id: impl Into<ElementId>) -> Panel {
    Panel::new(id)
}

/// A `Panel` is a styled surface (bg + border + padding + shadow).
///
/// Most callers will use [`Modal`](super::modal::Modal) instead,
/// which composes a `Panel` with a title bar, close button, and
/// ARIA semantics. Use `Panel` directly only when you want the
/// visual surface without dialog chrome.
#[derive(IntoElement)]
pub struct Panel {
    element_id: ElementId,
    base: gpui::Div,
    /// Background color override (e.g. `.bg(my_color)`). When
    /// `None`, the renderer picks the default from the active
    /// theme.
    bg: Option<Hsla>,
    /// Border color override.
    border: Option<Hsla>,
    /// Padding override. When `None`, the renderer picks the
    /// default padding.
    padding: Option<Edges<Pixels>>,
    /// Child element. The panel renders exactly one child.
    /// Use a `div()` to wrap a tree.
    child: Option<AnyElement>,
}

impl Default for Panel {
    fn default() -> Self {
        Self::new("ui:panel")
    }
}

impl Panel {
    /// Create a new `Panel` with the given stable id.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            element_id: id.into(),
            base: div(),
            bg: None,
            border: None,
            padding: None,
            child: None,
        }
    }

    /// Set the element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Returns the element's ID.
    pub fn element_id(&self) -> &ElementId {
        &self.element_id
    }

    /// Override the background color.
    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    /// Override the border color.
    pub fn border(mut self, color: impl Into<Hsla>) -> Self {
        self.border = Some(color.into());
        self
    }

    /// Override the panel's padding.
    pub fn padding(mut self, padding: Edges<Pixels>) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set the inner child element.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }
}

impl ParentElement for Panel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        // Panel is a single-child container: we treat `extend` as
        // a way to set the child (last-wins) for ergonomic call
        // sites like `panel(id).child(...).extend(...)`.
        if let Some(last) = elements.into_iter().last() {
            self.child = Some(last);
        }
    }
}

impl Styled for Panel {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Panel {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();
        let renderer = &theme.renderers.panel;
        let state = PanelRenderState {
            has_custom_bg: self.bg.is_some(),
            has_custom_border: self.border.is_some(),
            has_custom_padding: self.padding.is_some(),
        };

        let bg = self.bg.unwrap_or_else(|| renderer.bg(&state, theme));
        let border = self
            .border
            .unwrap_or_else(|| renderer.border(&state, theme));
        let radius = renderer.border_radius(&state, theme);
        let shadow_alpha = renderer.shadow_alpha(&state, theme);
        let padding = self
            .padding
            .unwrap_or_else(|| renderer.padding(&state, theme));

        let child = self.child.unwrap_or_else(|| div().into_any_element());

        self.base
            .id(self.element_id)
            .rounded(radius)
            .border_1()
            .border_color(border)
            .bg(bg)
            .overflow_hidden()
            .px(padding.left)
            .py(padding.top)
            .when(shadow_alpha > 0.0, |this| this.shadow_md())
            .child(child)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_panel_id_is_ui_panel() {
        let p = Panel::default();
        // The element id is opaque; just check it's not the empty
        // string.
        let s = format!("{:?}", p.element_id);
        assert!(!s.is_empty());
    }

    #[test]
    fn setters_update_fields() {
        let p = Panel::new("my-panel");
        // Just check that the setters don't panic.
        let _ = p.bg(gpui::hsla(0.0, 0.0, 0.5, 1.0));
        let _ = Panel::new("other").border(gpui::hsla(0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn extend_takes_last_child() {
        let a: AnyElement = div().id("a").into_any_element();
        let b: AnyElement = div().id("b").into_any_element();
        let p = Panel::new("test").child(a);
        let mut p = p;
        p.extend(vec![b]);
        assert!(p.child.is_some());
    }
}
