//! Headless `toggle_button` — a button with a `selected` flag and
//! `on_toggle` callback. No visual.
//!
//! `apply` is purely a11y: focus + click. The caller (or the
//! renderer via `default_render`) owns every visual concern,
//! including hover / active feedback.
//!
//! For the common case, set `caption` / `icon` and call
//! `.render(cx)` — the headless wraps the content with the
//! renderer's tokens.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Pixels, SharedString,
    Stateful, StatefulInteractiveElement, Window, px,
};

use super::icon::IconSource;
use super::switch::ToggleCallback;
use crate::renderer::toggle_button::ToggleButtonRenderer;

#[derive(Clone)]
pub struct ToggleButtonProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub on_toggle: Option<ToggleCallback>,
    pub disabled: bool,
    /// Current `selected` state. The renderer reads this to
    /// paint the on/off look. The caller mutates it in response
    /// to `on_toggle` (or holds it in their own state entity).
    pub selected: bool,
    /// Action variant — `Neutral` / `Primary` / `Danger`. The
    /// renderer dispatches to `action.<variant>.{bg,fg}`.
    pub variant: crate::renderer::ActionVariantKind,
    /// Optional text caption. Used by `render`.
    pub caption: Option<SharedString>,
    /// Optional icon source. Used by `render`.
    pub icon: Option<IconSource>,
    /// Pixel size of the icon, when `icon` is set.
    pub icon_size: Pixels,
}

pub fn toggle_button(id: impl Into<ElementId>, cx: &mut App) -> ToggleButtonProps {
    ToggleButtonProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        on_toggle: None,
        disabled: false,
        selected: false,
        variant: crate::renderer::ActionVariantKind::default(),
        caption: None,
        icon: None,
        icon_size: px(16.),
    }
}

impl ToggleButtonProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(bool, Option<&ClickEvent>, &mut Window, &mut App),
    {
        self.on_toggle = Some(Arc::new(f));
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }
    pub fn variant(mut self, v: crate::renderer::ActionVariantKind) -> Self {
        self.variant = v;
        self
    }

    /// Set the toggle button's text caption. Consumed by `render`.
    pub fn caption(mut self, text: impl Into<SharedString>) -> Self {
        self.caption = Some(text.into());
        self
    }

    /// Set the toggle button's icon source. Consumed by `render`.
    pub fn icon(mut self, source: IconSource) -> Self {
        self.icon = Some(source);
        self
    }

    /// Override the icon's pixel size. Default: 16.
    pub fn icon_size(mut self, size: Pixels) -> Self {
        self.icon_size = size;
        self
    }

    /// Convenience: set both caption and icon in one call.
    pub fn caption_icon(
        mut self,
        caption: impl Into<SharedString>,
        icon: IconSource,
    ) -> Self {
        self.caption = Some(caption.into());
        self.icon = Some(icon);
        self
    }

    /// Wire the headless contract onto the caller's `el`.
    ///
    /// Purely a11y: id, focus, click (which fires
    /// `on_toggle(!selected, ...)`). No visual feedback
    /// injected — caller / renderer owns hover / active.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        let mut s = el.id(self.id.clone()).track_focus(&self.focus_handle);
        if !self.disabled
            && let Some(f) = self.on_toggle.clone()
        {
            let current = self.selected;
            s = s.on_click(move |ev, window, cx| {
                f(!current, Some(ev), window, cx);
            });
        }
        s
    }

    /// Render the toggle button using the registered `ToggleButtonRenderer`.
    ///
    /// Data flow is one-way: the renderer takes the full
    /// `ToggleButtonProps` and returns a fully-built
    /// `Stateful<Div>` (visuals + caption/icon children + hover /
    /// active + id + focus). This method only wires the
    /// `on_toggle` callback as `on_click` on top.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::ToggleButton as ToggleButtonMarker;

        let r: &Arc<dyn ToggleButtonRenderer> = cx
            .renderer_arc::<ToggleButtonMarker, dyn ToggleButtonRenderer>()
            .expect("ToggleButtonRenderer registered");

        let mut styled = r.compose(&self, &self.focus_handle, cx);
        if !self.disabled
            && let Some(f) = self.on_toggle.clone()
        {
            let current = self.selected;
            styled = styled.on_click(move |ev, window, cx| {
                f(!current, Some(ev), window, cx);
            });
        }
        styled
    }
}
