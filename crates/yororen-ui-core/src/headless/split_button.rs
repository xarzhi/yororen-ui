//! Headless `split_button` — primary action + chevron-triggered
//! dropdown menu.
//!
//! Data contract (pure data, no visual decisions):
//!
//! - `primary`            — click handler for the main label.
//! - `caption`            — text for the main label.
//! - `items`              — dropdown menu items (reused from
//!                          `dropdown_menu::DropdownItem`).
//! - `on_select`          — fires when the user picks an item.
//! - `state`              — the `Entity<DropdownMenuState>` that
//!                          stores `open` + highlighted index.
//!                          Caller mints it (typically in
//!                          `App::new`) so the toggle survives
//!                          across re-paints.
//! - `disabled`           — disables both primary + chevron.
//!
//! The factory mints two focus handles (`primary_focus` /
//! `chevron_focus`) so the renderer can compose two underlying
//! `button` props without needing `&mut App` at render time.
//!
//! Example (end-user app code):
//!
//! ```ignore
//! split_button("save-1", |_, _, cx| { /* primary save */ }, cx)
//!     .state(app.split_dd_state.clone())
//!     .caption("Save")
//!     .items(vec![
//!         DropdownItem::Item(DropdownMenuItem::new("save", "Save")),
//!         DropdownItem::Item(DropdownMenuItem::new("save_as", "Save as…")),
//!     ])
//!     .on_select(|id, _w, cx| { /* dispatch */ })
//!     .render(cx)
//! ```

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, Entity, FocusHandle, InteractiveElement, SharedString,
    Stateful, Window,
};

use crate::headless::dropdown_menu::{DropdownItem, DropdownMenuState};

pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;
pub type SelectCallback = Arc<dyn Fn(SharedString, &mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct SplitButtonProps {
    pub id: ElementId,
    pub primary: ClickCallback,
    pub disabled: bool,
    pub caption: Option<SharedString>,
    pub items: Vec<DropdownItem>,
    pub on_select: Option<SelectCallback>,
    pub state: Option<Entity<DropdownMenuState>>,
    /// Focus handle for the primary button (minted in factory).
    /// The renderer reuses this when composing the inner
    /// `ButtonProps` so the same id maps to a stable focus.
    pub primary_focus: FocusHandle,
    /// Focus handle for the chevron button (minted in factory).
    pub chevron_focus: FocusHandle,
}

pub fn split_button(
    id: impl Into<ElementId>,
    primary: impl 'static + Send + Sync + Fn(&ClickEvent, &mut Window, &mut App),
    cx: &mut App,
) -> SplitButtonProps {
    SplitButtonProps {
        id: id.into(),
        primary: Arc::new(primary),
        disabled: false,
        caption: None,
        items: Vec::new(),
        on_select: None,
        state: None,
        primary_focus: cx.focus_handle(),
        chevron_focus: cx.focus_handle(),
    }
}

impl SplitButtonProps {
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
    pub fn items(mut self, items: Vec<DropdownItem>) -> Self {
        self.items = items;
        self
    }
    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(SharedString, &mut Window, &mut App),
    {
        self.on_select = Some(Arc::new(f));
        self
    }
    pub fn state(mut self, s: Entity<DropdownMenuState>) -> Self {
        self.state = Some(s);
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the split button using the registered
    /// `SplitButtonRenderer`. Returns a `Stateful<Div>` whose
    /// children are the trigger row (primary + chevron) and,
    /// when `state.open`, an absolutely-positioned dropdown
    /// body composed of `panel` + `list_item` renderers.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::split_button::SplitButtonRenderer;
        use crate::renderer::markers::SplitButton as SplitButtonMarker;

        let r: &Arc<dyn SplitButtonRenderer> = cx
            .renderer_arc::<SplitButtonMarker, dyn SplitButtonRenderer>()
            .expect("SplitButtonRenderer registered");
        let div = r.compose(&self, cx);
        self.apply(div)
    }
}
