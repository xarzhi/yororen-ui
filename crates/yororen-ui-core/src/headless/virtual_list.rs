//! Headless `virtual_list` — wraps `gpui::ListState` with a row
//! renderer closure. The caller owns a [`VirtualListController`]
//! (a thin handle over `gpui::ListState`) and threads it into the
//! props via the factory; the renderer produces a `gpui::List`
//! element driven by the closure.
//!
//! ## Why a controller and not raw `gpui::ListState`?
//!
//! `gpui::ListState::new(count, alignment, overdraw)` is the only
//! constructor and there's no other public API surface to add — so
//! the controller is a small ergonomic wrapper that gives the
//! caller `reset / splice / scroll_to_reveal_item /
//! scroll_to_top / scroll_to_bottom` as `&self` methods (the
//! underlying `ListState` is `Rc<RefCell<…>>`, so `&self` mutates
//! the shared inner state). It also pins the default
//! alignment/overdraw so the caller doesn't have to repeat them
//! at every render call site.
//!
//! ## Render closure ownership
//!
//! The render_row closure is `Box<dyn FnMut + 'static>` and is
//! consumed by `render(cx)` (which delegates to the renderer that
//! hands it to `gpui::list`). The renderer is responsible for
//! wrapping the closure in whatever `RenderOnce` shell it needs
//! (e.g. a `VirtualListElement` in the default renderer) — this
//! module deliberately stops at "data + control" and does not
//! define a `RenderOnce` element, since render primitives are
//! out of scope for the headless layer.
//!
//! ## Uniform-height variant
//!
//! For the common case where every row is the same height,
//! [`uniform_virtual_list`] wraps `gpui::uniform_list` instead.
//! `gpui::uniform_list` measures only the first row and lays out
//! the rest in a straight line — this is significantly faster
//! than `gpui::list` (which measures every row through taffy) for
//! large lists. The trade-off is no `on_visible_range_change`
//! support (gpui's `UniformList` has no `set_scroll_handler`
//! equivalent), and the row closure must produce equal-height
//! elements.

use std::ops::Range;

use gpui::{
    App, Div, ElementId, InteractiveElement, ListAlignment, ListSizingBehavior, ListState, Pixels,
    ScrollStrategy, Stateful, UniformListScrollHandle, Window, px,
};

/// A `&self` handle over a `gpui::ListState` — the caller stores
/// one of these on their view and uses `reset / splice /
/// scroll_to_reveal_item / scroll_to_top / scroll_to_bottom` to
/// mutate the list across frames.
///
/// Cheap to clone (the inner `ListState` is `Rc<RefCell<…>>`).
#[derive(Clone, Debug)]
pub struct VirtualListController {
    state: ListState,
}

impl VirtualListController {
    /// Mint a controller with the given item count, alignment, and
    /// overdraw (in pixels above and below the visible area).
    pub fn new(item_count: usize, alignment: ListAlignment, overdraw: Pixels) -> Self {
        Self {
            state: ListState::new(item_count, alignment, overdraw),
        }
    }

    /// Mint a controller with `ListAlignment::Top` and a 16-px
    /// overdraw — the typical default for a scrolling list.
    pub fn with_default(item_count: usize) -> Self {
        Self::new(item_count, ListAlignment::Top, px(16.))
    }

    /// Snapshot the inner `gpui::ListState` — pass this to
    /// [`virtual_list`].
    pub fn state(&self) -> ListState {
        self.state.clone()
    }

    /// Inform the list that the item count has changed to
    /// `element_count` (used after adding/removing items in bulk).
    pub fn reset(&self, element_count: usize) {
        self.state.reset(element_count);
    }

    /// Inform the list that the items in `old_range` have been
    /// replaced by `count` new items.
    pub fn splice(&self, old_range: Range<usize>, count: usize) {
        self.state.splice(old_range, count);
    }

    /// Append `n` new items at the tail of the list. Unlike
    /// [`reset`](Self::reset), `append` preserves the current
    /// `logical_scroll_top` — the scroll position is unaffected
    /// because `splice(old_count..old_count, n)` only inserts new
    /// items beyond `scroll_top.item_ix`. This is the right call
    /// for "infinite loading" use cases: the user's scroll
    /// position stays put while new tail items become available.
    ///
    /// Internally this is `splice(item_count..item_count, n)`.
    /// No-op when `n == 0`.
    pub fn append(&self, n: usize) {
        if n > 0 {
            let cur = self.state.item_count();
            self.state.splice(cur..cur, n);
        }
    }

    /// Scroll the list so that item `ix` is fully visible.
    pub fn scroll_to_reveal_item(&self, ix: usize) {
        self.state.scroll_to_reveal_item(ix);
    }

    /// Scroll the list to the very top (item 0, no pixel offset).
    /// No-op if the list is empty.
    pub fn scroll_to_top(&self) {
        self.state.scroll_to(gpui::ListOffset::default());
    }

    /// Scroll the list so the last item is at the top of the
    /// viewport. No-op if the list is empty.
    ///
    /// **Why not `scroll_to_reveal_item(n - 1)`?** `gpui::ListState`
    /// only measures items that have been laid out (visible +
    /// overdraw). For a large list whose tail has never been on
    /// screen, the cumulative height up to `n - 1` is `0` (every
    /// item is `Unmeasured`), so `scroll_to_reveal_item` computes
    /// `goal_top = max(0, 0 - viewport_height + padding) = 0` and
    /// the list refuses to scroll. `scroll_to` directly sets
    /// `logical_scroll_top.item_ix = n - 1`, which forces the next
    /// paint to render (and thus measure) the tail. The trade-off
    /// is that the last item appears at the **top** of the viewport
    /// rather than the bottom — acceptable for a "jump to bottom"
    /// affordance, and the only robust option without a
    /// `measure_all` step.
    pub fn scroll_to_bottom(&self) {
        let n = self.state.item_count();
        if n > 0 {
            self.state.scroll_to(gpui::ListOffset {
                item_ix: n - 1,
                offset_in_item: px(0.),
            });
        }
    }
}

/// Type of the per-item render closure. Called by `gpui::list` for
/// each visible row, with the item index and the gpui context.
pub type RenderRowFn = Box<dyn FnMut(usize, &mut Window, &mut App) -> gpui::AnyElement + 'static>;

/// Per-frame callback invoked by the renderer when the visible
/// range changes (scroll, resize, or item-count change). The
/// arguments are `(visible_range, total_count, window, cx)`.
/// Typical uses: infinite scrolling (when `range.end + N >= total`,
/// fetch more items and call `controller.reset(new_total)`), lazy
/// loading of row-specific resources, and viewport-out resource
/// recycling.
///
/// The callback is `FnMut + 'static` — **not** `Send + Sync` —
/// because the renderer hands it to
/// `gpui::ListState::set_scroll_handler`, which itself only
/// requires `FnMut + 'static` (ListState is `Rc<RefCell<…>>`,
/// strictly single-threaded). Relaxing the bound lets callers
/// capture other non-Send state on the same thread (e.g. a clone
/// of the `VirtualListController` itself).
pub type VisibleRangeCallback =
    Box<dyn FnMut(Range<usize>, usize, &mut Window, &mut App) + 'static>;

/// A snapshot of the data + control layer for a virtual list.
/// Constructed by [`virtual_list`], mutated through builder methods,
/// and consumed by `.render(cx)` (which hands it to the
/// `VirtualListRenderer`).
pub struct VirtualListProps {
    pub id: ElementId,
    pub item_count: usize,
    pub alignment: ListAlignment,
    pub overdraw: Pixels,
    pub sizing_behavior: ListSizingBehavior,
    pub state: ListState,
    pub render_row: Option<RenderRowFn>,
    pub on_visible_range_change: Option<VisibleRangeCallback>,
}

impl std::fmt::Debug for VirtualListProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualListProps")
            .field("id", &self.id)
            .field("item_count", &self.item_count)
            .field("alignment", &self.alignment)
            .field("overdraw", &self.overdraw)
            .field("sizing_behavior", &self.sizing_behavior)
            .field("render_row", &"<fn>")
            .field(
                "on_visible_range_change",
                &self.on_visible_range_change.as_ref().map(|_| "<fn>"),
            )
            .finish()
    }
}

/// Build a headless `VirtualListProps` for the given `id`, driven by
/// the caller's [`VirtualListController`].
///
/// The returned props need at least `.row(closure)` before
/// `.render(cx)` is meaningful — without a row closure the list has
/// nothing to draw.
pub fn virtual_list(
    id: impl Into<ElementId>,
    controller: &VirtualListController,
    _cx: &mut App,
) -> VirtualListProps {
    VirtualListProps {
        id: id.into(),
        item_count: controller.state.item_count(),
        alignment: ListAlignment::Top,
        overdraw: px(16.),
        sizing_behavior: ListSizingBehavior::default(),
        state: controller.state(),
        render_row: None,
        on_visible_range_change: None,
    }
}

impl VirtualListProps {
    /// Update the item count (without touching the controller's
    /// inner state — the caller should also call
    /// `controller.reset(n)` so the ListState stays in sync).
    pub fn item_count(mut self, n: usize) -> Self {
        self.item_count = n;
        self
    }

    /// Top vs bottom alignment — see `gpui::ListAlignment`.
    pub fn alignment(mut self, a: ListAlignment) -> Self {
        self.alignment = a;
        self
    }

    /// Overdraw in pixels — extra space rendered above and below
    /// the visible area to smooth out scrolling.
    pub fn overdraw(mut self, px: Pixels) -> Self {
        self.overdraw = px;
        self
    }

    /// Sizing behavior for layout. `Infer` makes the list adopt the
    /// height of its tallest item; `Auto` (default) lets the parent
    /// drive the size.
    pub fn sizing(mut self, s: ListSizingBehavior) -> Self {
        self.sizing_behavior = s;
        self
    }

    /// Provide the closure that produces each visible row.
    pub fn row(
        mut self,
        f: impl FnMut(usize, &mut Window, &mut App) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.render_row = Some(Box::new(f));
        self
    }

    /// Register a callback that fires whenever the visible item
    /// range changes (scroll, resize, item-count change). The
    /// renderer wires this through `ListState::set_scroll_handler`.
    ///
    /// Typical use — infinite scrolling:
    /// ```ignore
    /// virtual_list("feed", &controller, cx)
    ///     .row(...)
    ///     .on_visible_range_change(move |range, total, _w, cx| {
    ///         if range.end + 50 >= total {
    ///             // Within 50 items of the end — load another batch.
    ///             controller.reset(total + 100);
    ///         }
    ///     })
    /// ```
    pub fn on_visible_range_change<F>(mut self, f: F) -> Self
    where
        F: FnMut(Range<usize>, usize, &mut Window, &mut App) + 'static,
    {
        self.on_visible_range_change = Some(Box::new(f));
        self
    }

    /// `apply` for callers that want a custom render path. The
    /// virtual list is closure-driven, so `apply` is vestigial —
    /// it just sets the id and lets the caller provide their own
    /// visual.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render through the registered `VirtualListRenderer`. The
    /// props are consumed so the row closure can be transferred
    /// into the renderer's `gpui::list` element.
    pub fn render(mut self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::VirtualList as VirtualListMarker;
        use crate::renderer::virtual_list::VirtualListRenderer;
        let r: &std::sync::Arc<dyn VirtualListRenderer> = cx
            .renderer_arc::<VirtualListMarker, dyn VirtualListRenderer>()
            .expect("VirtualListRenderer registered");
        // Pull the closure out so we can hand it (and only it) to
        // the renderer; the renderer decides what `RenderOnce`
        // shell to wrap it in.
        let render_row = self
            .render_row
            .take()
            .expect("VirtualListProps::render requires .row(closure)");
        r.compose(self, render_row, cx)
    }
}

// =====================================================================
// Uniform-height variant
// =====================================================================

/// A `&self` handle over a `gpui::UniformListScrollHandle` — the
/// caller stores one of these on their view and uses
/// `scroll_to_item / scroll_to_top / scroll_to_bottom` to drive
/// the list's scroll position across frames.
///
/// Unlike [`VirtualListController`], the uniform variant does not
/// own the item count: `gpui::uniform_list` takes the count
/// directly at construction, so the caller passes it as the second
/// argument to [`uniform_virtual_list`] each frame.
///
/// Cheap to clone (the inner handle is `Rc<RefCell<…>>`).
#[derive(Clone, Debug)]
pub struct UniformVirtualListController {
    handle: UniformListScrollHandle,
}

impl UniformVirtualListController {
    /// Mint a fresh controller. The bound list count is whatever
    /// the caller passes to [`uniform_virtual_list`] each frame.
    pub fn new() -> Self {
        Self {
            handle: UniformListScrollHandle::new(),
        }
    }

    /// Snapshot the inner scroll handle — pass this to
    /// `UniformList::track_scroll`. Renderers do this internally.
    pub fn handle(&self) -> UniformListScrollHandle {
        self.handle.clone()
    }

    /// Scroll the list so that item `ix` is at the top of the
    /// viewport. Non-strict: no-op if the item is already fully
    /// visible.
    pub fn scroll_to_item(&self, ix: usize) {
        self.handle.scroll_to_item(ix, ScrollStrategy::Top);
    }

    /// Scroll the list to item 0 at the top of the viewport.
    pub fn scroll_to_top(&self) {
        self.handle.scroll_to_item(0, ScrollStrategy::Top);
    }

    /// Scroll the list to the bottom (the last item is fully
    /// visible at the bottom of the viewport).
    pub fn scroll_to_bottom(&self) {
        self.handle.scroll_to_bottom();
    }
}

impl Default for UniformVirtualListController {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of the per-item render closure for a uniform virtual list.
/// Same signature as the heterogeneous-height variant
/// ([`RenderRowFn`]); the renderer is responsible for adapting it
/// to `gpui::uniform_list`'s `Fn(Range<usize>, …) -> Vec<R>`
/// signature (typically by interior-mutability wrapping).
pub type UniformRenderRowFn =
    Box<dyn FnMut(usize, &mut Window, &mut App) -> gpui::AnyElement + 'static>;

/// A snapshot of the data + control layer for a uniform-height
/// virtual list. Constructed by [`uniform_virtual_list`], mutated
/// through builder methods, and consumed by `.render(cx)` (which
/// hands it to the `UniformVirtualListRenderer`).
pub struct UniformVirtualListProps {
    pub id: ElementId,
    pub item_count: usize,
    pub sizing_behavior: ListSizingBehavior,
    pub handle: UniformListScrollHandle,
    pub render_row: Option<UniformRenderRowFn>,
}

impl std::fmt::Debug for UniformVirtualListProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UniformVirtualListProps")
            .field("id", &self.id)
            .field("item_count", &self.item_count)
            .field("sizing_behavior", &self.sizing_behavior)
            .field("render_row", &"<fn>")
            .finish()
    }
}

/// Build a headless `UniformVirtualListProps` for the given `id`
/// and item count, driven by the caller's
/// [`UniformVirtualListController`].
///
/// The returned props need at least `.row(closure)` before
/// `.render(cx)` is meaningful — without a row closure the list
/// has nothing to draw.
pub fn uniform_virtual_list(
    id: impl Into<ElementId>,
    item_count: usize,
    controller: &UniformVirtualListController,
    _cx: &mut App,
) -> UniformVirtualListProps {
    UniformVirtualListProps {
        id: id.into(),
        item_count,
        sizing_behavior: ListSizingBehavior::default(),
        handle: controller.handle(),
        render_row: None,
    }
}

impl UniformVirtualListProps {
    /// Update the item count.
    pub fn item_count(mut self, n: usize) -> Self {
        self.item_count = n;
        self
    }

    /// Sizing behavior for layout. `Infer` makes the list adopt
    /// `item_height * item_count`; `Auto` (default) lets the
    /// parent drive the size.
    pub fn sizing(mut self, s: ListSizingBehavior) -> Self {
        self.sizing_behavior = s;
        self
    }

    /// Provide the closure that produces each visible row. The
    /// closure must produce equal-height elements — uniform_list
    /// measures only the first row and assumes the rest have the
    /// same height.
    pub fn row<F>(mut self, f: F) -> Self
    where
        F: FnMut(usize, &mut Window, &mut App) -> gpui::AnyElement + 'static,
    {
        self.render_row = Some(Box::new(f));
        self
    }

    /// `apply` for callers that want a custom render path. The
    /// uniform list is closure-driven, so `apply` is vestigial —
    /// it just sets the id and lets the caller provide their own
    /// visual.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render through the registered `UniformVirtualListRenderer`.
    /// The props are consumed so the row closure can be
    /// transferred into the renderer's `gpui::uniform_list`
    /// element.
    pub fn render(mut self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::UniformVirtualList as Marker;
        use crate::renderer::uniform_virtual_list::UniformVirtualListRenderer;
        let r: &std::sync::Arc<dyn UniformVirtualListRenderer> = cx
            .renderer_arc::<Marker, dyn UniformVirtualListRenderer>()
            .expect("UniformVirtualListRenderer registered");
        let render_row = self
            .render_row
            .take()
            .expect("UniformVirtualListProps::render requires .row(closure)");
        r.compose(self, render_row, cx)
    }
}
