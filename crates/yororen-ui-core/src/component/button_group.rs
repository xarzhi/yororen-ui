use gpui::{
    AbsoluteLength, DefiniteLength, Div, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Styled, div, prelude::FluentBuilder,
};

pub fn button_group() -> ButtonGroup {
    ButtonGroup::new()
}

#[derive(IntoElement)]
pub struct ButtonGroup {
    element_id: ElementId,
    base: Div,
    children: Vec<gpui::AnyElement>,
    gap: Option<DefiniteLength>,
    radius: Option<AbsoluteLength>,
    connected: bool,
    vertical: bool,
}

impl Default for ButtonGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ButtonGroup {
    pub fn new() -> Self {
        Self {
            element_id: "ui:button-group".into(),
            base: div(),
            children: Vec::new(),
            gap: None,
            radius: None,
            connected: false,
            vertical: false,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Set the visual gap between sibling buttons. Ignored when
    /// `connected(true)` is set — connected groups deliberately
    /// touch.
    pub fn gap(mut self, gap: DefiniteLength) -> Self {
        self.gap = Some(gap);
        self
    }

    /// Outer corner radius applied to the group as a whole. Only
    /// honored when `connected(true)`; in the non-connected layout
    /// each button keeps its own radius.
    pub fn radius(mut self, radius: AbsoluteLength) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Glue the buttons into a single visual surface (a "segmented
    /// control" / pill). Works on both axes:
    /// - horizontal `connected`: classic toolbar pill.
    /// - vertical `connected`: tall stacked group sharing one
    ///   border-radius pair.
    ///
    /// `gap()` is intentionally ignored in this mode.
    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }

    /// Lay children out vertically instead of the default row.
    /// Combines cleanly with `connected(true)` to produce stacked
    /// pill groups. Children stretch to share the same width.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }
}

impl ParentElement for ButtonGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for ButtonGroup {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let gap = self.gap;
        let radius = self.radius;
        let connected = self.connected;
        let vertical = self.vertical;
        let element_id = self.element_id;

        // Layout axis:
        // - horizontal (default): `flex` + `items_center` lines up
        //   variable-height buttons on their vertical center.
        // - vertical: `flex_col` with the default `align-items:
        //   stretch` so children share the same width (sidebars
        //   and stacked CTAs look right).
        let mut group = self.base.id(element_id).flex();
        if vertical {
            group = group.flex_col();
        } else {
            group = group.items_center();
        }

        // `connected` mode glues the children into a visual pill /
        // segmented control: a single rounded outer border with
        // `overflow_hidden` so each child's own corners are clipped.
        // In that mode `gap` is intentionally ignored — the whole
        // point of `connected` is that the buttons touch. This rule
        // is the same on both axes; the only thing that differs is
        // which corner pair gets clipped, and `overflow_hidden`
        // handles that uniformly for `flex` and `flex_col`.
        if connected {
            group = group
                .when_some(radius, |this, radius| this.rounded(radius))
                .overflow_hidden();
        } else if let Some(gap) = gap {
            group = group.gap(gap);
        }

        group.children(self.children)
    }
}
