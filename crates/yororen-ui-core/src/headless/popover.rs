//! Headless `popover` — owns `open` + placement + a stored trigger
//! and content element. The renderer watches the `Entity<PopoverState>`
//! and lays out the content via `gpui::anchored` when `open` flips.

use std::sync::Arc;

use gpui::{
    App, AppContext, Bounds, Div, ElementId, Entity, InteractiveElement, Pixels, Size, Stateful,
};

/// Preferred placement of a popover relative to its trigger.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopoverPlacement {
    #[default]
    BottomStart,
    BottomEnd,
    TopStart,
    TopEnd,
    LeftStart,
    LeftEnd,
    RightStart,
    RightEnd,
}

/// State of a single popover. Mutate `open` to show / hide.
///
/// The `trigger_bounds` and `content_size` fields are written by
/// the renderer during prepaint (the trigger element reports its
/// position; the content element reports its measured size). The
/// renderer then positions the content via `anchored`.
#[derive(Clone)]
pub struct PopoverState {
    pub open: bool,
    pub placement: PopoverPlacement,
    pub width: Option<Pixels>,
    pub dismiss_on_escape: bool,
    pub dismiss_on_outside_click: bool,

    pub trigger_bounds: Option<Bounds<Pixels>>,
    pub content_size: Option<Size<Pixels>>,

    on_close: Option<CloseFn>,
}

pub type CloseFn = Arc<dyn Fn(&mut gpui::Window, &mut App) + Send + Sync>;

impl PopoverState {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|_| Self {
            open: false,
            placement: PopoverPlacement::default(),
            width: None,
            dismiss_on_escape: true,
            dismiss_on_outside_click: true,
            trigger_bounds: None,
            content_size: None,
            on_close: None,
        })
    }

    pub fn open(&mut self) {
        self.open = true;
    }
    pub fn close(&mut self) {
        self.open = false;
    }
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn set_placement(&mut self, p: PopoverPlacement) {
        self.placement = p;
    }
    pub fn set_width(&mut self, w: Pixels) {
        self.width = Some(w);
    }
    pub fn set_dismiss_on_escape(&mut self, v: bool) {
        self.dismiss_on_escape = v;
    }
    pub fn set_dismiss_on_outside_click(&mut self, v: bool) {
        self.dismiss_on_outside_click = v;
    }
    pub fn set_on_close<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(&mut gpui::Window, &mut App),
    {
        self.on_close = Some(Arc::new(f));
    }
    pub fn invoke_close(&self, window: &mut gpui::Window, cx: &mut App) {
        if let Some(f) = &self.on_close {
            f(window, cx);
        }
    }
}

/// The headless popover props handed to `.apply(div)` or the
/// renderer's `DefaultPopover::default_render`.
///
/// The actual trigger / content elements are produced by the
/// caller and stashed in the entity. The headless apply just
/// attaches the id.
#[derive(Clone)]
pub struct PopoverProps {
    pub id: ElementId,
    pub state: Entity<PopoverState>,
}

pub fn popover(id: impl Into<ElementId>, state: Entity<PopoverState>) -> PopoverProps {
    PopoverProps {
        id: id.into(),
        state,
    }
}

impl PopoverProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}

/// Re-export `Point` so callers can use it from the headless
/// popover without importing `gpui::Point` directly.
pub use gpui::Point as AnchorPoint;
