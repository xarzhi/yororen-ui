use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
};

/// A small helper element that records its laid-out bounds into an `Entity<Bounds<Pixels>>`.
///
/// Useful for positioning floating UI (menus/popovers) relative to triggers while keeping them
/// inside the window.
///
/// **Identity contract**: this element intentionally returns
/// `None` from `Element::id()`. The tracked bounds are the inner
/// element's bounds, so the element that *uses* this tracker is
/// responsible for its own element id. In practice, callers wrap
/// the tracker in a parent `div().id(parent_id)`; the parent id
/// keys `use_keyed_state` and the tracker's `Entity<Bounds>` is
/// the side-effect sink.
pub(crate) struct BoundsTrackerElement {
    pub(crate) bounds_state: gpui::Entity<Bounds<gpui::Pixels>>,
    pub(crate) inner: gpui::AnyElement,
}

impl IntoElement for BoundsTrackerElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for BoundsTrackerElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut gpui::Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        (self.inner.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.bounds_state.update(cx, |state, _| *state = bounds);
        self.inner.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        cx: &mut App,
    ) {
        self.inner.paint(window, cx);
    }
}
