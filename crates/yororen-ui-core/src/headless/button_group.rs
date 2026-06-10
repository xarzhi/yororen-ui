//! Headless `button_group` — horizontal/vertical cluster of
//! buttons. No state of its own; the caller composes children.
//!
//! ```ignore
//! use yororen_ui::headless::button;
//! use yororen_ui::headless::button_group::button_group;
//!
//! // Default rendered — uses the installed GlobalTheme:
//! button_group("save-cluster", cx)
//!     .child(button("a", cx).caption("A").on_click(...).render(cx))
//!     .child(button("b", cx).caption("B").on_click(...).render(cx))
//!     .render(cx)
//! ```
//!
//! The renderer handles the **container**'s visual (flex
//! direction, gap, container-level decorations, segmented
//! corner rounding). Each child is independently styled by
//! `ButtonRenderer`.

use std::fmt;

use std::sync::Arc;

use gpui::{App, Div, ElementId, Stateful};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Headless props for `button_group`.
///
/// `children` stores the styled children the caller passed via
/// `.child(...)`. The renderer consumes them at `.render(cx)`
/// time to apply the segmented-control corner rounding.
/// `Stateful<Div>` is not `Clone`, so this struct is not
/// `Clone` either.
pub struct ButtonGroupProps {
    pub id: ElementId,
    pub orientation: ButtonGroupOrientation,
    /// When `true` (the default), the renderer produces a
    /// segmented-control look: no gap, shared border, only the
    /// first child's leading corners and the last child's
    /// trailing corners are rounded. When `false`, the renderer
    /// leaves each child as-is (use this when you want a loose
    /// cluster of independent buttons).
    pub attached: bool,
    pub children: Vec<Stateful<Div>>,
}

impl fmt::Debug for ButtonGroupProps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ButtonGroupProps")
            .field("id", &self.id)
            .field("orientation", &self.orientation)
            .field("attached", &self.attached)
            .field("children", &format_args!("[{} children]", self.children.len()))
            .finish()
    }
}

pub fn button_group(id: impl Into<ElementId>, _cx: &mut gpui::App) -> ButtonGroupProps {
    ButtonGroupProps {
        id: id.into(),
        orientation: ButtonGroupOrientation::default(),
        attached: true,
        children: Vec::new(),
    }
}

impl ButtonGroupProps {
    pub fn vertical(mut self) -> Self {
        self.orientation = ButtonGroupOrientation::Vertical;
        self
    }

    /// Toggle the segmented-control look. Defaults to `true` so
    /// `button_group(...)` out of the box is a segmented
    /// control. Pass `false` for a loose cluster where each
    /// child keeps its own rounded corners and a small gap.
    pub fn attached(mut self, v: bool) -> Self {
        self.attached = v;
        self
    }

    /// Append a styled child. The child is typically a
    /// `Stateful<Div>` produced by `button(...).render(cx)` so
    /// the button renderer can style it (bg / fg / padding /
    /// hover / active) before the group renderer decides
    /// corner rounding.
    pub fn child(mut self, c: Stateful<Div>) -> Self {
        self.children.push(c);
        self
    }

    /// Render the button group using the registered
    /// `ButtonGroupRenderer`. Returns a `Stateful<Div>` with
    /// the element id, the renderer-built container layout
    /// (flex direction, gap, border, overflow), the
    /// caller-supplied children already attached, and the
    /// segmented-control corner rounding applied.
    ///
    /// `ButtonGroupRenderer::compose` takes ownership of the
    /// props (so it can move the `Stateful<Div>` children out
    /// of `Vec`); this method is a thin pass-through.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::button_group::ButtonGroupRenderer;
        use crate::renderer::markers::ButtonGroup as ButtonGroupMarker;

        let r: &Arc<dyn ButtonGroupRenderer> = cx
            .renderer_arc::<ButtonGroupMarker, dyn ButtonGroupRenderer>()
            .expect("ButtonGroupRenderer registered");
        r.compose(self, cx)
    }
}
