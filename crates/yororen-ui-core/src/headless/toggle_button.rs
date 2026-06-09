//! Headless `toggle_button` — a button with a `selected` flag and
//! `on_toggle` callback. No visual.
//!
//! `apply` is purely a11y: focus + click. The caller (or the
//! renderer via `default_render`) owns every visual concern,
//! including hover / active feedback.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Stateful,
    StatefulInteractiveElement, Styled, Window, div,
};

use super::switch::ToggleCallback;
use crate::renderer::RendererContext;
use crate::renderer::markers::ToggleButton as ToggleButtonMarker;
use crate::renderer::toggle_button::{ToggleButtonRenderState, ToggleButtonRenderer};
use crate::theme::ActiveTheme;

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
}

pub fn toggle_button(id: impl Into<ElementId>, cx: &mut App) -> ToggleButtonProps {
    ToggleButtonProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        on_toggle: None,
        disabled: false,
        selected: false,
        variant: crate::renderer::ActionVariantKind::default(),
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
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let r: &Arc<dyn ToggleButtonRenderer> = cx
            .renderer_arc::<ToggleButtonMarker, dyn ToggleButtonRenderer>()
            .expect("ToggleButtonRenderer registered");
        let state = ToggleButtonRenderState {
            variant: self.variant,
            selected: self.selected,
            disabled: self.disabled,
            custom_style: None,
        };
        let bg = r.bg(&state, theme);
        let fg = r.fg(&state, theme);
        let min_h = r.min_height(&state, theme);
        let radius = r.border_radius(&state, theme);
        let opacity = if self.disabled {
            r.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let el = div()
            .bg(bg)
            .text_color(fg)
            .min_h(min_h)
            .rounded(radius)
            .px(gpui::px(12.))
            .py(gpui::px(6.))
            .opacity(opacity)
            .flex()
            .items_center()
            .justify_center();
        let hover_bg = r.hover_bg(&state, theme);
        let active_bg = r.active_bg(&state, theme);
        self.apply(el)
            .hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
    }
}
